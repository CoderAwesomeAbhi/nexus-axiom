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

        // If no key is configured, return fast deterministic output.
        let Some(api_key) = self.api_key.as_ref() else {
            return Ok(format!(
                "AI Analysis (Simulated): The process '{}' attempted a highly suspicious W^X memory allocation indicative of shellcode injection or an unpacking routine. Recommendation: Isolate the host and investigate the origin of the binary.",
                comm
            ));
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
}
