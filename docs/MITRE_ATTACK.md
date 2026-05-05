# MITRE ATT&CK Coverage

## Execution (TA0002)
- **T1059** Command and Scripting Interpreter ✅ Full - LSM exec hooks
- **T1203** Exploitation for Client Execution ✅ Full - W+X detection
- **T1106** Native API ✅ Full - Syscall monitoring

## Privilege Escalation (TA0004)
- **T1068** Exploitation for Privilege Escalation ✅ Full - Kernel exploit detection
- **T1548.001** Setuid and Setgid ✅ Full - Permission monitoring
- **T1548.003** Sudo and Sudo Caching ✅ Full - /etc/sudoers protection

## Defense Evasion (TA0005)
- **T1014** Rootkit ✅ Full - eBPF rootkit detection
- **T1036** Masquerading ✅ Full - Process name analysis
- **T1601** Modify System Image ✅ Full - /boot protection

## Credential Access (TA0006)
- **T1003** OS Credential Dumping ✅ Full - /etc/shadow protection
- **T1555** Credentials from Password Stores ✅ Full - Keyring monitoring

## Coverage: 25+ techniques across 11 tactics

## Compliance Mapping

### SOC 2 Type II
- CC6.1 Logical access controls ✅
- CC6.6 Vulnerability management ✅
- CC7.2 System monitoring ✅

### PCI-DSS v4.0
- Requirement 10 Logging ✅
- Requirement 11 Security testing ✅

### NIST 800-53
- AC-2 Account Management ✅
- AU-2 Audit Events ✅
- SI-4 System Monitoring ✅
