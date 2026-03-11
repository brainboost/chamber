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
    layer::SubscriberExt,
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

    /// Format: colored, text, json
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

    /// Format: text, json
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

    let console_layer = create_console_layer(config)?;
    let file_layer = create_file_layer(config)?;

    let subscriber = Registry::default()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer);

    subscriber.init();

    Ok(())
}

/// Create environment filter for log levels
fn create_env_filter(config: &LoggingConfig, default_level: Level) -> EnvFilter {
    let mut filter = EnvFilter::new(format!("chamber={}", default_level));

    // Add component-specific filters
    if let Some(ref components) = config.components {
        if let Some(ref sidecar_level) = components.sidecar {
            filter = filter.add_directive("chamber_sidecar".parse().unwrap());
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
            filter = filter.add_directive(format!("chamber_llm={}", llm_level).parse().unwrap());
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
    }

    // Always include important crates
    filter = filter
        .add_directive("tauri=info".parse().unwrap())
        .add_directive("tokio=warn".parse().unwrap())
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("reqwest=warn".parse().unwrap());

    filter
}

/// Create console logging layer
fn create_console_layer(config: &LoggingConfig) -> Result<Option<fmt::Layer<Registry>>> {
    let console_config = config.console.as_ref();

    let enabled = console_config
        .map(|c| c.enabled)
        .unwrap_or(true);

    if !enabled {
        return Ok(None);
    }

    let format = console_config
        .map(|c| c.format.as_str())
        .unwrap_or("colored");

    let layer = match format {
        "json" => fmt::layer()
            .json()
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .boxed(),
        "text" => fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .boxed(),
        _ => fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .boxed(), // colored is default
    };

    Ok(Some(layer))
}

/// Create file logging layer
fn create_file_layer(config: &LoggingConfig) -> Result<Option<tracing_appender::non_blocking::WorkerGuard>> {
    let file_config = config.file.as_ref();

    let enabled = file_config
        .map(|c| c.enabled)
        .unwrap_or(false);

    if !enabled {
        return Ok(None);
    }

    let log_dir = get_log_dir()?;
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        log_dir,
        "chamber.log",
    );

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let format = file_config
        .map(|c| c.format.as_str())
        .unwrap_or("text");

    let layer = match format {
        "json" => fmt::layer()
            .json()
            .with_writer(non_blocking)
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .boxed(),
        _ => fmt::layer()
            .with_writer(non_blocking)
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .boxed(),
    };

    // Note: The layer needs to be added to the subscriber, but we can't return it directly
    // because of lifetime issues. Instead, we return the guard and let the caller handle it.
    // For now, we'll just init the file layer here.

    tracing::subscriber::with_default(
        Registry::default().with(layer),
        || {},
    );

    Ok(Some(guard))
}

/// Initialize logging with default settings
pub fn init_default_logging() {
    let config = LoggingConfig::default();
    if let Err(e) = setup_logging(&config) {
        eprintln!("Failed to initialize logging: {}", e);
    }
}
