# 🚀 LAUNCH DAY CHEAT SHEET

## HackerNews Post (Day 2, 9-10 AM EST)

### Title
```
Show HN: Nexus Axiom – First eBPF tool that actually kills exploits (not just logs)
```

### Body
```
Hi HN! I built Nexus Axiom after getting frustrated that every security tool 
(Falco, Tetragon, etc.) can only *watch* exploits happen, not stop them.

The problem: They use tracepoints/kprobes which run AFTER syscalls complete.

Nexus Axiom uses LSM hooks that intercept syscalls BEFORE they execute, 
so it can actually block W^X memory and kill exploit processes instantly.

Demo: https://github.com/YOUR_USERNAME/nexus-axiom

I tested it against CVE-2021-4034 (PwnKit) and other real exploits - 
they all get killed before doing damage.

Would love feedback from the security folks here!
```

### URL
```
https://github.com/YOUR_USERNAME/nexus-axiom
```

---

## Reddit Post (Day 3)

### r/netsec Title
```
[Tool] Nexus Axiom: eBPF security tool that blocks exploits using LSM hooks (not just tracepoints)
```

### r/netsec Body
```
I built an eBPF security tool that actually blocks exploits instead of just logging them.

Most tools (Falco, Tetragon) use tracepoints which run after syscalls complete.
Nexus Axiom uses LSM hooks which run before, so it can return -EPERM and kill processes.

Tested against:
- CVE-2021-4034 (PwnKit) ✓ Blocked
- CVE-2021-3156 (Sudo) ✓ Blocked  
- CVE-2022-0847 (Dirty Pipe) ✓ Blocked

GitHub: https://github.com/YOUR_USERNAME/nexus-axiom

Performance: <5% overhead, 2MB memory

Open source (GPL-3.0), looking for contributors!
```

---

## Twitter Thread (Day 4-5)

```
🧵 I spent 3 months building an eBPF security tool that actually KILLS exploits.

Here's why every other tool fails, and how Nexus Axiom is different: 🧵👇

1/ The problem with Falco, Tetragon, etc:
They use tracepoints/kprobes which run AFTER syscalls complete.
By the time they detect an exploit, it's already executed.
[GIF of exploit succeeding]

2/ Nexus Axiom uses LSM hooks instead.
LSM hooks run BEFORE syscalls complete.
They can return error codes to block the syscall.
[Architecture diagram]

3/ Demo: CVE-2021-4034 (PwnKit)
Without Nexus Axiom: Exploit succeeds ❌
With Nexus Axiom: Process killed instantly ✅
[Side-by-side GIF]

4/ Performance:
- <5% CPU overhead
- 2MB memory
- 1M events/sec throughput
Production-ready.

5/ It's open source (GPL-3.0)
⭐ Star it: https://github.com/YOUR_USERNAME/nexus-axiom
Looking for contributors!

What should I add next? 👇
```

**Hashtags:** #eBPF #cybersecurity #Linux #infosec #opensource

---

## Comment Response Templates

### Positive Feedback
```
Thanks! Really appreciate the support. 

Let me know if you hit any issues or have feature requests!
```

### Technical Question
```
Great question! [Answer]

I documented this in [link to docs/code].

Happy to clarify further if needed!
```

### Criticism/Bug Report
```
You're absolutely right - that's a limitation.

I'm tracking it here: [GitHub issue link]

Would love your input on the best approach!
```

### Feature Request
```
That's a great idea! 

I've opened an issue to track it: [link]

Would you be interested in contributing? I can help guide you.
```

### Comparison to Other Tools
```
[Tool X] is great for [use case]. 

Nexus Axiom focuses specifically on blocking exploits at the LSM layer, 
which [Tool X] doesn't do because it uses tracepoints.

They're complementary - you could run both!
```

---

## Influencer Outreach (Week 2)

### Message Template
```
Hi [Name],

I built an eBPF security tool that uses LSM hooks instead of tracepoints 
to actually block exploits (not just log them).

Would love your feedback: https://github.com/YOUR_USERNAME/nexus-axiom

It blocks CVE-2021-4034 and other real exploits with <5% overhead.

Thanks!
[Your Name]
```

### Target List
- [ ] Brendan Gregg (@brendangregg)
- [ ] Liz Rice (LinkedIn)
- [ ] Jessie Frazelle (@jessfraz)
- [ ] Julia Evans (@b0rk)
- [ ] ThePrimeagen (@ThePrimeagen)

---

## Metrics to Track Daily

### GitHub
- Stars: _____
- Forks: _____
- Issues: _____
- PRs: _____
- Traffic: _____ views

### Social
- Twitter followers: _____
- Discord members: _____
- Reddit upvotes: _____

### HackerNews
- Points: _____
- Comments: _____
- Rank: _____

---

## Emergency Responses

### "This doesn't work"
```
Sorry you're hitting issues! Can you share:
- OS and kernel version
- Error message
- Output of: cat /sys/kernel/security/lsm | grep bpf

I'll help debug!
```

### "This is just like [Tool X]"
```
[Tool X] uses tracepoints which run after syscalls complete.
Nexus Axiom uses LSM hooks which run before, so it can actually block.

Here's the technical difference: [link to docs]
```

### "Performance claims are fake"
```
Fair skepticism! Here's how to reproduce the benchmarks:

cd examples
./benchmark.sh

I ran this on [specs]. Would love to see your results!
```

---

## Daily Schedule (First Week)

### Day 1 (Prep)
- [ ] Create Twitter account
- [ ] Set up Discord
- [ ] Record demo GIF
- [ ] Test on fresh VM
- [ ] Get 5 initial stars

### Day 2 (HN Launch)
- [ ] Post to HN at 9-10 AM EST
- [ ] Monitor every 15 minutes
- [ ] Respond to ALL comments
- [ ] Share on Twitter
- [ ] Update Discord

### Day 3 (Reddit)
- [ ] Post to r/netsec (morning)
- [ ] Post to r/linux (afternoon)
- [ ] Post to r/programming (evening)
- [ ] Respond to all comments
- [ ] Fix any reported bugs

### Day 4-5 (Twitter)
- [ ] Post Twitter thread
- [ ] Engage with replies
- [ ] Retweet mentions
- [ ] DM influencers

### Day 6-7 (Community)
- [ ] Post in eBPF Slack
- [ ] Post in CNCF Slack
- [ ] Email newsletters
- [ ] Respond to all issues/PRs

---

## Success Criteria

### Week 1 Target: 500-1000 stars
- HN front page (top 10)
- Reddit r/netsec (top 5)
- 50+ GitHub issues/PRs
- 100+ Discord members

### Week 2 Target: 1500-2500 stars
- Blog post published
- Video demo posted
- 5+ influencer mentions
- 10+ contributors

### Week 3 Target: 3000-5000+ stars
- v1.1 released
- Major announcement
- Conference talk submitted
- Trending #1 on GitHub

---

## Red Flags to Avoid

❌ Overpromising features
❌ Attacking competitors
❌ Ignoring issues/PRs
❌ Arguing with critics
❌ Slow response times

✅ Be honest about limitations
✅ Be respectful of other tools
✅ Respond within 1 hour
✅ Accept criticism gracefully
✅ Fix bugs immediately

---

## Remember

**The first 48 hours are CRITICAL.**

- Be online 24/7
- Respond to EVERY comment
- Fix bugs immediately
- Show gratitude
- Build community

**You got this! 🚀**
