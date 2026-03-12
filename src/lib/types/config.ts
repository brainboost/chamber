export interface ChamberConfig {
  orchestrator: OrchestratorConfig;
  reasoning_models: ReasoningModel[];
  tools: ToolsConfig;
  workspace: WorkspaceConfig;
  sidecar: SidecarConfig;
  ui: UIConfig;
}

export type ThemeMode = 'light' | 'dark' | 'system';

export interface UIConfig {
  theme: ThemeMode;
}

export interface OrchestratorConfig {
  provider: string;
  model: string;
  temperature?: number;
  max_tokens?: number;
}

export interface ReasoningModel {
  name: string;
  provider: string;
  model: string;
  enabled: boolean;
  temperature?: number;
  max_tokens?: number;
}

export interface ToolsConfig {
  enabled_tools: string[];
  approval_required: boolean;
  approval_timeout_seconds: number;
}

export interface WorkspaceConfig {
  path: string;
  sessions_dir: string;
  config_dir: string;
}

export interface SidecarConfig {
  host: string;
  port: number;
  health_check_interval_seconds: number;
  max_restart_attempts: number;
}
