"""
Nexus Axiom — AI Exploit Prediction Engine
LSTM-based sequence model that predicts attack likelihood from behavioral telemetry.

Input features (per timestep):
  [event_type, prot_flags, uid, is_blocked, syscall_rate, mmap_count, mprotect_count, ptrace_count]

Output: probability of exploit attempt in next N events (0.0 – 1.0)
"""

import json
import math
import os
import struct
import sys
import time
from pathlib import Path
from typing import Optional

import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader, TensorDataset

# ── Constants ────────────────────────────────────────────────────────────────
FEATURE_DIM   = 8    # features per timestep
SEQ_LEN       = 32   # look-back window
HIDDEN_DIM    = 64
NUM_LAYERS    = 2
DROPOUT       = 0.2
BATCH_SIZE    = 64
EPOCHS        = 30
LR            = 1e-3
MODEL_PATH    = Path(__file__).parent / "exploit_predictor.pt"
DATA_PATH     = Path(__file__).parent / "training_data.jsonl"
IPC_SOCKET    = "/tmp/nexus_predictor.sock"

# ── Model ─────────────────────────────────────────────────────────────────────
class ExploitPredictor(nn.Module):
    """Bidirectional LSTM with attention for exploit sequence classification."""

    def __init__(self, feature_dim=FEATURE_DIM, hidden_dim=HIDDEN_DIM,
                 num_layers=NUM_LAYERS, dropout=DROPOUT):
        super().__init__()
        self.lstm = nn.LSTM(
            input_size=feature_dim,
            hidden_size=hidden_dim,
            num_layers=num_layers,
            batch_first=True,
            bidirectional=True,
            dropout=dropout if num_layers > 1 else 0.0,
        )
        # Attention over time dimension
        self.attn = nn.Linear(hidden_dim * 2, 1)
        self.classifier = nn.Sequential(
            nn.Linear(hidden_dim * 2, 32),
            nn.ReLU(),
            nn.Dropout(dropout),
            nn.Linear(32, 1),
            nn.Sigmoid(),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        # x: (batch, seq_len, feature_dim)
        out, _ = self.lstm(x)                          # (batch, seq, hidden*2)
        attn_w = torch.softmax(self.attn(out), dim=1)  # (batch, seq, 1)
        ctx = (attn_w * out).sum(dim=1)                # (batch, hidden*2)
        return self.classifier(ctx).squeeze(-1)        # (batch,)


# ── Feature normalisation ─────────────────────────────────────────────────────
# Empirical ranges for each feature — keeps inputs in [0, 1]
_NORM = torch.tensor([
    7.0,    # event_type  (1–7)
    7.0,    # prot_flags  (0–7 bitmask)
    65535.0,# uid
    1.0,    # is_blocked
    1000.0, # syscall_rate (per second)
    50.0,   # mmap_count
    20.0,   # mprotect_count
    5.0,    # ptrace_count
], dtype=torch.float32)

def normalise(seq: torch.Tensor) -> torch.Tensor:
    return (seq / _NORM.to(seq.device)).clamp(0.0, 1.0)


# ── Training data generation (synthetic + real) ───────────────────────────────
def generate_synthetic_data(n_samples: int = 2000):
    """
    Generates labelled sequences.
    Label=1: exploit pattern (W^X + mprotect spike + ptrace)
    Label=0: benign pattern
    """
    X, y = [], []
    rng = torch.Generator()
    rng.manual_seed(42)

    for i in range(n_samples):
        is_attack = i < n_samples // 2
        seq = torch.zeros(SEQ_LEN, FEATURE_DIM)

        if is_attack:
            # Simulate: normal → recon → shellcode injection → execution
            for t in range(SEQ_LEN):
                phase = t / SEQ_LEN
                seq[t, 0] = 1 + int(phase * 6)          # escalating event types
                seq[t, 1] = 7 if phase > 0.6 else 1     # PROT_READ|WRITE|EXEC spike
                seq[t, 2] = torch.randint(0, 1000, (1,), generator=rng).item()
                seq[t, 3] = 1.0 if phase > 0.7 else 0.0 # blocked events
                seq[t, 4] = 200 + phase * 800            # syscall rate climbs
                seq[t, 5] = phase * 30                   # mmap count
                seq[t, 6] = phase * 15 if phase > 0.5 else 0  # mprotect spike
                seq[t, 7] = 1.0 if phase > 0.8 else 0.0 # ptrace
        else:
            # Benign: low, stable activity
            seq[:, 0] = torch.randint(1, 4, (SEQ_LEN,), generator=rng).float()
            seq[:, 1] = torch.randint(0, 3, (SEQ_LEN,), generator=rng).float()
            seq[:, 2] = torch.randint(1000, 60000, (SEQ_LEN,), generator=rng).float()
            seq[:, 3] = 0.0
            seq[:, 4] = torch.rand(SEQ_LEN, generator=rng) * 100
            seq[:, 5] = torch.rand(SEQ_LEN, generator=rng) * 5
            seq[:, 6] = 0.0
            seq[:, 7] = 0.0

        X.append(seq)
        y.append(float(is_attack))

    return torch.stack(X), torch.tensor(y, dtype=torch.float32)


def load_real_data() -> Optional[tuple]:
    """Load real events from training_data.jsonl if it exists."""
    if not DATA_PATH.exists():
        return None

    sequences, labels = [], []
    buffer: list = []
    label = 0

    with open(DATA_PATH) as f:
        for line in f:
            try:
                ev = json.loads(line)
                feat = [
                    ev.get("event_type", 0),
                    ev.get("prot", 0),
                    ev.get("uid", 0),
                    float(ev.get("blocked", False)),
                    ev.get("syscall_rate", 0.0),
                    ev.get("mmap_count", 0),
                    ev.get("mprotect_count", 0),
                    ev.get("ptrace_count", 0),
                ]
                buffer.append(feat)
                label = max(label, int(ev.get("is_attack", 0)))

                if len(buffer) == SEQ_LEN:
                    sequences.append(buffer)
                    labels.append(label)
                    buffer = []
                    label = 0
            except (json.JSONDecodeError, KeyError):
                continue

    if not sequences:
        return None

    X = torch.tensor(sequences, dtype=torch.float32)
    y = torch.tensor(labels, dtype=torch.float32)
    return X, y


# ── Training ──────────────────────────────────────────────────────────────────
def train(save_path: Path = MODEL_PATH):
    print("🧠 Nexus Axiom — Training Exploit Predictor")

    # Combine synthetic + real data
    X_syn, y_syn = generate_synthetic_data(2000)
    real = load_real_data()
    if real:
        X_real, y_real = real
        X = torch.cat([X_syn, X_real])
        y = torch.cat([y_syn, y_real])
        print(f"   Loaded {len(X_real)} real sequences + 2000 synthetic")
    else:
        X, y = X_syn, y_syn
        print("   Using 2000 synthetic sequences (no real data yet)")

    X = normalise(X)

    # Train/val split
    n = len(X)
    idx = torch.randperm(n)
    split = int(n * 0.8)
    X_tr, y_tr = X[idx[:split]], y[idx[:split]]
    X_val, y_val = X[idx[split:]], y[idx[split:]]

    tr_loader = DataLoader(TensorDataset(X_tr, y_tr), batch_size=BATCH_SIZE, shuffle=True)
    val_loader = DataLoader(TensorDataset(X_val, y_val), batch_size=BATCH_SIZE)

    model = ExploitPredictor()
    optimizer = optim.AdamW(model.parameters(), lr=LR, weight_decay=1e-4)
    scheduler = optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=EPOCHS)
    criterion = nn.BCELoss()

    best_val_loss = math.inf
    for epoch in range(1, EPOCHS + 1):
        model.train()
        tr_loss = 0.0
        for xb, yb in tr_loader:
            optimizer.zero_grad()
            pred = model(xb)
            loss = criterion(pred, yb)
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            optimizer.step()
            tr_loss += loss.item()
        scheduler.step()

        model.eval()
        val_loss = 0.0
        correct = 0
        with torch.no_grad():
            for xb, yb in val_loader:
                pred = model(xb)
                val_loss += criterion(pred, yb).item()
                correct += ((pred > 0.5) == yb.bool()).sum().item()

        val_acc = correct / len(X_val) * 100
        avg_tr = tr_loss / len(tr_loader)
        avg_val = val_loss / len(val_loader)

        if avg_val < best_val_loss:
            best_val_loss = avg_val
            torch.save(model.state_dict(), save_path)

        if epoch % 5 == 0 or epoch == 1:
            print(f"   Epoch {epoch:3d}/{EPOCHS}  "
                  f"train={avg_tr:.4f}  val={avg_val:.4f}  acc={val_acc:.1f}%")

    print(f"✅ Model saved → {save_path}  (best val loss: {best_val_loss:.4f})")
    return model


# ── Inference server (Unix socket IPC with Rust) ──────────────────────────────
def load_model(path: Path = MODEL_PATH) -> ExploitPredictor:
    model = ExploitPredictor()
    if path.exists():
        model.load_state_dict(torch.load(path, map_location="cpu", weights_only=True))
        print(f"✅ Loaded model from {path}")
    else:
        print("⚠️  No saved model found — using untrained weights. Run: python predictor.py train")
    model.eval()
    return model


def predict_sequence(model: ExploitPredictor, events: list) -> dict:
    """
    events: list of dicts with keys matching FEATURE_DIM fields.
    Returns {"confidence": float, "threat_level": str, "attack_type": str}
    """
    # Pad or truncate to SEQ_LEN
    feats = []
    for ev in events[-SEQ_LEN:]:
        feats.append([
            ev.get("event_type", 0),
            ev.get("prot", 0),
            ev.get("uid", 0),
            float(ev.get("blocked", False)),
            ev.get("syscall_rate", 0.0),
            ev.get("mmap_count", 0),
            ev.get("mprotect_count", 0),
            ev.get("ptrace_count", 0),
        ])

    # Pad with zeros if shorter than SEQ_LEN
    while len(feats) < SEQ_LEN:
        feats.insert(0, [0.0] * FEATURE_DIM)

    x = torch.tensor([feats], dtype=torch.float32)
    x = normalise(x)

    with torch.no_grad():
        confidence = model(x).item()

    # Classify threat
    if confidence >= 0.85:
        threat_level = "CRITICAL"
        attack_type = _classify_attack(events)
    elif confidence >= 0.65:
        threat_level = "HIGH"
        attack_type = _classify_attack(events)
    elif confidence >= 0.40:
        threat_level = "MEDIUM"
        attack_type = "Suspicious Activity"
    else:
        threat_level = "LOW"
        attack_type = "Benign"

    return {
        "confidence": round(confidence, 4),
        "threat_level": threat_level,
        "attack_type": attack_type,
        "timestamp": time.time(),
    }


def _classify_attack(events: list) -> str:
    """Heuristic attack classification from recent events."""
    has_wx    = any(e.get("prot", 0) & 0x7 == 0x7 for e in events)
    has_ptrace = any(e.get("event_type", 0) == 5 for e in events)
    has_exec  = any(e.get("event_type", 0) == 6 for e in events)
    mprotect_count = sum(1 for e in events if e.get("event_type", 0) == 4)

    if has_wx and mprotect_count > 3:
        return "Shellcode Injection (W^X + mprotect chain)"
    if has_ptrace:
        return "Process Injection via ptrace"
    if has_exec and has_wx:
        return "ROP Chain / JIT Spray"
    if mprotect_count > 5:
        return "Memory Permission Escalation"
    return "Exploit Attempt"


# ── Unix socket server for Rust IPC ──────────────────────────────────────────
def run_server(model_path: Path = MODEL_PATH):
    """
    Listens on a Unix socket. Protocol (line-delimited JSON):
      Request:  {"events": [...]}
      Response: {"confidence": 0.92, "threat_level": "CRITICAL", "attack_type": "..."}
    """
    import socket

    model = load_model(model_path)

    if os.path.exists(IPC_SOCKET):
        os.unlink(IPC_SOCKET)

    srv = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    srv.bind(IPC_SOCKET)
    srv.listen(5)
    os.chmod(IPC_SOCKET, 0o666)
    print(f"🔌 Predictor server listening on {IPC_SOCKET}")

    while True:
        conn, _ = srv.accept()
        try:
            data = b""
            while True:
                chunk = conn.recv(4096)
                if not chunk:
                    break
                data += chunk
                if b"\n" in data:
                    break

            req = json.loads(data.decode().strip())
            result = predict_sequence(model, req.get("events", []))
            conn.sendall((json.dumps(result) + "\n").encode())
        except Exception as e:
            err = json.dumps({"error": str(e), "confidence": 0.0, "threat_level": "UNKNOWN"})
            try:
                conn.sendall((err + "\n").encode())
            except Exception:
                pass
        finally:
            conn.close()


# ── CLI ───────────────────────────────────────────────────────────────────────
if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "server"

    if cmd == "train":
        train()
    elif cmd == "server":
        run_server()
    elif cmd == "demo":
        model = load_model()
        # Simulate an attack sequence
        attack_events = [
            {"event_type": i % 7 + 1, "prot": 7 if i > 20 else 1,
             "uid": 1000, "blocked": i > 25, "syscall_rate": 100 + i * 30,
             "mmap_count": i // 3, "mprotect_count": max(0, i - 20),
             "ptrace_count": 1 if i > 28 else 0}
            for i in range(32)
        ]
        result = predict_sequence(model, attack_events)
        print(f"\n🎯 Prediction Result:")
        print(f"   Confidence:   {result['confidence']:.1%}")
        print(f"   Threat Level: {result['threat_level']}")
        print(f"   Attack Type:  {result['attack_type']}")
    else:
        print(f"Usage: python predictor.py [train|server|demo]")
