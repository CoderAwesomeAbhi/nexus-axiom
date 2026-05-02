use anyhow::Result;
use serde_json::json;
use std::time::Duration;

/// AI Analyst uses a local or remote LLM to generate plain-English
/// threat analysis reports for blocked processes.
pub struct AIAnalyst {
    api_key: String,
    endpoint: String,
}

impl AIAnalyst {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            api_key: api_key.unwrap_or_else(|| "mock_key".to_string()),
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(), // Can be overridden for local Ollama
        }
    }

    /// Queries the AI to analyze a blocked threat. Uses blocking reqwest for simplicity in the event loop.
    pub fn analyze_threat(&self, pid: u32, comm: &str, reason: &str) -> Result<String> {
        log::info!("🧠 Initiating AI Threat Analysis for {} (PID: {})", comm, pid);
        
        let prompt = format!(
            "You are an expert SOC analyst. Analyze this blocked execution event:\n\
            Process: {}\n\
            PID: {}\n\
            Reason: {}\n\
            Write a 2-sentence executive summary of the likely threat and recommended next steps.",
            comm, pid, reason
        );

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        // If it's a mock key, we simulate the AI response to prevent crashing if the user hasn't configured an API key.
        if self.api_key == "mock_key" {
            std::thread::sleep(Duration::from_secs(1));
            return Ok(format!(
                "AI Analysis (Simulated): The process '{}' attempted a highly suspicious W^X memory allocation indicative of shellcode injection or an unpacking routine. Recommendation: Isolate the host and investigate the origin of the binary.",
                comm
            ));
        }

        let body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {"role": "system", "content": "You are a cybersecurity expert."},
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 100
        });

        let resp = client.post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()?;

        if resp.status().is_success() {
            let json: serde_json::Value = resp.json()?;
            if let Some(text) = json["choices"][0]["message"]["content"].as_str() {
                Ok(text.trim().to_string())
            } else {
                Ok("AI Response format error".to_string())
            }
        } else {
            Ok(format!("AI Analysis failed: HTTP {}", resp.status()))
        }
    }
}
