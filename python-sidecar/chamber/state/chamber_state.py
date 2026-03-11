"""LangGraph State Definitions for Chamber."""

from typing import Annotated, Literal, TypedDict
from langgraph.graph import add_messages
from langchain_core.messages import BaseMessage


class ChamberState(TypedDict):
    """State for the Chamber multi-model reasoning workflow."""

    # Messages history
    messages: Annotated[list[BaseMessage], add_messages]

    # Current session ID
    session_id: str

    # Orchestrator plan
    orchestrator_plan: str

    # Reasoning responses from different models
    reasoning_responses: dict[str, str]

    # Orchestrator synthesis
    orchestrator_synthesis: str

    # Next action decision
    next_action: Literal["continue_reasoning", "use_tools", "finalize"]

    # Tool approval requests
    pending_tool_approval: dict | None

    # Tool execution results
    tool_results: list[dict]

    # Error state
    error: str | None

    # Iteration count (to prevent infinite loops)
    iteration_count: int


def create_initial_state(session_id: str, user_message: str) -> ChamberState:
    """Create initial state for a new session."""
    from langchain_core.messages import HumanMessage

    return ChamberState(
        messages=[HumanMessage(content=user_message)],
        session_id=session_id,
        orchestrator_plan="",
        reasoning_responses={},
        orchestrator_synthesis="",
        next_action="continue_reasoning",
        pending_tool_approval=None,
        tool_results=[],
        error=None,
        iteration_count=0,
    )
