// Input validation for security
use anyhow::{Result, bail};
use regex::Regex;

pub struct Validator;

impl Validator {
    /// Validate email address
    pub fn validate_email(email: &str) -> Result<()> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        
        if email.len() > 254 {
            bail!("Email too long (max 254 characters)");
        }
        
        if !email_regex.is_match(email) {
            bail!("Invalid email format");
        }
        
        Ok(())
    }
    
    /// Validate user ID (alphanumeric + hyphens only)
    pub fn validate_user_id(id: &str) -> Result<()> {
        if id.is_empty() {
            bail!("User ID cannot be empty");
        }
        
        if id.len() > 64 {
            bail!("User ID too long (max 64 characters)");
        }
        
        let id_regex = Regex::new(r"^[a-zA-Z0-9-]+$").unwrap();
        if !id_regex.is_match(id) {
            bail!("User ID can only contain alphanumeric characters and hyphens");
        }
        
        Ok(())
    }
    
    /// Validate tenant ID
    pub fn validate_tenant_id(id: &str) -> Result<()> {
        Self::validate_user_id(id) // Same rules
    }
    
    /// Validate role name
    pub fn validate_role_name(name: &str) -> Result<()> {
        if name.is_empty() {
            bail!("Role name cannot be empty");
        }
        
        if name.len() > 32 {
            bail!("Role name too long (max 32 characters)");
        }
        
        let name_regex = Regex::new(r"^[a-z_]+$").unwrap();
        if !name_regex.is_match(name) {
            bail!("Role name can only contain lowercase letters and underscores");
        }
        
        Ok(())
    }
    
    /// Validate webhook URL
    pub fn validate_webhook_url(url: &str) -> Result<()> {
        if url.is_empty() {
            bail!("Webhook URL cannot be empty");
        }
        
        if !url.starts_with("https://") {
            bail!("Webhook URL must use HTTPS");
        }
        
        if url.len() > 2048 {
            bail!("Webhook URL too long (max 2048 characters)");
        }
        
        Ok(())
    }
    
    /// Validate API key (not empty, reasonable length)
    pub fn validate_api_key(key: &str) -> Result<()> {
        if key.is_empty() {
            bail!("API key cannot be empty");
        }
        
        if key.len() < 16 {
            bail!("API key too short (min 16 characters)");
        }
        
        if key.len() > 256 {
            bail!("API key too long (max 256 characters)");
        }
        
        Ok(())
    }
    
    /// Sanitize string for logging (remove control characters)
    pub fn sanitize_for_log(input: &str) -> String {
        input.chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .take(1000) // Limit length
            .collect()
    }
    
    /// Validate process name (for allowlist)
    pub fn validate_process_name(name: &str) -> Result<()> {
        if name.is_empty() {
            bail!("Process name cannot be empty");
        }
        
        if name.len() > 255 {
            bail!("Process name too long (max 255 characters)");
        }
        
        // Allow alphanumeric, hyphens, underscores, dots
        let name_regex = Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap();
        if !name_regex.is_match(name) {
            bail!("Process name contains invalid characters");
        }
        
        Ok(())
    }
    
    /// Validate PID
    pub fn validate_pid(pid: u32) -> Result<()> {
        if pid == 0 {
            bail!("PID cannot be 0");
        }
        
        if pid > 4194304 { // Linux max PID
            bail!("PID too large");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_email() {
        assert!(Validator::validate_email("test@example.com").is_ok());
        assert!(Validator::validate_email("user+tag@domain.co.uk").is_ok());
        assert!(Validator::validate_email("invalid").is_err());
        assert!(Validator::validate_email("@example.com").is_err());
        assert!(Validator::validate_email("test@").is_err());
    }
    
    #[test]
    fn test_validate_user_id() {
        assert!(Validator::validate_user_id("user-123").is_ok());
        assert!(Validator::validate_user_id("abc123").is_ok());
        assert!(Validator::validate_user_id("").is_err());
        assert!(Validator::validate_user_id("user@123").is_err());
        assert!(Validator::validate_user_id("a".repeat(65).as_str()).is_err());
    }
    
    #[test]
    fn test_validate_webhook_url() {
        assert!(Validator::validate_webhook_url("https://hooks.slack.com/services/xxx").is_ok());
        assert!(Validator::validate_webhook_url("http://example.com").is_err()); // Not HTTPS
        assert!(Validator::validate_webhook_url("").is_err());
    }
    
    #[test]
    fn test_sanitize_for_log() {
        let input = "test\x00\x01\x02normal";
        let sanitized = Validator::sanitize_for_log(input);
        assert!(!sanitized.contains('\x00'));
        assert!(sanitized.contains("normal"));
    }
}
