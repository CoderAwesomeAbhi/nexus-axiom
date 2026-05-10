// Policy DSL: Human-readable security policies
// Example: workload:web can mmap wx=deny uid:1000 can exec allow

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub selector: Selector,
    pub action: Action,
    pub operation: Operation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Selector {
    Workload(String),
    Uid(u32),
    Cgroup(u64),
    Path(String),
    All,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    MmapWX,
    MprotectWX,
    Exec,
    Ptrace,
    FileWrite(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Allow,
    Deny,
    Audit,
}

impl Policy {
    pub fn parse(input: &str) -> Result<Self> {
        let mut rules = Vec::new();
        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            rules.push(Rule::parse(line)?);
        }
        Ok(Policy { rules })
    }

    pub fn evaluate(&self, selector: &Selector, op: &Operation) -> Action {
        for rule in &self.rules {
            if rule.matches(selector, op) {
                return rule.action.clone();
            }
        }
        Action::Deny // Default deny
    }
}

impl Rule {
    fn parse(line: &str) -> Result<Self> {
        // Parse: "workload:web can mmap wx=deny"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            bail!("Invalid rule: {}", line);
        }

        let selector = Selector::parse(parts[0])?;
        let operation = Operation::parse(parts[2])?;
        let action = Action::parse(parts[3])?;

        Ok(Rule { selector, action, operation })
    }

    fn matches(&self, sel: &Selector, op: &Operation) -> bool {
        let selector_match = match (&self.selector, sel) {
            (Selector::Workload(a), Selector::Workload(b)) => a == b,
            (Selector::Uid(a), Selector::Uid(b)) => a == b,
            (Selector::Cgroup(a), Selector::Cgroup(b)) => a == b,
            (Selector::Path(a), Selector::Path(b)) => a == b,
            (Selector::All, _) => true,
            _ => false,
        };

        let op_match = match (&self.operation, op) {
            (Operation::MmapWX, Operation::MmapWX) => true,
            (Operation::MprotectWX, Operation::MprotectWX) => true,
            (Operation::Exec, Operation::Exec) => true,
            (Operation::Ptrace, Operation::Ptrace) => true,
            (Operation::FileWrite(a), Operation::FileWrite(b)) => a == b,
            _ => false,
        };

        selector_match && op_match
    }
}

impl Selector {
    fn parse(s: &str) -> Result<Self> {
        if let Some(w) = s.strip_prefix("workload:") {
            Ok(Selector::Workload(w.to_string()))
        } else if let Some(u) = s.strip_prefix("uid:") {
            Ok(Selector::Uid(u.parse()?))
        } else if s == "all" {
            Ok(Selector::All)
        } else {
            bail!("Unknown selector: {}", s)
        }
    }
}

impl Operation {
    fn parse(s: &str) -> Result<Self> {
        match s {
            "mmap" => Ok(Operation::MmapWX),
            "mprotect" => Ok(Operation::MprotectWX),
            "exec" => Ok(Operation::Exec),
            "ptrace" => Ok(Operation::Ptrace),
            _ => bail!("Unknown operation: {}", s),
        }
    }
}

impl Action {
    fn parse(s: &str) -> Result<Self> {
        let s = s.trim_start_matches("wx=");
        match s {
            "allow" => Ok(Action::Allow),
            "deny" => Ok(Action::Deny),
            "audit" => Ok(Action::Audit),
            _ => bail!("Unknown action: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_policy() {
        let input = "workload:web can mmap wx=deny\nuid:1000 can exec allow";
        let policy = Policy::parse(input).unwrap();
        assert_eq!(policy.rules.len(), 2);
    }

    #[test]
    fn test_evaluate_deny() {
        let input = "workload:web can mmap wx=deny";
        let policy = Policy::parse(input).unwrap();
        let action = policy.evaluate(
            &Selector::Workload("web".to_string()),
            &Operation::MmapWX,
        );
        assert!(matches!(action, Action::Deny));
    }

    #[test]
    fn test_evaluate_allow() {
        let input = "uid:1000 can exec allow";
        let policy = Policy::parse(input).unwrap();
        let action = policy.evaluate(
            &Selector::Uid(1000),
            &Operation::Exec,
        );
        assert!(matches!(action, Action::Allow));
    }

    #[test]
    fn test_evaluate_default_deny() {
        let policy = Policy { rules: Vec::new() };
        let action = policy.evaluate(
            &Selector::Uid(9999),
            &Operation::MmapWX,
        );
        assert!(matches!(action, Action::Deny));
    }

    #[test]
    fn test_wildcard_selector() {
        let input = "all can ptrace audit";
        let policy = Policy::parse(input).unwrap();
        let action = policy.evaluate(
            &Selector::Uid(42),
            &Operation::Ptrace,
        );
        assert!(matches!(action, Action::Audit));
    }
}
