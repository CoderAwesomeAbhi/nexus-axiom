#!/usr/bin/env python3
"""
collect_training_data.py — Parse Nexus Axiom JSON logs to extract ML features.
"""

import json
import csv
import sys
from collections import defaultdict

FEATURES = [
    "wx_count", "mprotect_sum", "ptrace_sum", "syscall_rate",
    "blocked_ratio", "uid_is_root", "event_burst_ratio", "unique_event_diversity", "label"
]

def parse_logs(log_path, output_path):
    print(f"Parsing logs from {log_path}...")
    
    # Store events grouped by PID
    pid_events = defaultdict(list)
    
    try:
        with open(log_path, 'r') as f:
            for line in f:
                if not line.strip():
                    continue
                try:
                    event = json.loads(line)
                    pid_events[event.get('pid', 0)].append(event)
                except json.JSONDecodeError:
                    continue
    except FileNotFoundError:
        print(f"Error: Log file {log_path} not found.")
        sys.exit(1)
        
    print(f"Found events for {len(pid_events)} processes. Extracting features...")
    
    samples = []
    for pid, events in pid_events.items():
        if len(events) < 5:
            continue # Skip processes with too few events
            
        wx_count = sum(1 for e in events if e.get('event_type') == 'W^X_MMAP')
        mprotect_sum = sum(1 for e in events if e.get('event_type') == 'W^X_MPROTECT')
        ptrace_sum = sum(1 for e in events if e.get('event_type') == 'PTRACE')
        
        # Calculate time-based metrics (simplified)
        timestamps = [e.get('timestamp', 0) for e in events]
        if max(timestamps) > min(timestamps):
            duration_sec = (max(timestamps) - min(timestamps)) / 1000.0
            syscall_rate = len(events) / max(duration_sec, 1.0)
        else:
            syscall_rate = len(events)
            
        blocked_count = sum(1 for e in events if e.get('blocked', False))
        blocked_ratio = blocked_count / len(events)
        
        uid_is_root = 1 if any(e.get('uid') == 0 for e in events) else 0
        
        event_burst_ratio = syscall_rate * 0.5 # Simplified heuristic
        
        unique_events = len(set(e.get('event_type') for e in events))
        unique_event_diversity = unique_events / max(len(events), 1)
        
        # Determine label automatically (if it was blocked, assume malicious for training)
        label = 1 if blocked_count > 0 else 0
        
        samples.append([
            wx_count, mprotect_sum, ptrace_sum, round(syscall_rate, 2),
            round(blocked_ratio, 3), uid_is_root, round(event_burst_ratio, 2),
            round(unique_event_diversity, 3), label
        ])
        
    with open(output_path, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(FEATURES)
        writer.writerows(samples)
        
    print(f"Extracted {len(samples)} training samples to {output_path}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python collect_training_data.py <input.json> <output.csv>")
        sys.exit(1)
    parse_logs(sys.argv[1], sys.argv[2])
