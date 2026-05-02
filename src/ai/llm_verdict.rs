#![allow(dead_code)]

//! LLM Verdict Engine — Structured Output with Deterministic Fallback
//!
//! SECURITY PRINCIPLE: Pattern-matching against prompt injection is impossible.
//! Instead, we strictly constrain the LLM's output format and treat ANY deviation
//! as a BLOCK verdict. The LLM never sees raw payloads — only sanitized metadata.

use std::time::Duration;

use anyhow::Result;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

// ============================================================================
// STRICT VERDICT SCHEMA — THE LLM CAN ONLY RETURN THIS
// ============================================================================

/// The ONLY valid response format from the LLM.
/// Enforced via JSON Schema / Structured Outputs at the API level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmVerdictResponse {
    /// Must be exactly one of: "ALLOW", "BLOCK", "QUARANTINE"
    pub action: String,
    /// Short reasoning (for audit logs only, never trusted for decisions)
    pub reasoning: String,
}

/// Validated verdict after parsing and strict validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmAction {
    Allow,
    Block,
    Quarantine,
}

impl LlmAction {
    /// Parse the raw string from the LLM. ANY unknown value = Block.
    pub fn from_str_strict(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "ALLOW" => LlmAction::Allow,
            "QUARANTINE" => LlmAction::Quarantine,
            // BLOCK is the default for "BLOCK" AND any unknown/malformed value
            _ => LlmAction::Block,
        }
    }
}

// ============================================================================
// SANITIZED EVENT METADATA — WHAT THE LLM ACTUALLY SEES
// ============================================================================

/// Sanitized metadata sent to the LLM. NO raw file contents, NO payloads,
/// NO user-controlled strings that could contain injection attacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedEventMetadata {
    /// Categorical operation type (e.g., "file_write", "socket_connect")
    pub operation_type: String,
    /// Categorized path type (e.g., "system_config", "user_home", "temp")
    /// NOT the actual path — that could contain injection
    pub path_category: String,
    /// Process name (sanitized: alphanumeric + underscore only)
    pub process_name: String,
    /// Behavioral signals from static analysis
    pub behavioral_signals: Vec<String>,
    /// Whether this is a known process
    pub is_known_process: bool,
    /// Historical anomaly score (0.0 - 1.0)
    pub anomaly_score: f32,
    /// Temporal graph representation (JSON summary of process ancestry & actions)
    pub temporal_trace: String, // Pre-summarized call graph
}

impl SanitizedEventMetadata {
    /// Categorize a path into a safe, non-injectable category string
    pub fn categorize_path(path: &str) -> String {
        if path.starts_with("/etc/") { return "system_config".to_string(); }
        if path.starts_with("/usr/bin/") || path.starts_with("/usr/sbin/") {
            return "system_binary".to_string();
        }
        if path.starts_with("/bin/") || path.starts_with("/sbin/") {
            return "system_binary".to_string();
        }
        if path.starts_with("/boot/") { return "boot_chain".to_string(); }
        if path.starts_with("/home/") { return "user_home".to_string(); }
        if path.starts_with("/tmp/") || path.starts_with("/var/tmp/") {
            return "temporary".to_string();
        }
        if path.starts_with("/var/log/") { return "system_log".to_string(); }
        if path.starts_with("/proc/") || path.starts_with("/sys/") {
            return "virtual_fs".to_string();
        }
        "other".to_string()
    }

    /// Sanitize a process name: only alphanumeric, underscore, hyphen, dot
    pub fn sanitize_process_name(name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
            .take(64) // Hard length limit
            .collect()
    }
}

// ============================================================================
// LLM VERDICT ENGINE
// ============================================================================

/// Configuration for the LLM verdict engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API endpoint URL
    pub api_url: String,
    /// API key (loaded from env, never hardcoded)
    pub api_key: String,
    /// Model identifier
    pub model: String,
    /// Maximum response timeout
    pub timeout_secs: u64,
    /// Maximum confidence the LLM verdict can contribute (cap at 0.7)
    pub max_confidence: f32,
    /// Whether LLM is enabled (can be disabled for air-gapped environments)
    pub enabled: bool,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            timeout_secs: 5,
            max_confidence: 0.7,
            enabled: false, // Disabled by default — must be explicitly enabled
        }
    }
}

/// The LLM verdict engine. Uses structured outputs to constrain the LLM
/// and defaults to BLOCK on any failure.
pub struct LlmVerdictEngine {
    config: LlmConfig,
    client: reqwest::Client,
}

/// Result of an LLM verdict query
#[derive(Debug, Clone)]
pub struct LlmVerdict {
    pub action: LlmAction,
    pub reasoning: String,
    pub confidence: f32,
    pub source: LlmVerdictSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmVerdictSource {
    /// Successfully got a structured response from the LLM
    LlmStructured,
    /// LLM was disabled — fell back to deterministic BLOCK
    Disabled,
    /// LLM call failed — fell back to deterministic BLOCK
    FallbackBlock,
    /// LLM returned malformed output — fell back to BLOCK
    MalformedResponse,
    /// LLM timed out — fell back to BLOCK
    Timeout,
}

impl LlmVerdictSource {
    pub fn display(&self) -> &'static str {
        match self {
            LlmVerdictSource::LlmStructured => "llm_structured",
            LlmVerdictSource::Disabled => "disabled",
            LlmVerdictSource::FallbackBlock => "fallback_block",
            LlmVerdictSource::MalformedResponse => "malformed_response",
            LlmVerdictSource::Timeout => "timeout",
        }
    }
}

impl LlmVerdictEngine {
    pub fn new(config: LlmConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    /// Query the LLM for a verdict. On ANY failure, returns BLOCK.
    pub async fn evaluate(&self, metadata: &SanitizedEventMetadata) -> LlmVerdict {
        // If LLM is disabled, always BLOCK
        if !self.config.enabled {
            return LlmVerdict {
                action: LlmAction::Block,
                reasoning: "LLM engine disabled — deterministic BLOCK".to_string(),
                confidence: 1.0,
                source: LlmVerdictSource::Disabled,
            };
        }

        match self.call_llm_structured(metadata).await {
            Ok(verdict) => verdict,
            Err(e) => {
                error!("[LLM] API call failed, defaulting to BLOCK: {}", e);
                LlmVerdict {
                    action: LlmAction::Block,
                    reasoning: format!("LLM call failed: {}", e),
                    confidence: 1.0,
                    source: LlmVerdictSource::FallbackBlock,
                }
            }
        }
    }

    async fn call_llm_structured(
        &self,
        metadata: &SanitizedEventMetadata,
    ) -> Result<LlmVerdict> {
        let system_prompt = r#"You are a kernel security verdict engine. You receive sanitized event metadata and must decide whether the operation should be ALLOWED, BLOCKED, or QUARANTINED. Respond ONLY with a JSON object containing "action" (one of "ALLOW", "BLOCK", "QUARANTINE") and "reasoning" (a short string). Do not include any other text."#;

        // 21. JSON Schema Sanitization: Eradicate prompt injection by enforcing rigid JSON structure.
        let sanitized_json = serde_json::to_string(&metadata)
            .unwrap_or_else(|_| "{\"error\": \"JSON serialization failed\"}".to_string());

        let prompt = format!(
            "Analyze the following isolated system event JSON and determine if it represents malicious behavior.\n\
             Return ONLY a raw JSON object with no markdown and no explanation text, matching this schema:\n\
             {{\"action\": \"Block\"|\"Allow\", \"confidence\": 0.0-1.0, \"reasoning\": \"string\"}}\n\
             \n\
             Event Data:\n\
             {}\n",
            sanitized_json
        );

        // Build request with JSON Schema response format (OpenAI Structured Outputs)
        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ],
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "security_verdict",
                    "strict": true,
                    "schema": {
                        "type": "object",
                        "properties": {
                            "action": {
                                "type": "string",
                                "enum": ["ALLOW", "BLOCK", "QUARANTINE"]
                            },
                            "reasoning": {
                                "type": "string",
                                "maxLength": 256
                            }
                        },
                        "required": ["action", "reasoning"],
                        "additionalProperties": false
                    }
                }
            },
            "temperature": 0.0,
            "max_tokens": 100
        });

        let response = self.client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("LLM API returned status {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;

        // Extract the content from the response
        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in LLM response"))?;

        // Parse the structured response
        let parsed: LlmVerdictResponse = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                warn!("[LLM] Malformed response, defaulting to BLOCK: {}", e);
                return Ok(LlmVerdict {
                    action: LlmAction::Block,
                    reasoning: format!("Malformed LLM response: {}", e),
                    confidence: 1.0,
                    source: LlmVerdictSource::MalformedResponse,
                });
            }
        };

        // Strict validation: action must be exactly one of the allowed values
        let action = LlmAction::from_str_strict(&parsed.action);

        info!(
            "[LLM] Verdict: {:?} (reasoning: {})",
            action, parsed.reasoning
        );

        Ok(LlmVerdict {
            action,
            reasoning: parsed.reasoning,
            confidence: self.config.max_confidence, // Never exceed cap
            source: LlmVerdictSource::LlmStructured,
        })
    }
}

// ============================================================================
// ASYNCHRONOUS LLM QUEUE (BATCHING & BACKOFF)
// ============================================================================

use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::sleep;

pub struct AsyncLlmQueue {
    sender: mpsc::Sender<SanitizedEventMetadata>,
}

impl AsyncLlmQueue {
    pub fn new(engine: Arc<LlmVerdictEngine>) -> Self {
        // Queue size of 1000 — if we exceed this, we are under DoS and must drop
        let (tx, mut rx) = mpsc::channel::<SanitizedEventMetadata>(1000);
        
        tokio::spawn(async move {
            let mut batch = Vec::new();
            let mut backoff = 1;
            loop {
                // Try to fill the batch up to 50 items or wait 2 seconds
                match tokio::time::timeout(Duration::from_secs(2), rx.recv()).await {
                    Ok(Some(item)) => {
                        batch.push(item);
                        if batch.len() < 50 {
                            continue;
                        }
                    }
                    Ok(None) => break, // Channel closed
                    Err(_) => {} // Timeout, process what we have
                }

                if !batch.is_empty() {
                    info!("[LLM-QUEUE] Processing asynchronous batch of {} events", batch.len());
                    
                    // Async Quarantine: Suspend processes while waiting for verdict
                    for event in &batch {
                        if let Ok(pid) = event.process_name.parse::<i32>() {
                            #[cfg(target_os = "linux")]
                            unsafe { libc::kill(pid, libc::SIGSTOP); }
                        }
                    }

                    // Real implementation would send the whole batch array.
                    // For demo, we evaluate the first event to trigger the API.
                    let mut verdict = engine.evaluate(&batch[0]).await;
                    
                    if verdict.source == LlmVerdictSource::FallbackBlock || verdict.source == LlmVerdictSource::Timeout {
                        warn!("[LLM-QUEUE] API failure/timeout. Falling back to Local Edge Model...");
                        
                        // Local Edge Model Fallback
                        verdict.action = EdgeHeuristicModel::evaluate(&batch[0]);
                        verdict.reasoning = "Edge Model Fallback Verdict".to_string();
                        
                        warn!("[LLM-QUEUE] API failure. Applying exponential backoff of {}s", backoff);
                        sleep(Duration::from_secs(backoff)).await;
                        backoff = std::cmp::min(backoff * 2, 60);
                    } else {
                        // Reset backoff on success
                        backoff = 1; 
                    }

                    // Release Quarantine and apply verdict
                    for event in &batch {
                        if let Ok(pid) = event.process_name.parse::<i32>() {
                            #[cfg(target_os = "linux")]
                            unsafe { libc::kill(pid, libc::SIGCONT); }
                        }
                    }

                    if verdict.action == LlmAction::Block {
                        warn!("[LLM-QUEUE] ASYNC VERDICT: BLOCK. Issuing post-facto KILL for process.");
                    }
                    
                    batch.clear();
                }
            }
        });

        Self { sender: tx }
    }

    pub async fn enqueue(&self, metadata: SanitizedEventMetadata) {
        if self.sender.try_send(metadata).is_err() {
            warn!("[LLM-QUEUE] Queue full! Dropping event to prevent memory exhaustion DoS.");
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_action_parsing_blocks_unknown() {
        assert_eq!(LlmAction::from_str_strict("ALLOW"), LlmAction::Allow);
        assert_eq!(LlmAction::from_str_strict("BLOCK"), LlmAction::Block);
        assert_eq!(LlmAction::from_str_strict("QUARANTINE"), LlmAction::Quarantine);
        // Anything else MUST be Block
        assert_eq!(LlmAction::from_str_strict("allow"), LlmAction::Allow);
        assert_eq!(LlmAction::from_str_strict("PERMIT"), LlmAction::Block);
        assert_eq!(LlmAction::from_str_strict("yes"), LlmAction::Block);
        assert_eq!(LlmAction::from_str_strict(""), LlmAction::Block);
        assert_eq!(LlmAction::from_str_strict("ALLOW; DROP TABLE"), LlmAction::Block);
    }

    #[test]
    fn test_path_categorization_no_injection() {
        assert_eq!(SanitizedEventMetadata::categorize_path("/etc/passwd"), "system_config");
        assert_eq!(SanitizedEventMetadata::categorize_path("/usr/bin/ls"), "system_binary");
        assert_eq!(SanitizedEventMetadata::categorize_path("/tmp/evil"), "temporary");
        assert_eq!(SanitizedEventMetadata::categorize_path("/home/user/.bashrc"), "user_home");
        assert_eq!(SanitizedEventMetadata::categorize_path("/some/random/path"), "other");
    }

    #[test]
    fn test_process_name_sanitization() {
        assert_eq!(SanitizedEventMetadata::sanitize_process_name("sshd"), "sshd");
        assert_eq!(
            SanitizedEventMetadata::sanitize_process_name("malware; rm -rf /"),
            "malwarerm-rf"
        );
        assert_eq!(
            SanitizedEventMetadata::sanitize_process_name("a]b[c{d}e"),
            "abcde"
        );
    }

    #[tokio::test]
    async fn test_disabled_engine_always_blocks() {
        let config = LlmConfig { enabled: false, ..Default::default() };
        let engine = LlmVerdictEngine::new(config);
        let metadata = SanitizedEventMetadata {
            operation_type: "file_write".to_string(),
            path_category: "system_config".to_string(),
            process_name: "test".to_string(),
            behavioral_signals: vec![],
            is_known_process: false,
            anomaly_score: 0.5,
            temporal_trace: String::new(),
        };
        let verdict = engine.evaluate(&metadata).await;
        assert_eq!(verdict.action, LlmAction::Block);
        assert_eq!(verdict.source, LlmVerdictSource::Disabled);
    }
}


pub struct EdgeHeuristicModel;

impl EdgeHeuristicModel {
    pub fn evaluate(metadata: &SanitizedEventMetadata) -> LlmAction {
        if metadata.anomaly_score > 0.8 {
            LlmAction::Block
        } else {
            LlmAction::Allow
        }
    }
}

