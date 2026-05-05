# AI PROMPTS FOR LAUNCH - COPY/PASTE TO CLAUDE OR CHATGPT

## 1. HACKERNEWS POST

**Prompt:**
```
Write a HackerNews "Show HN" post for an eBPF security tool called Nexus Axiom.

Key points:
- Uses LSM (Linux Security Module) hooks to block exploits BEFORE execution
- Unlike Falco/Tetragon which use tracepoints that fire AFTER syscall completes
- Tested with PwnKit (CVE-2021-4034), Dirty Pipe (CVE-2022-0847), and 10+ other CVEs
- Written in Rust + eBPF
- Open source (GPL-3.0)
- Honest about limitations (Linux 5.8+, needs lsm=bpf, ~1μs overhead)

Tone: Technical but accessible, humble, seeking feedback
Length: 250-300 words
Audience: Security engineers and eBPF developers

Include:
- Hook: Why existing tools can't actually stop exploits
- Problem: Tracepoints fire too late
- Solution: LSM hooks run in security decision path
- Demo: Link to GitHub
- Ask: "What am I missing?" or "Looking for feedback"

GitHub: https://github.com/CoderAwesomeAbhi/nexus-axiom
```

---

## 2. REDDIT POST (r/netsec)

**Prompt:**
```
Write a Reddit post for r/netsec about an eBPF security tool.

Title format: "[Tool] Brief description"

Key points:
- eBPF security tool using LSM hooks (not tracepoints)
- Actually blocks exploits before execution
- Tested with real CVEs (PwnKit, Dirty Pipe)
- Open source, honest about limitations

Tone: Technical, matter-of-fact, not promotional
Length: 150-200 words
Include: Link to GitHub and VERIFICATION.md

End with: "Feedback welcome from security engineers"
```

---

## 3. REDDIT POST (r/rust)

**Prompt:**
```
Write a Reddit post for r/rust about an eBPF security tool written in Rust.

Title format: Focus on Rust + eBPF combination

Key points:
- Rust userspace + eBPF kernel code
- Uses libbpf-rs for eBPF interaction
- LSM hooks for exploit prevention
- Real-world CVE testing

Tone: Technical, focus on Rust aspects
Length: 150-200 words
Audience: Rust developers interested in systems programming

Include: Interesting Rust/eBPF integration challenges
```

---

## 4. TWITTER THREAD (6 tweets)

**Prompt:**
```
Write a 6-tweet Twitter thread about an eBPF security tool.

Tweet 1: Hook - "Most eBPF security tools watch attacks happen. This one stops them."
Tweet 2: Problem - Why tracepoints can't block exploits
Tweet 3: Solution - How LSM hooks work differently
Tweet 4: Proof - CVEs tested (PwnKit, Dirty Pipe, etc.)
Tweet 5: Honest limitations
Tweet 6: Call to action - Star on GitHub

Tone: Technical but engaging, use emojis sparingly
Each tweet: 240-280 characters
Include relevant hashtags: #eBPF #security #rust #linux

Make it tweetable - people should want to retweet.
```

---

## 5. INFLUENCER DM (Twitter)

**Prompt:**
```
Write a short Twitter DM to an eBPF expert (like Brendan Gregg or Liz Rice).

Key points:
- Built eBPF security tool using LSM hooks
- Different from existing tools (blocks vs observes)
- Asking for feedback, not promotion
- Include demo link

Tone: Humble, respectful of their time, specific
Length: 2-3 sentences max
Make it easy to say yes or no

End with: "No pressure if you're busy!"
```

---

## 6. VALIDATION BOUNTY POST (Reddit)

**Prompt:**
```
Write a Reddit post offering $100 bounty for independent validation.

Title: "[Bounty] $100 for independent validation of eBPF security tool"

Key points:
- Offering $100 to first 3 security researchers
- Must independently test and post public report
- Tool claims to block W^X exploits using LSM hooks
- Requirements: Ubuntu 22.04 VM, test provided exploits, honest report
- First 3 get paid

Tone: Professional, clear requirements, emphasize honesty
Length: 200-250 words
Include: How to claim, what to test, payment method

Make it clear: Report failures too - honesty is valued.
```

---

## 7. GITHUB ISSUE RESPONSE TEMPLATE

**Prompt:**
```
Write a template for responding to GitHub issues on an eBPF security tool.

Scenarios:
1. Bug report
2. Feature request
3. Question about how it works
4. Comparison to other tools

Tone: Helpful, technical, honest
Include: Thanks for reporting, ask for details, timeline for fix

Make it friendly but professional.
```

---

## 8. HACKERNEWS COMMENT RESPONSES

**Prompt:**
```
Write 5 template responses for common HackerNews comments:

1. "How is this different from Falco?"
2. "What's the performance overhead?"
3. "Does this work with Docker/Kubernetes?"
4. "Can this be bypassed?"
5. "Why not use SELinux/AppArmor?"

Tone: Technical, honest, not defensive
Length: 2-3 sentences each
Include: Acknowledge limitations, provide specifics

Make responses that invite further discussion.
```

---

## 9. PRODUCTHUNT LAUNCH

**Prompt:**
```
Write a ProductHunt launch post for an eBPF security tool.

Tagline: (10 words max, catchy)
Description: (200 words, focus on value prop)
First comment: (Introduce yourself, why you built it)

Key points:
- Blocks exploits before execution (unique value)
- Tested with real CVEs
- Open source
- For Linux servers/containers

Tone: Accessible to non-technical audience
Include: Demo link, GitHub link

Make it ProductHunt-friendly (less technical than HN).
```

---

## 10. LINKEDIN POST

**Prompt:**
```
Write a LinkedIn post about launching an open-source eBPF security tool.

Key points:
- Built to solve real problem (eBPF tools that only observe)
- Technical achievement (LSM hooks)
- Open source contribution
- Looking for feedback from security community

Tone: Professional, achievement-focused, humble
Length: 150-200 words
Include: What you learned, call for collaboration

Make it LinkedIn-appropriate (career/learning angle).
```

---

## HOW TO USE THESE PROMPTS

1. **Copy each prompt** to Claude or ChatGPT
2. **Review the output** - make sure it sounds like you
3. **Customize** with your personal voice
4. **Use immediately** - don't overthink it

**Time to generate all content: 30 minutes**

**Then you just need to:**
- Post to HackerNews (5 min)
- Post to Reddit (10 min)
- Tweet thread (5 min)
- DM influencers (15 min)
- Respond to comments (1-2 hours)

**Total time: 2-3 hours + AI does the writing**

---

## BONUS: EMAIL TO SECURITY NEWSLETTERS

**Prompt:**
```
Write a pitch email to security newsletter editors (e.g., tl;dr sec, Unsupervised Learning).

Subject: Story idea - eBPF security tool that actually blocks exploits

Key points:
- Most eBPF security tools can only observe, not prevent
- Built tool using LSM hooks to block before execution
- Tested with real CVEs
- Getting traction on HackerNews (mention stars if >500)

Tone: Professional pitch, newsworthy angle
Length: 150 words max
Include: Why their audience would care

Make it easy for them to say yes.
```

---

**COPY THESE PROMPTS → PASTE TO AI → GET ALL CONTENT IN 30 MINUTES**
