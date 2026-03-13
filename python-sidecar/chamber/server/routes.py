"""API Routes for Chamber Sidecar."""

import logging
import os
from typing import Any

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from chamber.state import create_initial_state
from chamber.graph.chamber_graph import ChamberGraph

logger = logging.getLogger(__name__)

router = APIRouter()

# Per-session compiled graphs (reused across turns to avoid rebuild cost)
# The intermediate planning state (orchestrator plan, reasoning, synthesis) is
# NOT carried across turns — each graph invocation gets a fresh single-turn state.
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


class CredentialsRequest(BaseModel):
    """Credential env vars pushed from the Tauri app."""
    env_vars: dict[str, str]


@router.post("/credentials")
async def update_credentials(request: CredentialsRequest) -> SidecarResponse:
    """Accept credential env vars from the Tauri app and apply them at runtime.

    Called after the user saves or imports credentials in the UI so the sidecar
    picks them up without needing a restart. Also clears cached graphs so the
    next session uses fresh provider instances with the new credentials.
    """
    for key, value in request.env_vars.items():
        os.environ[key] = value
        logger.info(f"Updated credential env: {key}")
    # Clear cached graphs so next request creates fresh providers with new credentials
    sessions.clear()
    graphs.clear()
    return SidecarResponse(success=True, data={"updated": list(request.env_vars.keys())})


@router.post("/session/message")
async def send_message(request: MessageRequest) -> SidecarResponse:
    """Send a message to a session.

    Args:
        request: Message request

    Returns:
        SidecarResponse with the AI's response in data.response
    """
    try:
        session_id = request.session_id
        message = request.message
        user_content = message.get("content", "")

        logger.info(f"Received message for session {session_id}")

        # Build the compiled graph once per session (reused across turns).
        # If it's missing (first request or failed init), build it now.
        if session_id not in graphs:
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
            # Build first — if this raises (e.g. missing API key) graphs stays empty
            # so the next request will retry cleanly instead of hitting a KeyError.
            graph_instance = ChamberGraph(config)
            graphs[session_id] = graph_instance.build()

        # Each turn gets a fresh single-turn state.
        # Intermediate planning messages ([ORCHESTRATOR PLAN], reasoning, synthesis)
        # are implementation details of one turn and must not leak into the next.
        state = create_initial_state(session_id, user_content)

        # Run the graph for this turn
        result = await graphs[session_id].ainvoke(state)

        # Extract final answer from the [FINAL ANSWER] message
        final_response = ""
        for msg in reversed(result.get("messages", [])):
            raw = getattr(msg, "content", None)
            # content can be a list of content blocks (LangChain structured output)
            if isinstance(raw, list):
                content = " ".join(
                    block["text"] if isinstance(block, dict) else getattr(block, "text", "")
                    for block in raw
                    if (isinstance(block, dict) and block.get("type") == "text")
                    or hasattr(block, "text")
                )
            elif isinstance(raw, str):
                content = raw
            else:
                continue
            if "[FINAL ANSWER]" in content:
                final_response = content.split("[FINAL ANSWER]\n", 1)[-1]
                break

        return SidecarResponse(
            success=True,
            data={"session_id": session_id, "status": "completed", "response": final_response}
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
        if session_id not in graphs:
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
        if session_id not in graphs:
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
