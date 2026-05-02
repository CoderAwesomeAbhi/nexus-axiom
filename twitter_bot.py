#!/usr/bin/env python3
"""
Nexus Axiom - Twitter Bot
Auto-tweet when exploits are blocked (with user permission)
"""

import json
import hashlib
from datetime import datetime

class TwitterBot:
    def __init__(self):
        self.config_file = 'twitter_config.json'
        self.load_config()
    
    def load_config(self):
        """Load Twitter config"""
        try:
            with open(self.config_file, 'r') as f:
                self.config = json.load(f)
        except FileNotFoundError:
            self.config = {
                'enabled': False,
                'auto_tweet': False,
                'api_key': '',
                'api_secret': '',
                'access_token': '',
                'access_secret': ''
            }
    
    def save_config(self):
        """Save config"""
        with open(self.config_file, 'w') as f:
            json.dump(self.config, f, indent=2)
    
    def generate_tweet(self, exploit_data):
        """Generate tweet text"""
        templates = [
            "🛡️ Just blocked a {type} exploit with @NexusAxiom!\n\nProcess: {process}\nStatus: ✅ KILLED\n\n#cybersecurity #eBPF #infosec",
            "💀 Another one bites the dust!\n\n{type} attack blocked by @NexusAxiom\nPID: {pid}\n\nYour move, hackers. 😎\n\n#security #Linux",
            "🚨 EXPLOIT TERMINATED\n\nType: {type}\nProcess: {process}\nResult: ✅ BLOCKED\n\nPowered by @NexusAxiom\n\n#eBPF #cybersecurity",
            "⚡ Real-time protection in action!\n\nBlocked: {type}\nTime: <1ms\n\nThis is why I use @NexusAxiom 🛡️\n\n#infosec #security",
            "🎯 Exploit blocked before execution!\n\n{type} → ✅ KILLED\n\nZero-day? No problem.\n\n@NexusAxiom #cybersecurity"
        ]
        
        import random
        template = random.choice(templates)
        
        return template.format(
            type=exploit_data.get('type', 'UNKNOWN'),
            process=exploit_data.get('process', 'unknown'),
            pid=exploit_data.get('pid', 0)
        )
    
    def tweet_exploit_block(self, exploit_data):
        """Tweet about blocked exploit"""
        if not self.config['enabled'] or not self.config['auto_tweet']:
            return False
        
        tweet_text = self.generate_tweet(exploit_data)
        
        # In production, use tweepy or similar
        print("📱 WOULD TWEET:")
        print("-" * 60)
        print(tweet_text)
        print("-" * 60)
        print()
        
        return True
    
    def setup(self):
        """Interactive setup"""
        print("🐦 TWITTER BOT SETUP")
        print("=" * 60)
        print()
        print("Auto-tweet when exploits are blocked!")
        print()
        print("⚠️  This requires Twitter API access:")
        print("  1. Go to https://developer.twitter.com")
        print("  2. Create an app")
        print("  3. Get API keys")
        print()
        
        enable = input("Enable Twitter bot? (y/n): ").strip().lower()
        
        if enable == 'y':
            self.config['enabled'] = True
            
            print()
            print("API Keys (leave blank to skip):")
            api_key = input("API Key: ").strip()
            api_secret = input("API Secret: ").strip()
            access_token = input("Access Token: ").strip()
            access_secret = input("Access Secret: ").strip()
            
            if api_key and api_secret:
                self.config['api_key'] = api_key
                self.config['api_secret'] = api_secret
                self.config['access_token'] = access_token
                self.config['access_secret'] = access_secret
            
            print()
            auto = input("Auto-tweet every block? (y/n): ").strip().lower()
            self.config['auto_tweet'] = (auto == 'y')
            
            self.save_config()
            
            print()
            print("✅ Twitter bot configured!")
            print()
            
            if self.config['auto_tweet']:
                print("🎉 You'll now auto-tweet exploit blocks!")
                print("   Use #NexusAxiom to join the community")
            else:
                print("💡 Run 'nexus-axiom tweet' to manually tweet blocks")
        else:
            print()
            print("Twitter bot disabled.")
        
        print()
        print("=" * 60)

def demo_tweet():
    """Demo tweet generation"""
    bot = TwitterBot()
    
    exploit_data = {
        'type': 'CVE-2021-4034 (PwnKit)',
        'process': 'exploit_pwnkit',
        'pid': 1337
    }
    
    print("🐦 DEMO: Auto-Tweet Feature")
    print("=" * 60)
    print()
    print("When you block an exploit, Nexus Axiom can auto-tweet it!")
    print()
    
    bot.tweet_exploit_block(exploit_data)
    
    print("💡 Benefits:")
    print("  • Build your security reputation")
    print("  • Show off your protection")
    print("  • Help spread awareness")
    print("  • Join the #NexusAxiom community")
    print()
    print("Setup: python3 twitter_bot.py setup")

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1 and sys.argv[1] == 'setup':
        bot = TwitterBot()
        bot.setup()
    elif len(sys.argv) > 1 and sys.argv[1] == 'demo':
        demo_tweet()
    else:
        print("Usage:")
        print("  python3 twitter_bot.py setup  # Configure Twitter bot")
        print("  python3 twitter_bot.py demo   # See demo tweet")
