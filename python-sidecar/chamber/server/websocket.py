"""WebSocket Streaming for Chamber Sidecar."""

import json
import logging
from typing import Any

from fastapi import APIRouter, WebSocket, WebSocketDisconnect

logger = logging.getLogger(__name__)

websocket_router = APIRouter()


class ConnectionManager:
    """Manage WebSocket connections."""

    def __init__(self):
        """Initialize connection manager."""
        self.active_connections: dict[str, WebSocket] = {}

    async def connect(self, session_id: str, websocket: WebSocket):
        """Connect a WebSocket.

        Args:
            session_id: Session ID
            websocket: WebSocket connection
        """
        await websocket.accept()
        self.active_connections[session_id] = websocket
        logger.info(f"WebSocket connected for session {session_id}")

    def disconnect(self, session_id: str):
        """Disconnect a WebSocket.

        Args:
            session_id: Session ID
        """
        if session_id in self.active_connections:
            del self.active_connections[session_id]
            logger.info(f"WebSocket disconnected for session {session_id}")

    async def send_message(self, session_id: str, message: dict):
        """Send message to a specific session.

        Args:
            session_id: Session ID
            message: Message dict
        """
        if session_id in self.active_connections:
            websocket = self.active_connections[session_id]
            await websocket.send_json(message)


manager = ConnectionManager()


@websocket_router.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket, session_id: str = None):  # ty:ignore[invalid-parameter-default]
    """WebSocket endpoint for real-time updates.

    Args:
        websocket: WebSocket connection
        session_id: Optional session ID from query params
    """
    if not session_id:
        await websocket.close(code=1008, reason="session_id required")
        return

    await manager.connect(session_id, websocket)

    try:
        while True:
            # Receive messages (for keep-alive or client commands)
            data = await websocket.receive_text()

            # Echo back for now (in production, handle commands)
            await websocket.send_json({"type": "ack", "data": "received"})

    except WebSocketDisconnect:
        manager.disconnect(session_id)
        logger.info(f"Client disconnected: {session_id}")
    except Exception as e:
        logger.error(f"WebSocket error: {e}")
        manager.disconnect(session_id)
