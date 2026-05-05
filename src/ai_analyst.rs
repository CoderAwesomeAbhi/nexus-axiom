use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

/// AI Analyst uses a local or remote LLM to generate plain-English
/// threat analysis reports for blocked processes.
#[derive(Clone)]
pub struct AIAnalyst {
    api_key: Option<String>,
    endpoint: String,
    client: Client,
}

impl AIAnalyst {
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let api_key = api_key
            .or_else(|| std::env::var("NEXUS_AXIOM_OPENAI_API_KEY").ok())
            .or_else(|| std::env::var("OPENAI_API_KEY").ok());

        let endpoint = std::env::var("NEXUS_AXIOM_AI_ENDPOINT")
            .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .context("failed to build AI HTTP client")?;

        Ok(Self {
            api_key,
            endpoint,
            client,
        })
    }

    /// Queries the AI to analyze a blocked threat. Uses blocking reqwest for simplicity in the event loop.
    pub fn analyze_threat(&self, pid: u32, comm: &str, reason: &str) -> Result<String> {
        log::info!(
            "🧠 Initiating AI Threat Analysis for {} (PID: {})",
            comm,
            pid
        );

        let prompt = format!(
            "You are an expert SOC analyst. Analyze this blocked execution event:\n\
            Process: {}\n\
            PID: {}\n\
            Reason: {}\n\
            Write a 2-sentence executive summary of the likely threat and recommended next steps.",
            comm, pid, reason
        );

        // If no key is configured, use rule-based analysis
        let Some(api_key) = self.api_key.as_ref() else {
            return Ok(self.rule_based_analysis(comm, reason));
        };

        let body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {"role": "system", "content": "You are a cybersecurity expert."},
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 100
        });

        let resp = self
            .client
            .post(&self.endpoint)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .context("AI request failed")?;

        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("AI analysis failed: HTTP {}", status);
        }

        let json: serde_json::Value = resp.json().context("failed to parse AI response payload")?;
        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .context("AI response missing choices[0].message.content")?;
        Ok(text.trim().to_string())
    }

    /// Rule-based threat analysis when no API key is available
    fn rule_based_analysis(&self, comm: &str, reason: &str) -> String {
        let threat_level = if comm.contains("exploit") || comm.contains("pwn") {
            "CRITICAL"
        } else if comm.contains("sh") || comm.contains("bash") {
            "HIGH"
        } else {
            "MEDIUM"
        };

        let analysis = match reason {
            r if r.contains("W^X") => {
                format!(
                    "[{}] Process '{}' attempted W^X memory allocation - classic shellcode injection pattern. \
                    This is used by exploits to write malicious code and execute it. \
                    Recommendation: Investigate binary origin, check for CVE matches, isolate host if suspicious.",
                    threat_level, comm
                )
            }
            r if r.contains("mprotect") => {
                format!(
                    "[{}] Process '{}' attempted to change memory protection to executable. \
                    Common in JIT spraying and ROP chain attacks. \
                    Recommendation: Analyze memory dumps, check for known exploit signatures.",
                    threat_level, comm
                )
            }
            _ => {
                format!(
                    "[{}] Process '{}' triggered security policy violation: {}. \
                    Recommendation: Review process behavior and system logs.",
                    threat_level, comm, reason
                )
            }
        };

        format!("Rule-Based Analysis: {}", analysis)
    }
}
