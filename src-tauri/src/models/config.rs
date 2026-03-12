use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamberConfig {
    pub orchestrator: OrchestratorConfig,
    pub reasoning_models: Vec<ReasoningModel>,
    pub tools: ToolsConfig,
    pub workspace: WorkspaceConfig,
    pub sidecar: SidecarConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub temperature: f32,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningModel {
    pub name: String,
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub temperature: f32,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    #[serde(default)]
    pub enabled_tools: Vec<String>,
    #[serde(default)]
    pub approval_required: bool,
    #[serde(default)]
    pub approval_timeout_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub path: String,
    #[serde(default)]
    pub sessions_dir: String,
    #[serde(default)]
    pub config_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub health_check_interval_seconds: u32,
    #[serde(default)]
    pub max_restart_attempts: u32,
}

impl Default for ChamberConfig {
    fn default() -> Self {
        Self {
            orchestrator: OrchestratorConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-5".to_string(),
                temperature: 0.7,
                max_tokens: Some(4096),
            },
            reasoning_models: vec![
                ReasoningModel {
                    name: "claude-opus".to_string(),
                    provider: "anthropic".to_string(),
                    model: "claude-opus-4-5".to_string(),
                    enabled: true,
                    temperature: 0.7,
                    max_tokens: Some(4096),
                },
                ReasoningModel {
                    name: "gemini-thinking".to_string(),
                    provider: "gemini".to_string(),
                    model: "gemini-2.0-flash-thinking-exp".to_string(),
                    enabled: true,
                    temperature: 0.7,
                    max_tokens: Some(4096),
                },
                ReasoningModel {
                    name: "grok".to_string(),
                    provider: "xai".to_string(),
                    model: "grok-beta".to_string(),
                    enabled: false,
                    temperature: 0.7,
                    max_tokens: Some(4096),
                },
            ],
            tools: ToolsConfig {
                enabled_tools: vec![
                    "web_search".to_string(),
                    "calculator".to_string(),
                    "file_ops".to_string(),
                ],
                approval_required: true,
                approval_timeout_seconds: 300,
            },
            workspace: WorkspaceConfig {
                path: "${HOME}/.chamber/workspace".to_string(),
                sessions_dir: "sessions".to_string(),
                config_dir: "config".to_string(),
            },
            sidecar: SidecarConfig {
                host: "127.0.0.1".to_string(),
                port: 8765,
                health_check_interval_seconds: 30,
                max_restart_attempts: 3,
            },
        }
    }
}

impl ChamberConfig {
    pub fn expand_env_vars(&mut self) {
        self.workspace.path = Self::expand_path(&self.workspace.path);
    }

    fn expand_path(path: &str) -> String {
        if path.contains("${HOME}") {
            if let Some(home) = dirs::home_dir() {
                return path.replace("${HOME}", &home.to_string_lossy());
            }
        }
        path.to_string()
    }
}
