use crate::models::config::SidecarConfig;
use crate::models::message::{SidecarRequest, SidecarResponse};
use anyhow::{Context, Result};
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct SidecarManager {
    config: SidecarConfig,
    process: Arc<Mutex<Option<Child>>>,
    restart_count: Arc<Mutex<u32>>,
}

impl SidecarManager {
    pub fn new(config: SidecarConfig) -> Self {
        Self {
            config,
            process: Arc::new(Mutex::new(None)),
            restart_count: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn start(&self, sidecar_path: &str) -> Result<()> {
        let mut process = self.process.lock().await;

        if process.is_some() {
            return Ok(()); // Already running
        }

        #[cfg(target_os = "windows")]
        let executable = format!("{}.exe", sidecar_path);
        #[cfg(not(target_os = "windows"))]
        let executable = sidecar_path.to_string();

        let child = Command::new(&executable)
            .arg("--host")
            .arg(&self.config.host)
            .arg("--port")
            .arg(self.config.port.to_string())
            .spawn()
            .context("Failed to start Python sidecar")?;

        *process = Some(child);

        // Wait for sidecar to be ready
        self.wait_for_ready().await?;

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut process = self.process.lock().await;

        if let Some(mut child) = process.take() {
            child.kill().context("Failed to kill sidecar process")?;
            child.wait().context("Failed to wait for sidecar process")?;
        }

        Ok(())
    }

    pub async fn restart(&self, sidecar_path: &str) -> Result<()> {
        let mut restart_count = self.restart_count.lock().await;

        if *restart_count >= self.config.max_restart_attempts {
            anyhow::bail!(
                "Max restart attempts ({}) reached",
                self.config.max_restart_attempts
            );
        }

        *restart_count += 1;

        self.stop().await?;
        sleep(Duration::from_secs(2)).await;
        self.start(sidecar_path).await?;

        Ok(())
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("http://{}:{}/health", self.config.host, self.config.port);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn wait_for_ready(&self) -> Result<()> {
        let max_attempts = 30;
        let mut attempts = 0;

        while attempts < max_attempts {
            if self.health_check().await? {
                return Ok(());
            }

            sleep(Duration::from_millis(500)).await;
            attempts += 1;
        }

        anyhow::bail!("Sidecar failed to become ready within timeout")
    }

    pub async fn send_message(&self, request: SidecarRequest) -> Result<SidecarResponse> {
        let url = format!(
            "http://{}:{}/api/session/message",
            self.config.host, self.config.port
        );

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .context("Failed to send message to sidecar")?;

        let sidecar_response: SidecarResponse = response
            .json()
            .await
            .context("Failed to parse sidecar response")?;

        Ok(sidecar_response)
    }

    pub async fn pause_session(&self, session_id: &str) -> Result<SidecarResponse> {
        let url = format!(
            "http://{}:{}/api/session/{}/pause",
            self.config.host, self.config.port, session_id
        );

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .context("Failed to pause session")?;

        let sidecar_response: SidecarResponse = response
            .json()
            .await
            .context("Failed to parse pause response")?;

        Ok(sidecar_response)
    }

    pub async fn resume_session(&self, session_id: &str) -> Result<SidecarResponse> {
        let url = format!(
            "http://{}:{}/api/session/{}/resume",
            self.config.host, self.config.port, session_id
        );

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .context("Failed to resume session")?;

        let sidecar_response: SidecarResponse = response
            .json()
            .await
            .context("Failed to parse resume response")?;

        Ok(sidecar_response)
    }

    pub fn get_websocket_url(&self) -> String {
        format!("ws://{}:{}/ws", self.config.host, self.config.port)
    }

    pub async fn is_running(&self) -> bool {
        let process = self.process.lock().await;
        process.is_some()
    }

    pub async fn reset_restart_count(&self) {
        let mut restart_count = self.restart_count.lock().await;
        *restart_count = 0;
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        // Note: Can't use async in Drop, so we use try_lock
        if let Ok(mut process) = self.process.try_lock() {
            if let Some(mut child) = process.take() {
                let _ = child.kill();
            }
        }
    }
}
