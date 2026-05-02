#!/usr/bin/env python3
"""
Nexus Axiom - Global Leaderboard
Gamify security: compete for most exploits blocked
"""

import json
import hashlib
import socket
from datetime import datetime

class Leaderboard:
    def __init__(self):
        self.stats_file = 'leaderboard_stats.json'
        self.load_stats()
    
    def load_stats(self):
        """Load local stats"""
        try:
            with open(self.stats_file, 'r') as f:
                self.stats = json.load(f)
        except FileNotFoundError:
            self.stats = {
                'hostname': socket.gethostname(),
                'user_id': self.generate_user_id(),
                'exploits_blocked': 0,
                'uptime_hours': 0,
                'cves_blocked': [],
                'first_seen': datetime.now().isoformat(),
                'last_updated': datetime.now().isoformat()
            }
            self.save_stats()
    
    def save_stats(self):
        """Save stats to file"""
        self.stats['last_updated'] = datetime.now().isoformat()
        with open(self.stats_file, 'w') as f:
            json.dump(self.stats, f, indent=2)
    
    def generate_user_id(self):
        """Generate anonymous user ID"""
        hostname = socket.gethostname()
        return hashlib.sha256(hostname.encode()).hexdigest()[:16]
    
    def record_block(self, exploit_type, cve=None):
        """Record an exploit block"""
        self.stats['exploits_blocked'] += 1
        if cve and cve not in self.stats['cves_blocked']:
            self.stats['cves_blocked'].append(cve)
        self.save_stats()
    
    def get_rank(self):
        """Calculate rank based on stats"""
        blocks = self.stats['exploits_blocked']
        
        if blocks < 10:
            return "🥉 Rookie Guardian"
        elif blocks < 50:
            return "🥈 Security Sentinel"
        elif blocks < 100:
            return "🥇 Exploit Hunter"
        elif blocks < 500:
            return "💎 Elite Defender"
        elif blocks < 1000:
            return "👑 Master Protector"
        else:
            return "🏆 LEGENDARY GUARDIAN"
    
    def display_stats(self):
        """Display user stats"""
        print("🏆 NEXUS AXIOM LEADERBOARD")
        print("=" * 60)
        print()
        print(f"User ID: {self.stats['user_id']}")
        print(f"Hostname: {self.stats['hostname']}")
        print(f"Rank: {self.get_rank()}")
        print()
        print("📊 YOUR STATS:")
        print(f"  Exploits Blocked: {self.stats['exploits_blocked']:,}")
        print(f"  CVEs Blocked: {len(self.stats['cves_blocked'])}")
        print(f"  Uptime: {self.stats['uptime_hours']} hours")
        print(f"  Member Since: {self.stats['first_seen'][:10]}")
        print()
        
        # Show progress to next rank
        blocks = self.stats['exploits_blocked']
        next_milestone = None
        
        if blocks < 10:
            next_milestone = (10, "Security Sentinel")
        elif blocks < 50:
            next_milestone = (50, "Exploit Hunter")
        elif blocks < 100:
            next_milestone = (100, "Elite Defender")
        elif blocks < 500:
            next_milestone = (500, "Master Protector")
        elif blocks < 1000:
            next_milestone = (1000, "LEGENDARY GUARDIAN")
        
        if next_milestone:
            remaining = next_milestone[0] - blocks
            progress = (blocks / next_milestone[0]) * 100
            print(f"📈 NEXT RANK: {next_milestone[1]}")
            print(f"   Progress: [{'█' * int(progress/5)}{'░' * (20-int(progress/5))}] {progress:.1f}%")
            print(f"   {remaining} more exploits to block!")
            print()
        
        print("=" * 60)
        print()
        print("💡 Tips to rank up:")
        print("  • Run the Exploit Zoo daily")
        print("  • Keep Nexus Axiom running 24/7")
        print("  • Test against new CVEs")
        print()
        print("🌍 Global Leaderboard (coming soon):")
        print("  • Compare with other users")
        print("  • Weekly challenges")
        print("  • Exclusive badges")
        print()
        print("⭐ Share your rank on Twitter with #NexusAxiom")

def simulate_blocks():
    """Simulate some blocks for demo"""
    lb = Leaderboard()
    
    # Simulate blocks
    for i in range(42):
        lb.record_block('W^X_MEMORY')
    
    lb.record_block('PRIV_ESC', 'CVE-2021-4034')
    lb.record_block('PRIV_ESC', 'CVE-2021-3156')
    lb.record_block('FILE_WRITE', 'CVE-2022-0847')
    
    lb.stats['uptime_hours'] = 168  # 1 week
    lb.save_stats()
    
    lb.display_stats()

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1 and sys.argv[1] == 'demo':
        simulate_blocks()
    else:
        lb = Leaderboard()
        lb.display_stats()
