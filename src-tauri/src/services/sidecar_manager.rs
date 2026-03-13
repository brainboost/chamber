use crate::models::config::SidecarConfig;
use crate::models::message::{SidecarRequest, SidecarResponse};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::Read;
use std::process::{Child, ChildStderr, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct SidecarManager {
    config: SidecarConfig,
    process: Arc<Mutex<Option<Child>>>,
    process_stderr: Arc<Mutex<Option<ChildStderr>>>,
    restart_count: Arc<Mutex<u32>>,
    credential_manager: Option<Arc<Mutex<Option<super::CredentialManager>>>>,
}

impl SidecarManager {
    pub fn new(config: SidecarConfig) -> Self {
        Self {
            config,
            process: Arc::new(Mutex::new(None)),
            process_stderr: Arc::new(Mutex::new(None)),
            restart_count: Arc::new(Mutex::new(0)),
            credential_manager: None,
        }
    }

    pub fn with_credentials(
        config: SidecarConfig,
        credential_manager: Arc<Mutex<Option<super::CredentialManager>>>,
    ) -> Self {
        Self {
            config,
            process: Arc::new(Mutex::new(None)),
            process_stderr: Arc::new(Mutex::new(None)),
            restart_count: Arc::new(Mutex::new(0)),
            credential_manager: Some(credential_manager),
        }
    }

    pub async fn start(&self, sidecar_path: &str) -> Result<()> {
        {
            let process = self.process.lock().await;
            if process.is_some() {
                return Ok(()); // Already running
            }
        }

        // Fetch credentials from credential manager if available
        // These are read from secure keychain storage and injected as environment variables
        // The Python sidecar then reads them via os.getenv() for provider initialization
        let credential_envs = if let Some(cred_manager) = &self.credential_manager {
            let manager_lock = cred_manager.lock().await;
            if let Some(manager) = manager_lock.as_ref() {
                manager
                    .get_credentials_as_env()
                    .await
                    .unwrap_or_default()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        // Run the sidecar via `uv run python -m chamber.main` from the project directory.
        // For a production bundled binary, sidecar_path would be the executable itself;
        // detect that case by checking whether the path is a directory.
        let path = std::path::Path::new(sidecar_path);
        let mut cmd = if path.is_dir() {
            // Development / uv-managed mode
            let mut c = Command::new("uv");
            c.args(["run", "python", "-m", "chamber.main"])
                .arg("--host")
                .arg(&self.config.host)
                .arg("--port")
                .arg(self.config.port.to_string())
                .current_dir(sidecar_path)
                .stderr(Stdio::piped());
            c
        } else {
            // Production bundled binary
            #[cfg(target_os = "windows")]
            let executable = format!("{}.exe", sidecar_path);
            #[cfg(not(target_os = "windows"))]
            let executable = sidecar_path.to_string();

            let mut c = Command::new(&executable);
            c.arg("--host")
                .arg(&self.config.host)
                .arg("--port")
                .arg(self.config.port.to_string())
                .stderr(Stdio::piped());
            c
        };

        // Inject credentials as environment variables
        // Python providers will read these via os.getenv() during initialization
        for (key, value) in &credential_envs {
            tracing::debug!("Injecting credential env: {}", key);
            cmd.env(key, value);
        }

        let mut child = cmd.spawn().with_context(|| {
            if path.is_dir() {
                format!(
                    "Failed to launch sidecar via `uv run` in '{}'. \
                    Is `uv` installed? Run: winget install astral-sh.uv  (or: pip install uv)\n\
                    Then run: cd {} && uv sync",
                    sidecar_path, sidecar_path
                )
            } else {
                format!("Failed to start sidecar binary '{}'", sidecar_path)
            }
        })?;

        // Take stderr handle before storing the child
        let stderr_handle = child.stderr.take();

        {
            let mut process = self.process.lock().await;
            *process = Some(child);
        }
        *self.process_stderr.lock().await = stderr_handle;

        // Wait for sidecar to become healthy (lock is released above)
        self.wait_for_ready().await?;

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut process = self.process.lock().await;
        if let Some(mut child) = process.take() {
            child.kill().context("Failed to kill sidecar process")?;
            child.wait().context("Failed to wait for sidecar process")?;
        }
        drop(process);

        *self.process_stderr.lock().await = None;

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
        let max_attempts = 30; // 15 seconds total

        for _ in 0..max_attempts {
            // Check if the process exited early (crash, import error, port conflict, etc.)
            let exit_status = {
                let mut process_lock = self.process.lock().await;
                if let Some(child) = process_lock.as_mut() {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            *process_lock = None; // Clear the dead handle
                            Some(status)
                        }
                        _ => None, // Still running or couldn't check
                    }
                } else {
                    None
                }
            };

            if let Some(status) = exit_status {
                // Process died — collect stderr to explain why
                let stderr_text = {
                    let mut stderr_lock = self.process_stderr.lock().await;
                    if let Some(mut stderr) = stderr_lock.take() {
                        let mut buf = String::new();
                        let _ = stderr.read_to_string(&mut buf);
                        buf.trim().to_string()
                    } else {
                        String::new()
                    }
                };

                if stderr_text.is_empty() {
                    anyhow::bail!(
                        "Sidecar process exited unexpectedly ({}). \
                        Check that Python dependencies are installed: cd python-sidecar && uv sync",
                        status
                    );
                } else {
                    anyhow::bail!(
                        "Sidecar process exited unexpectedly ({}):\n{}",
                        status,
                        stderr_text
                    );
                }
            }

            if self.health_check().await.unwrap_or(false) {
                return Ok(());
            }

            sleep(Duration::from_millis(500)).await;
        }

        anyhow::bail!(
            "Sidecar did not respond to health checks within 15s on {}:{}. \
            Check that port {} is not already in use.",
            self.config.host,
            self.config.port,
            self.config.port
        )
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
            .timeout(Duration::from_secs(120))
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
        let mut process = self.process.lock().await;
        if let Some(child) = process.as_mut() {
            match child.try_wait() {
                Ok(None) => true, // Still alive
                _ => {
                    *process = None; // Clean up dead handle
                    false
                }
            }
        } else {
            false
        }
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
