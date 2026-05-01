#!/usr/bin/env python3
"""
Nexus Axiom - Instant Alert System
Send exploit alerts to Slack/Discord/Telegram
"""

import json
import requests
import sys
from datetime import datetime

class AlertSystem:
    def __init__(self, webhook_url, platform='slack'):
        self.webhook_url = webhook_url
        self.platform = platform
    
    def send_slack_alert(self, exploit_data):
        """Send alert to Slack"""
        payload = {
            "text": "🚨 EXPLOIT BLOCKED BY NEXUS AXIOM",
            "attachments": [{
                "color": "danger",
                "fields": [
                    {"title": "Process", "value": exploit_data['process'], "short": True},
                    {"title": "PID", "value": str(exploit_data['pid']), "short": True},
                    {"title": "Attack Type", "value": exploit_data['type'], "short": True},
                    {"title": "Severity", "value": "CRITICAL", "short": True},
                    {"title": "Status", "value": "✅ KILLED", "short": True},
                    {"title": "Time", "value": exploit_data['time'], "short": True}
                ],
                "footer": "Nexus Axiom Security",
                "ts": int(datetime.now().timestamp())
            }]
        }
        
        response = requests.post(self.webhook_url, json=payload)
        return response.status_code == 200
    
    def send_discord_alert(self, exploit_data):
        """Send alert to Discord"""
        payload = {
            "embeds": [{
                "title": "🚨 EXPLOIT BLOCKED",
                "description": f"Nexus Axiom terminated a {exploit_data['type']} attack",
                "color": 15158332,  # Red
                "fields": [
                    {"name": "Process", "value": exploit_data['process'], "inline": True},
                    {"name": "PID", "value": str(exploit_data['pid']), "inline": True},
                    {"name": "Attack", "value": exploit_data['type'], "inline": True},
                    {"name": "Status", "value": "✅ KILLED", "inline": True}
                ],
                "footer": {"text": "Nexus Axiom Security"},
                "timestamp": datetime.utcnow().isoformat()
            }]
        }
        
        response = requests.post(self.webhook_url, json=payload)
        return response.status_code == 204
    
    def send_telegram_alert(self, exploit_data):
        """Send alert to Telegram"""
        message = f"""
🚨 *EXPLOIT BLOCKED*

*Process:* {exploit_data['process']}
*PID:* {exploit_data['pid']}
*Attack:* {exploit_data['type']}
*Status:* ✅ KILLED
*Time:* {exploit_data['time']}

_Nexus Axiom Security_
"""
        
        payload = {
            "text": message,
            "parse_mode": "Markdown"
        }
        
        response = requests.post(self.webhook_url, json=payload)
        return response.status_code == 200
    
    def send_alert(self, exploit_data):
        """Send alert to configured platform"""
        if self.platform == 'slack':
            return self.send_slack_alert(exploit_data)
        elif self.platform == 'discord':
            return self.send_discord_alert(exploit_data)
        elif self.platform == 'telegram':
            return self.send_telegram_alert(exploit_data)
        else:
            print(f"Unknown platform: {self.platform}")
            return False

def setup_alerts():
    """Interactive setup for alerts"""
    print("🔔 NEXUS AXIOM - ALERT SETUP")
    print("=" * 60)
    print()
    print("Get instant notifications when exploits are blocked!")
    print()
    print("Supported platforms:")
    print("  1. Slack")
    print("  2. Discord")
    print("  3. Telegram")
    print()
    
    choice = input("Choose platform (1-3): ").strip()
    
    platform_map = {'1': 'slack', '2': 'discord', '3': 'telegram'}
    platform = platform_map.get(choice, 'slack')
    
    print()
    print(f"Selected: {platform.title()}")
    print()
    
    if platform == 'slack':
        print("📝 Setup Instructions:")
        print("  1. Go to https://api.slack.com/messaging/webhooks")
        print("  2. Create a new webhook")
        print("  3. Copy the webhook URL")
        print()
    elif platform == 'discord':
        print("📝 Setup Instructions:")
        print("  1. Go to Server Settings > Integrations > Webhooks")
        print("  2. Create a new webhook")
        print("  3. Copy the webhook URL")
        print()
    elif platform == 'telegram':
        print("📝 Setup Instructions:")
        print("  1. Create a bot with @BotFather")
        print("  2. Get your chat ID")
        print("  3. Use: https://api.telegram.org/bot<TOKEN>/sendMessage?chat_id=<CHAT_ID>")
        print()
    
    webhook_url = input("Paste webhook URL: ").strip()
    
    # Save config
    config = {
        'platform': platform,
        'webhook_url': webhook_url,
        'enabled': True
    }
    
    with open('alert_config.json', 'w') as f:
        json.dump(config, f, indent=2)
    
    print()
    print("✅ Alert system configured!")
    print()
    print("Testing alert...")
    
    # Send test alert
    alert_system = AlertSystem(webhook_url, platform)
    test_data = {
        'process': 'test_exploit',
        'pid': 1337,
        'type': 'W^X_MEMORY',
        'time': datetime.now().strftime('%Y-%m-%d %H:%M:%S')
    }
    
    if alert_system.send_alert(test_data):
        print("✅ Test alert sent successfully!")
        print()
        print("🎉 You'll now get instant notifications when exploits are blocked!")
    else:
        print("❌ Failed to send test alert. Check your webhook URL.")

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == 'setup':
        setup_alerts()
    else:
        print("Usage: python3 alert_system.py setup")
