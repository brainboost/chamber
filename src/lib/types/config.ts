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

// Authentication types
export type AuthType = "api_key" | "oauth_token";

export interface Credential {
  provider: string;
  auth_type: AuthType;
  // For API Key type
  key?: string;
  // For OAuth Token type
  access_token?: string;
  refresh_token?: string;
  expires_at?: number;
  scopes?: string[];
}

export interface ApiKeyCredential extends Credential {
  auth_type: "api_key";
  key: string;
}

export interface OAuthTokenCredential extends Credential {
  auth_type: "oauth_token";
  access_token: string;
  refresh_token: string;
  expires_at?: number;
  scopes: string[];
}

export interface OAuthConfig {
  provider: string;
  auth_url: string;
  token_url: string;
  scopes: string[];
  client_id?: string;
  redirect_uri: string;
}

export interface ProviderAuthStatus {
  provider: string;
  has_credential: boolean;
  auth_type?: AuthType;
  // For OAuth, whether the token needs refresh
  needs_refresh?: boolean;
}
