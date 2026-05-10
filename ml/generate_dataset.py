import csv
import random
import os

FEATURES = [
    "wx_count",
    "mprotect_sum",
    "ptrace_sum",
    "syscall_rate",
    "blocked_ratio",
    "uid_is_root",
    "event_burst_ratio",
    "unique_event_diversity",
    "label"
]

def generate_dataset(filename="ml/dataset.csv", n_samples=5000):
    os.makedirs(os.path.dirname(filename), exist_ok=True)
    
    with open(filename, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(FEATURES)
        
        for _ in range(n_samples):
            is_attack = random.random() < 0.5
            
            if is_attack:
                # Malicious profile (e.g., PwnKit, DirtyPipe, Shellcode injection)
                wx_count = random.randint(1, 10)
                mprotect_sum = random.randint(2, 8)
                ptrace_sum = random.randint(0, 3)
                syscall_rate = random.uniform(50.0, 500.0)
                blocked_ratio = random.uniform(0.1, 0.9)
                uid_is_root = 1 if random.random() < 0.6 else 0
                event_burst_ratio = random.uniform(2.0, 10.0)
                unique_event_diversity = random.uniform(0.5, 1.0)
                label = 1
            else:
                # Benign profile (e.g., bash, nginx, systemd)
                wx_count = 0 if random.random() < 0.95 else 1
                mprotect_sum = random.randint(0, 2)
                ptrace_sum = 0
                syscall_rate = random.uniform(1.0, 40.0)
                blocked_ratio = random.uniform(0.0, 0.05)
                uid_is_root = 1 if random.random() < 0.1 else 0
                event_burst_ratio = random.uniform(0.1, 1.5)
                unique_event_diversity = random.uniform(0.1, 0.4)
                label = 0
                
            writer.writerow([
                wx_count, mprotect_sum, ptrace_sum, round(syscall_rate, 2),
                round(blocked_ratio, 3), uid_is_root, round(event_burst_ratio, 2),
                round(unique_event_diversity, 3), label
            ])
            
    print(f"Generated {n_samples} samples in {filename}")

if __name__ == "__main__":
    generate_dataset()
