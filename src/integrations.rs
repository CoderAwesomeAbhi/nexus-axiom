use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ── Configuration types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub slack: Option<SlackConfig>,
    pub pagerduty: Option<PagerDutyConfig>,
    pub datadog: Option<DatadogConfig>,
    pub splunk: Option<SplunkConfig>,
    pub siem: Option<SIEMConfig>,
    pub syslog: Option<SyslogConfig>,
    pub webhook: Option<WebhookConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub webhook_url: String,
    pub channel: String,
    pub severity_threshold: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyConfig {
    pub integration_key: String,
    pub severity_threshold: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatadogConfig {
    pub api_key: String,
    pub app_key: String,
    pub site: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplunkConfig {
    pub hec_url: String,
    pub hec_token: String,
    pub index: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SIEMConfig {
    pub endpoint: String,
    pub api_key: String,
    pub format: String, // CEF, LEEF, JSON, OCSF
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyslogConfig {
    pub host: String,
    pub port: u16,
    pub protocol: String, // "udp" or "tcp"
    pub facility: u8,     // syslog facility (1=user, 4=auth, 10=security)
    pub rfc: String,      // "3164" or "5424"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub method: String, // POST, PUT
    pub headers: std::collections::HashMap<String, String>,
    pub template: Option<String>, // optional Handlebars-style template
}

// ── Alert payload ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct AlertPayload {
    pub severity: String,
    pub title: String,
    pub description: String,
    pub timestamp: u64,
    pub source: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mitre_technique: Option<String>,
}

// ── Circuit breaker ──────────────────────────────────────────────────────────

struct CircuitBreaker {
    failure_count: AtomicU32,
    is_open: AtomicBool,
    threshold: u32,
}

impl CircuitBreaker {
    fn new(threshold: u32) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            is_open: AtomicBool::new(false),
            threshold,
        }
    }

    fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.is_open.store(false, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= self.threshold {
            self.is_open.store(true, Ordering::Relaxed);
            log::warn!("⚠️  Circuit breaker OPEN after {} failures", count);
        }
    }

    fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Relaxed)
    }

    fn reset(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.is_open.store(false, Ordering::Relaxed);
    }
}

// ── Integration manager ──────────────────────────────────────────────────────

pub struct IntegrationManager {
    config: IntegrationConfig,
    client: reqwest::Client,
    breakers: IntegrationBreakers,
    last_sent: Mutex<std::collections::HashMap<String, std::time::Instant>>,
}

struct IntegrationBreakers {
    slack: Arc<CircuitBreaker>,
    pagerduty: Arc<CircuitBreaker>,
    datadog: Arc<CircuitBreaker>,
    splunk: Arc<CircuitBreaker>,
    siem: Arc<CircuitBreaker>,
    syslog: Arc<CircuitBreaker>,
    webhook: Arc<CircuitBreaker>,
}

impl IntegrationBreakers {
    fn new() -> Self {
        Self {
            slack: Arc::new(CircuitBreaker::new(5)),
            pagerduty: Arc::new(CircuitBreaker::new(5)),
            datadog: Arc::new(CircuitBreaker::new(5)),
            splunk: Arc::new(CircuitBreaker::new(5)),
            siem: Arc::new(CircuitBreaker::new(5)),
            syslog: Arc::new(CircuitBreaker::new(10)),
            webhook: Arc::new(CircuitBreaker::new(5)),
        }
    }
}

impl IntegrationManager {
    pub fn new(config: IntegrationConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config,
            client,
            breakers: IntegrationBreakers::new(),
            last_sent: Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Check if we are allowed to send to a specific integration based on rate limits (1 alert / 5 seconds)
    fn check_rate_limit(&self, integration: &str) -> bool {
        let mut last_sent = self.last_sent.lock().unwrap();
        let now = std::time::Instant::now();
        if let Some(last) = last_sent.get(integration) {
            if now.duration_since(*last) < Duration::from_secs(5) {
                log::debug!("Rate limiting {} alert", integration);
                return false;
            }
        }
        last_sent.insert(integration.to_string(), now);
        true
    }

    /// Send alert to all configured integrations in parallel.
    pub async fn send_alert(&self, alert: AlertPayload) -> Result<()> {
        let mut tasks = Vec::new();

        if let Some(slack) = &self.config.slack {
            if !self.breakers.slack.is_open() && self.check_rate_limit("slack") {
                let slack = slack.clone();
                let alert = alert.clone();
                let client = self.client.clone();
                let breaker = self.breakers.slack.clone();
                tasks.push(tokio::spawn(async move {
                    match Self::send_slack_with_retry(&client, &slack, &alert).await {
                        Ok(()) => {
                            breaker.record_success();
                            Ok(())
                        }
                        Err(e) => {
                            breaker.record_failure();
                            Err(e)
                        }
                    }
                }));
            }
        }

        if let Some(pagerduty) = &self.config.pagerduty {
            if !self.breakers.pagerduty.is_open() && self.check_rate_limit("pagerduty") {
                let pagerduty = pagerduty.clone();
                let alert = alert.clone();
                let client = self.client.clone();
                let breaker = self.breakers.pagerduty.clone();
                tasks.push(tokio::spawn(async move {
                    match Self::send_pagerduty_with_retry(&client, &pagerduty, &alert).await {
                        Ok(()) => {
                            breaker.record_success();
                            Ok(())
                        }
                        Err(e) => {
                            breaker.record_failure();
                            Err(e)
                        }
                    }
                }));
            }
        }

        if let Some(datadog) = &self.config.datadog {
            if !self.breakers.datadog.is_open() && self.check_rate_limit("datadog") {
                let datadog = datadog.clone();
                let alert = alert.clone();
                let client = self.client.clone();
                let breaker = self.breakers.datadog.clone();
                tasks.push(tokio::spawn(async move {
                    match Self::send_datadog_with_retry(&client, &datadog, &alert).await {
                        Ok(()) => {
                            breaker.record_success();
                            Ok(())
                        }
                        Err(e) => {
                            breaker.record_failure();
                            Err(e)
                        }
                    }
                }));
            }
        }

        if let Some(splunk) = &self.config.splunk {
            if !self.breakers.splunk.is_open() && self.check_rate_limit("splunk") {
                let splunk = splunk.clone();
                let alert = alert.clone();
                let client = self.client.clone();
                let breaker = self.breakers.splunk.clone();
                tasks.push(tokio::spawn(async move {
                    match Self::send_splunk_with_retry(&client, &splunk, &alert).await {
                        Ok(()) => {
                            breaker.record_success();
                            Ok(())
                        }
                        Err(e) => {
                            breaker.record_failure();
                            Err(e)
                        }
                    }
                }));
            }
        }

        if let Some(siem) = &self.config.siem {
            if !self.breakers.siem.is_open() && self.check_rate_limit("siem") {
                let siem = siem.clone();
                let alert = alert.clone();
                let client = self.client.clone();
                let breaker = self.breakers.siem.clone();
                tasks.push(tokio::spawn(async move {
                    match Self::send_siem_with_retry(&client, &siem, &alert).await {
                        Ok(()) => {
                            breaker.record_success();
                            Ok(())
                        }
                        Err(e) => {
                            breaker.record_failure();
                            Err(e)
                        }
                    }
                }));
            }
        }

        if let Some(syslog) = &self.config.syslog {
            if !self.breakers.syslog.is_open() && self.check_rate_limit("syslog") {
                let syslog = syslog.clone();
                let alert = alert.clone();
                let breaker = self.breakers.syslog.clone();
                tasks.push(tokio::spawn(async move {
                    match Self::send_syslog(&syslog, &alert).await {
                        Ok(()) => {
                            breaker.record_success();
                            Ok(())
                        }
                        Err(e) => {
                            breaker.record_failure();
                            Err(e)
                        }
                    }
                }));
            }
        }

        if let Some(webhook) = &self.config.webhook {
            if !self.breakers.webhook.is_open() && self.check_rate_limit("webhook") {
                let webhook = webhook.clone();
                let alert = alert.clone();
                let client = self.client.clone();
                let breaker = self.breakers.webhook.clone();
                tasks.push(tokio::spawn(async move {
                    match Self::send_webhook_with_retry(&client, &webhook, &alert).await {
                        Ok(()) => {
                            breaker.record_success();
                            Ok(())
                        }
                        Err(e) => {
                            breaker.record_failure();
                            Err(e)
                        }
                    }
                }));
            }
        }

        for task in tasks {
            if let Err(e) = task.await {
                log::error!("Integration task panicked: {}", e);
            }
        }

        Ok(())
    }

    /// Check health of all configured integrations.
    pub async fn health_check(&self) -> Vec<(&str, bool)> {
        let mut results = Vec::new();

        if self.config.slack.is_some() {
            results.push(("slack", !self.breakers.slack.is_open()));
        }
        if self.config.pagerduty.is_some() {
            results.push(("pagerduty", !self.breakers.pagerduty.is_open()));
        }
        if self.config.datadog.is_some() {
            results.push(("datadog", !self.breakers.datadog.is_open()));
        }
        if self.config.splunk.is_some() {
            results.push(("splunk", !self.breakers.splunk.is_open()));
        }
        if self.config.siem.is_some() {
            results.push(("siem", !self.breakers.siem.is_open()));
        }
        if self.config.syslog.is_some() {
            results.push(("syslog", !self.breakers.syslog.is_open()));
        }
        if self.config.webhook.is_some() {
            results.push(("webhook", !self.breakers.webhook.is_open()));
        }

        results
    }

    /// Reset all circuit breakers (e.g., after config change).
    pub fn reset_breakers(&self) {
        self.breakers.slack.reset();
        self.breakers.pagerduty.reset();
        self.breakers.datadog.reset();
        self.breakers.splunk.reset();
        self.breakers.siem.reset();
        self.breakers.syslog.reset();
        self.breakers.webhook.reset();
    }

    // ── Slack ─────────────────────────────────────────────────────────────────

    async fn send_slack_with_retry(
        client: &reqwest::Client,
        config: &SlackConfig,
        alert: &AlertPayload,
    ) -> Result<()> {
        Self::retry_with_backoff(3, || async {
            let emoji = match alert.severity.as_str() {
                "critical" => "🚨",
                "high" => "⚠️",
                "medium" => "⚡",
                _ => "ℹ️",
            };

            let payload = serde_json::json!({
                "channel": config.channel,
                "username": "Nexus Axiom",
                "icon_emoji": ":shield:",
                "attachments": [{
                    "color": match alert.severity.as_str() {
                        "critical" => "danger",
                        "high" => "warning",
                        _ => "good",
                    },
                    "title": format!("{} {}", emoji, alert.title),
                    "text": alert.description,
                    "fields": [
                        {"title": "Severity", "value": &alert.severity, "short": true},
                        {"title": "Source", "value": &alert.source, "short": true}
                    ],
                    "footer": "Nexus Axiom Security",
                    "ts": alert.timestamp
                }]
            });

            let response = client
                .post(&config.webhook_url)
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                anyhow::bail!("Slack API error: {}", response.status());
            }

            log::info!("Sent alert to Slack: {}", alert.title);
            Ok(())
        })
        .await
    }

    // ── PagerDuty ─────────────────────────────────────────────────────────────

    async fn send_pagerduty_with_retry(
        client: &reqwest::Client,
        config: &PagerDutyConfig,
        alert: &AlertPayload,
    ) -> Result<()> {
        Self::retry_with_backoff(3, || async {
            let payload = serde_json::json!({
                "routing_key": config.integration_key,
                "event_action": "trigger",
                "payload": {
                    "summary": alert.title,
                    "severity": alert.severity,
                    "source": alert.source,
                    "timestamp": alert.timestamp,
                    "custom_details": {
                        "description": alert.description,
                        "tags": alert.tags,
                        "correlation_id": alert.correlation_id,
                        "mitre_technique": alert.mitre_technique,
                    }
                }
            });

            let response = client
                .post("https://events.pagerduty.com/v2/enqueue")
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                anyhow::bail!("PagerDuty API error: {}", response.status());
            }

            log::info!("Sent alert to PagerDuty: {}", alert.title);
            Ok(())
        })
        .await
    }

    // ── Datadog ───────────────────────────────────────────────────────────────

    async fn send_datadog_with_retry(
        client: &reqwest::Client,
        config: &DatadogConfig,
        alert: &AlertPayload,
    ) -> Result<()> {
        Self::retry_with_backoff(3, || async {
            let payload = serde_json::json!({
                "title": alert.title,
                "text": alert.description,
                "priority": match alert.severity.as_str() {
                    "critical" => "normal",
                    _ => "low"
                },
                "tags": alert.tags,
                "alert_type": "error"
            });

            let url = format!("https://api.{}/api/v1/events", config.site);
            let response = client
                .post(&url)
                .header("DD-API-KEY", &config.api_key)
                .header("DD-APPLICATION-KEY", &config.app_key)
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                anyhow::bail!("Datadog API error: {}", response.status());
            }

            log::info!("Sent alert to Datadog: {}", alert.title);
            Ok(())
        })
        .await
    }

    // ── Splunk HEC ────────────────────────────────────────────────────────────

    async fn send_splunk_with_retry(
        client: &reqwest::Client,
        config: &SplunkConfig,
        alert: &AlertPayload,
    ) -> Result<()> {
        Self::retry_with_backoff(3, || async {
            let payload = serde_json::json!({
                "event": {
                    "severity": alert.severity,
                    "title": alert.title,
                    "description": alert.description,
                    "source": alert.source,
                    "tags": alert.tags,
                    "correlation_id": alert.correlation_id,
                    "mitre_technique": alert.mitre_technique,
                },
                "index": config.index,
                "sourcetype": "nexus_axiom",
                "source": "nexus-axiom"
            });

            let response = client
                .post(&config.hec_url)
                .header("Authorization", format!("Splunk {}", config.hec_token))
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                anyhow::bail!("Splunk API error: {}", response.status());
            }

            log::info!("Sent alert to Splunk: {}", alert.title);
            Ok(())
        })
        .await
    }

    // ── Generic SIEM ──────────────────────────────────────────────────────────

    async fn send_siem_with_retry(
        client: &reqwest::Client,
        config: &SIEMConfig,
        alert: &AlertPayload,
    ) -> Result<()> {
        Self::retry_with_backoff(3, || async {
            let payload = match config.format.as_str() {
                "CEF" => Self::format_cef(alert),
                "LEEF" => Self::format_leef(alert),
                "OCSF" => Self::format_ocsf(alert),
                _ => serde_json::to_string(alert)?,
            };

            let response = client
                .post(&config.endpoint)
                .header("Authorization", format!("Bearer {}", config.api_key))
                .header("Content-Type", "application/json")
                .body(payload)
                .send()
                .await?;

            if !response.status().is_success() {
                anyhow::bail!("SIEM API error: {}", response.status());
            }

            log::info!("Sent alert to SIEM: {}", alert.title);
            Ok(())
        })
        .await
    }

    // ── Syslog (RFC 5424 / RFC 3164) ─────────────────────────────────────────

    async fn send_syslog(config: &SyslogConfig, alert: &AlertPayload) -> Result<()> {
        use std::net::UdpSocket;

        let severity = match alert.severity.as_str() {
            "critical" => 2u8, // Critical
            "high" => 3,       // Error
            "medium" => 4,     // Warning
            "low" => 6,        // Informational
            _ => 6,
        };

        let priority = (config.facility as u16 * 8 + severity as u16) as u8;
        let timestamp = chrono::Utc::now().to_rfc3339();
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "nexus-axiom".to_string());

        let message = match config.rfc.as_str() {
            "5424" => {
                // RFC 5424 format
                format!(
                    "<{}>1 {} {} nexus-axiom - - - {} | {}",
                    priority, timestamp, hostname, alert.title, alert.description
                )
            }
            _ => {
                // RFC 3164 format (legacy)
                format!(
                    "<{}>{} {} nexus-axiom: {} | {}",
                    priority, timestamp, hostname, alert.title, alert.description
                )
            }
        };

        let target = format!("{}:{}", config.host, config.port);

        match config.protocol.as_str() {
            "tcp" => {
                use tokio::io::AsyncWriteExt;
                let mut stream = tokio::net::TcpStream::connect(&target).await?;
                stream.write_all(message.as_bytes()).await?;
                stream.write_all(b"\n").await?;
            }
            _ => {
                // UDP (default)
                let socket = UdpSocket::bind("0.0.0.0:0")?;
                socket.send_to(message.as_bytes(), &target)?;
            }
        }

        log::debug!("Sent syslog to {}: {}", target, alert.title);
        Ok(())
    }

    // ── Generic Webhook ───────────────────────────────────────────────────────

    async fn send_webhook_with_retry(
        client: &reqwest::Client,
        config: &WebhookConfig,
        alert: &AlertPayload,
    ) -> Result<()> {
        Self::retry_with_backoff(3, || async {
            let body = if let Some(ref template) = config.template {
                // Simple template substitution
                template
                    .replace("{{severity}}", &alert.severity)
                    .replace("{{title}}", &alert.title)
                    .replace("{{description}}", &alert.description)
                    .replace("{{source}}", &alert.source)
                    .replace("{{timestamp}}", &alert.timestamp.to_string())
            } else {
                serde_json::to_string(alert)?
            };

            let mut request = match config.method.to_uppercase().as_str() {
                "PUT" => client.put(&config.url),
                _ => client.post(&config.url),
            };

            for (key, value) in &config.headers {
                request = request.header(key.as_str(), value.as_str());
            }

            let response = request
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await?;

            if !response.status().is_success() {
                anyhow::bail!("Webhook error: {}", response.status());
            }

            log::info!("Sent alert to webhook: {}", alert.title);
            Ok(())
        })
        .await
    }

    // ── Retry with exponential backoff ────────────────────────────────────────

    async fn retry_with_backoff<F, Fut, T>(max_retries: u32, mut f: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut retries = 0;
        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if retries < max_retries => {
                    retries += 1;
                    let delay = Duration::from_millis(100 * 2_u64.pow(retries - 1));
                    log::warn!("Retry {}/{} after {:?}: {}", retries, max_retries, delay, e);
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    // ── Format helpers ────────────────────────────────────────────────────────

    fn format_cef(alert: &AlertPayload) -> String {
        let severity_int = match alert.severity.as_str() {
            "critical" => "10",
            "high" => "7",
            "medium" => "5",
            _ => "3",
        };
        format!(
            "CEF:0|NexusAxiom|Security|1.0|{}|{}|{}|msg={} src={} cs1={} cs1Label=correlation_id",
            alert.severity,
            alert.title,
            severity_int,
            alert.description,
            alert.source,
            alert.correlation_id.as_deref().unwrap_or("none")
        )
    }

    fn format_leef(alert: &AlertPayload) -> String {
        format!(
            "LEEF:2.0|NexusAxiom|Security|1.0|{}|severity={}\ttitle={}\tdesc={}\tsrc={}\tcorrelationId={}",
            alert.severity, alert.severity, alert.title, alert.description, alert.source,
            alert.correlation_id.as_deref().unwrap_or("none")
        )
    }

    /// Format alert as OCSF (Open Cybersecurity Schema Framework) v1.1
    fn format_ocsf(alert: &AlertPayload) -> String {
        let ocsf = serde_json::json!({
            "class_uid": 2001,  // Security Finding
            "category_uid": 2,   // Findings
            "severity_id": match alert.severity.as_str() {
                "critical" => 5,
                "high" => 4,
                "medium" => 3,
                "low" => 2,
                _ => 1
            },
            "type_uid": 200101,  // Security Finding: Create
            "time": alert.timestamp,
            "message": alert.title,
            "finding_info": {
                "title": alert.title,
                "desc": alert.description,
                "src_url": alert.source,
                "types": alert.tags,
                "uid": alert.correlation_id,
            },
            "metadata": {
                "product": {
                    "name": "Nexus Axiom",
                    "vendor_name": "Nexus Axiom",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "version": "1.1.0"
            }
        });
        serde_json::to_string(&ocsf).unwrap_or_default()
    }
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            slack: None,
            pagerduty: None,
            datadog: None,
            splunk: None,
            siem: None,
            syslog: None,
            webhook: None,
        }
    }
}
