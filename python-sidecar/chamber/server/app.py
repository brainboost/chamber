"""FastAPI Application for Chamber Sidecar."""

import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from chamber.server.routes import router
from chamber.server.websocket import websocket_router

logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifespan context manager for startup/shutdown."""
    logger.info("Starting Chamber Sidecar")
    yield
    logger.info("Shutting down Chamber Sidecar")


def create_app() -> FastAPI:
    """Create and configure FastAPI application.

    Returns:
        FastAPI application instance
    """
    app = FastAPI(
        title="Chamber Sidecar",
        description="Multi-Model AI Reasoning Backend",
        version="0.1.0",
        lifespan=lifespan,
    )

    # Configure CORS
    app.add_middleware(
        CORSMiddleware,  # ty:ignore[invalid-argument-type]
        allow_origins=["*"],  # In production, specify Tauri origin
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # Include routers
    app.include_router(router, prefix="/api")
    app.include_router(websocket_router)

    @app.get("/health")
    async def health_check():
        """Health check endpoint."""
        return {"status": "healthy", "service": "chamber-sidecar"}

    return app
