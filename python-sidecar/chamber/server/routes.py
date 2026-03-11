"""API Routes for Chamber Sidecar."""

import logging
from typing import Any

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from chamber.state import create_initial_state
from chamber.graph.chamber_graph import ChamberGraph

logger = logging.getLogger(__name__)

router = APIRouter()

# Global state (in production, use proper session management)
sessions = {}
graphs = {}


class MessageRequest(BaseModel):
    """Message request model."""
    session_id: str
    message: dict


class SidecarResponse(BaseModel):
    """Standard sidecar response."""
    success: bool
    data: Any = None
    error: str | None = None


@router.post("/session/message")
async def send_message(request: MessageRequest) -> SidecarResponse:
    """Send a message to a session.

    Args:
        request: Message request

    Returns:
        SidecarResponse
    """
    try:
        session_id = request.session_id
        message = request.message

        logger.info(f"Received message for session {session_id}")

        # Create or get session
        if session_id not in sessions:
            # Create initial state
            user_content = message.get("content", "")
            state = create_initial_state(session_id, user_content)
            sessions[session_id] = state

            # Create graph (in production, load config from file)
            config = {
                "orchestrator": {
                    "provider": "anthropic",
                    "model": "claude-sonnet-4-5",
                    "temperature": 0.7,
                },
                "reasoning_models": [
                    {
                        "name": "claude-opus",
                        "provider": "anthropic",
                        "model": "claude-opus-4-5",
                        "enabled": True,
                    }
                ],
                "tools": {
                    "enabled_tools": ["calculator"],
                    "approval_required": True,
                },
            }

            graph = ChamberGraph(config)
            compiled_graph = graph.build()
            graphs[session_id] = compiled_graph

        # Execute graph
        graph = graphs[session_id]
        state = sessions[session_id]

        # Run graph (this would be async in production with streaming)
        result = await graph.ainvoke(state)

        sessions[session_id] = result

        return SidecarResponse(
            success=True,
            data={"session_id": session_id, "status": "processing"}
        )

    except Exception as e:
        logger.error(f"Error processing message: {e}", exc_info=True)
        return SidecarResponse(
            success=False,
            error=str(e)
        )


@router.post("/session/{session_id}/pause")
async def pause_session(session_id: str) -> SidecarResponse:
    """Pause a session.

    Args:
        session_id: Session ID

    Returns:
        SidecarResponse
    """
    try:
        if session_id not in sessions:
            raise HTTPException(status_code=404, detail="Session not found")

        logger.info(f"Pausing session {session_id}")

        # Save checkpoint (implement in production)

        return SidecarResponse(
            success=True,
            data={"session_id": session_id, "status": "paused"}
        )

    except Exception as e:
        logger.error(f"Error pausing session: {e}")
        return SidecarResponse(
            success=False,
            error=str(e)
        )


@router.post("/session/{session_id}/resume")
async def resume_session(session_id: str) -> SidecarResponse:
    """Resume a paused session.

    Args:
        session_id: Session ID

    Returns:
        SidecarResponse
    """
    try:
        if session_id not in sessions:
            raise HTTPException(status_code=404, detail="Session not found")

        logger.info(f"Resuming session {session_id}")

        # Load checkpoint (implement in production)

        return SidecarResponse(
            success=True,
            data={"session_id": session_id, "status": "resumed"}
        )

    except Exception as e:
        logger.error(f"Error resuming session: {e}")
        return SidecarResponse(
            success=False,
            error=str(e)
        )
