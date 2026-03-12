//! Logging configuration for Chamber.
//!
//! Provides configurable logging with console and file output.

use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::{Layer, SubscriberExt},
    util::SubscriberInitExt,
    EnvFilter, Registry,
};

/// Logging configuration loaded from config file
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LoggingConfig {
    /// Log level: debug, info, warning, error
    pub level: Option<String>,

    /// Console logging configuration
    pub console: Option<ConsoleConfig>,

    /// File logging configuration
    pub file: Option<FileConfig>,

    /// Component-specific log levels
    #[serde(default)]
    pub components: ComponentConfig,
}

/// Console logging configuration
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ConsoleConfig {
    /// Enable console logging
    #[serde(default = "default_console_enabled")]
    pub enabled: bool,

    /// Format: colored, text
    #[serde(default = "default_console_format")]
    pub format: String,
}

/// File logging configuration
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FileConfig {
    /// Enable file logging
    #[serde(default = "default_file_enabled")]
    pub enabled: bool,

    /// Path to log file
    pub path: Option<String>,

    /// Max size of log file in MB
    #[serde(default = "default_file_max_size")]
    pub max_size_mb: usize,

    /// Number of backup files to keep
    #[serde(default = "default_file_backup_count")]
    pub backup_count: usize,

    /// Format: text
    #[serde(default = "default_file_format")]
    pub format: String,
}

/// Component-specific log levels
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ComponentConfig {
    #[serde(default)]
    pub sidecar: Option<String>,
    #[serde(default)]
    pub tauri: Option<String>,
    #[serde(default)]
    pub llm: Option<String>,
    #[serde(default)]
    pub tools: Option<String>,
    #[serde(default)]
    pub websocket: Option<String>,
}

// Default functions for serde
fn default_console_enabled() -> bool {
    true
}

fn default_console_format() -> String {
    "colored".to_string()
}

fn default_file_enabled() -> bool {
    false
}

fn default_file_max_size() -> usize {
    10
}

fn default_file_backup_count() -> usize {
    5
}

fn default_file_format() -> String {
    "text".to_string()
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Some("info".to_string()),
            console: Some(ConsoleConfig {
                enabled: true,
                format: "colored".to_string(),
            }),
            file: Some(FileConfig {
                enabled: false,
                path: None,
                max_size_mb: 10,
                backup_count: 5,
                format: "text".to_string(),
            }),
            components: ComponentConfig::default(),
        }
    }
}

/// Get the default log directory for Chamber
pub fn get_log_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "chamber", "Chamber")
        .context("Failed to get project directories")?;

    let log_dir = proj_dirs.data_local_dir().join("logs");
    std::fs::create_dir_all(&log_dir)
        .context("Failed to create log directory")?;

    Ok(log_dir)
}

/// Parse log level string to tracing::Level
fn parse_log_level(level: &str) -> Level {
    match level.to_lowercase().as_str() {
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warning" | "warn" => Level::WARN,
        "error" => Level::ERROR,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    }
}

/// Setup logging from configuration
pub fn setup_logging(config: &LoggingConfig) -> Result<()> {
    let log_level = config
        .level
        .as_ref()
        .map(|s| parse_log_level(s))
        .unwrap_or(Level::INFO);

    let env_filter = create_env_filter(config, log_level);

    // Check what's enabled
    let console_enabled = config
        .console
        .as_ref()
        .map(|c| c.enabled)
        .unwrap_or(true);

    let file_enabled = config
        .file
        .as_ref()
        .map(|c| c.enabled)
        .unwrap_or(false);

    // Simple approach: build layers based on what's enabled
    if console_enabled {
        if file_enabled {
            // Both enabled - use the multi-layer approach
            let log_dir = get_log_dir()?;
            let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "chamber.log");
            let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

            // Build subscriber with both layers
            let subscriber = Registry::default()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_file(true)
                        .with_line_number(true)
                        .with_span_events(FmtSpan::CLOSE),
                )
                .with(
                    fmt::layer()
                        .with_writer(non_blocking)
                        .with_file(true)
                        .with_line_number(true)
                        .with_span_events(FmtSpan::CLOSE)
                        .boxed(),
                );

            subscriber.init();

            // Leak the guard to keep it alive for the program duration
            Box::leak(Box::new(file_guard));
        } else {
            // Only console
            let subscriber = Registry::default()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_file(true)
                        .with_line_number(true)
                        .with_span_events(FmtSpan::CLOSE),
                );

            subscriber.init();
        }
    } else if file_enabled {
        // Only file
        let log_dir = get_log_dir()?;
        let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "chamber.log");
        let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

        let subscriber = Registry::default()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_writer(non_blocking)
                    .with_file(true)
                    .with_line_number(true)
                    .with_span_events(FmtSpan::CLOSE)
                    .boxed(),
            );

        subscriber.init();

        // Leak the guard to keep it alive
        Box::leak(Box::new(file_guard));
    } else {
        // Neither - just the filter
        let subscriber = Registry::default().with(env_filter);
        subscriber.init();
    }

    Ok(())
}

/// Create environment filter for log levels
fn create_env_filter(config: &LoggingConfig, default_level: Level) -> EnvFilter {
    let mut filter = EnvFilter::new(format!("chamber={}", default_level));

    // Add component-specific filters
    let components = &config.components;
    if let Some(ref sidecar_level) = components.sidecar {
        filter = filter.add_directive(
            format!("chamber_sidecar={}", sidecar_level)
                .parse()
                .unwrap(),
        );
    }
    if let Some(ref tauri_level) = components.tauri {
        filter = filter.add_directive(
            format!("chamber_tauri={}", tauri_level)
                .parse()
                .unwrap(),
        );
    }
    if let Some(ref llm_level) = components.llm {
        filter = filter
            .add_directive(format!("chamber_llm={}", llm_level).parse().unwrap());
    }
    if let Some(ref tools_level) = components.tools {
        filter = filter.add_directive(
            format!("chamber_tools={}", tools_level)
                .parse()
                .unwrap(),
        );
    }
    if let Some(ref websocket_level) = components.websocket {
        filter = filter.add_directive(
            format!("chamber_websocket={}", websocket_level)
                .parse()
                .unwrap(),
        );
    }

    // Always include important crates
    filter = filter
        .add_directive("tauri=info".parse().unwrap())
        .add_directive("tokio=warn".parse().unwrap())
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("reqwest=warn".parse().unwrap());

    filter
}

/// Initialize logging with default settings
pub fn init_default_logging() {
    let config = LoggingConfig::default();
    if let Err(e) = setup_logging(&config) {
        eprintln!("Failed to initialize logging: {}", e);
    }
}
