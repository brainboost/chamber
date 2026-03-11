#!/usr/bin/env python3
"""Chamber Sidecar Entry Point."""

import argparse
import asyncio
import logging
import sys
from pathlib import Path

import uvicorn

from chamber.logging_config import setup_logging_from_config
from chamber.server.app import create_app

logger = logging.getLogger(__name__)


def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Chamber Python Sidecar")
    parser.add_argument(
        "--host",
        type=str,
        default="127.0.0.1",
        help="Host to bind to (default: 127.0.0.1)",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=8765,
        help="Port to bind to (default: 8765)",
    )
    parser.add_argument(
        "--log-level",
        type=str,
        default=None,
        choices=["debug", "info", "warning", "error"],
        help="Log level (overrides config file)",
    )
    parser.add_argument(
        "--config",
        type=str,
        default=None,
        help="Path to config file",
    )
    parser.add_argument(
        "--no-file-log",
        action="store_true",
        help="Disable file logging",
    )
    return parser.parse_args()


def main():
    """Main entry point."""
    args = parse_args()

    # Setup logging from config file
    setup_logging_from_config(args.config)

    # Override log level if specified
    if args.log_level:
        logging.getLogger().setLevel(getattr(logging, args.log_level.upper()))

    logger.info(f"Starting Chamber Sidecar on {args.host}:{args.port}")
    logger.debug(f"Python version: {sys.version}")
    logger.debug(f"Working directory: {Path.cwd()}")

    # Create FastAPI app
    app = create_app()

    # Determine log level for uvicorn
    log_level = args.log_level or logging.getLevelName(logging.getLogger().level)

    # Run server
    uvicorn.run(
        app,
        host=args.host,
        port=args.port,
        log_level=log_level.lower(),
        access_log=True,
    )


if __name__ == "__main__":
    main()
