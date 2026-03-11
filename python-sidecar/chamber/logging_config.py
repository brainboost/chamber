"""Logging configuration for Chamber sidecar."""

import logging
import logging.handlers
import sys
from pathlib import Path
from typing import Literal

import yaml
from pythonjsonlogger import jsonlogger

# Log level type
LogLevel = Literal["debug", "info", "warning", "error", "critical"]

# Default log format
DEFAULT_FORMAT = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
COLORED_FORMAT = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
JSON_FORMAT = "%(asctime)s %(name)s %(levelname)s %(message)s %(pathname)s %(lineno)d"


class ColoredFormatter(logging.Formatter):
    """Colored log formatter for console output."""

    # ANSI color codes
    COLORS = {
        "DEBUG": "\033[36m",  # Cyan
        "INFO": "\033[32m",  # Green
        "WARNING": "\033[33m",  # Yellow
        "ERROR": "\033[31m",  # Red
        "CRITICAL": "\033[35m",  # Magenta
    }
    RESET = "\033[0m"

    def format(self, record: logging.LogRecord) -> str:
        """Format log record with colors."""
        levelname = record.levelname
        if levelname in self.COLORS:
            record.levelname = f"{self.COLORS[levelname]}{levelname}{self.RESET}"
        return super().format(record)


def setup_logging(
    level: LogLevel = "info",
    console_enabled: bool = True,
    console_format: Literal["colored", "text", "json"] = "colored",
    file_enabled: bool = False,
    file_path: str | Path | None = None,
    file_max_size_mb: int = 10,
    file_backup_count: int = 5,
) -> None:
    """Setup logging configuration.

    Args:
        level: Log level (debug, info, warning, error, critical)
        console_enabled: Enable console logging
        console_format: Console format (colored, text, json)
        file_enabled: Enable file logging
        file_path: Path to log file
        file_max_size_mb: Max size of log file before rotation
        file_backup_count: Number of backup files to keep
    """
    # Get root logger
    root_logger = logging.getLogger()
    root_logger.setLevel(getattr(logging, level.upper()))

    # Clear existing handlers
    root_logger.handlers.clear()

    # Setup console handler
    if console_enabled:
        console_handler = logging.StreamHandler(sys.stdout)

        if console_format == "colored":
            console_handler.setFormatter(ColoredFormatter(COLORED_FORMAT))
        elif console_format == "json":
            console_handler.setFormatter(
                jsonlogger.JsonFormatter(JSON_FORMAT)
            )
        else:  # text
            console_handler.setFormatter(logging.Formatter(DEFAULT_FORMAT))

        console_handler.setLevel(getattr(logging, level.upper()))
        root_logger.addHandler(console_handler)

    # Setup file handler
    if file_enabled and file_path:
        file_path = Path(file_path)
        file_path.parent.mkdir(parents=True, exist_ok=True)

        file_handler = logging.handlers.RotatingFileHandler(
            file_path,
            maxBytes=file_max_size_mb * 1024 * 1024,
            backupCount=file_backup_count,
        )

        # Use JSON format for files
        file_handler.setFormatter(jsonlogger.JsonFormatter(JSON_FORMAT))
        file_handler.setLevel(getattr(logging, level.upper()))
        root_logger.addHandler(file_handler)


def load_logging_config(config_path: str | Path | None = None) -> dict:
    """Load logging configuration from YAML file.

    Args:
        config_path: Path to config file. If None, uses default location.

    Returns:
        Logging configuration dictionary
    """
    if config_path is None:
        config_path = Path.home() / ".chamber" / "workspace" / "config" / "chamber-config.yaml"
    else:
        config_path = Path(config_path)

    if not config_path.exists():
        # Return default config
        return {
            "logging": {
                "level": "info",
                "console": {"enabled": True, "format": "colored"},
                "file": {"enabled": False},
            }
        }

    with open(config_path) as f:
        config = yaml.safe_load(f)

    return config.get("logging", {})


def setup_logging_from_config(config_path: str | Path | None = None) -> None:
    """Setup logging from configuration file.

    Args:
        config_path: Path to config file. If None, uses default location.
    """
    config = load_logging_config(config_path)

    level = config.get("level", "info")

    console_config = config.get("console", {})
    console_enabled = console_config.get("enabled", True)
    console_format = console_config.get("format", "colored")

    file_config = config.get("file", {})
    file_enabled = file_config.get("enabled", False)
    file_path = file_config.get("path")
    if file_path and isinstance(file_path, str):
        file_path = file_path.replace("${HOME}", str(Path.home()))
    file_max_size = file_config.get("max_size_mb", 10)
    file_backup_count = file_config.get("backup_count", 5)

    setup_logging(
        level=level,
        console_enabled=console_enabled,
        console_format=console_format,
        file_enabled=file_enabled,
        file_path=file_path,
        file_max_size_mb=file_max_size,
        file_backup_count=file_backup_count,
    )

    # Set component-specific log levels
    components = config.get("components", {})
    for component, component_level in components.items():
        logger = logging.getLogger(component)
        logger.setLevel(getattr(logging, component_level.upper()))
