"""Main Chamber LangGraph Workflow."""

import logging
from typing import Literal

from langchain_core.messages import AIMessage, HumanMessage, SystemMessage
from langgraph.graph import END, StateGraph

from chamber.models import get_provider
from chamber.state import ChamberState
from chamber.tools import get_tools

logger = logging.getLogger(__name__)


class ChamberGraph:
    """Main Chamber multi-model reasoning graph."""

    def __init__(self, config: dict):
        """Initialize Chamber graph.

        Args:
            config: Chamber configuration dict
        """
        self.config = config
        self.orchestrator = None
        self.reasoning_models = []
        self.tools = []

        self._init_models()
        self._init_tools()

    def _init_models(self):
        """Initialize orchestrator and reasoning models."""
        # Initialize orchestrator
        orch_config = self.config["orchestrator"]
        self.orchestrator = get_provider(
            orch_config["provider"],
            orch_config["model"],
            orch_config.get("temperature", 0.7),
            orch_config.get("max_tokens"),
        ).get_model()

        # Initialize reasoning models
        for model_config in self.config["reasoning_models"]:
            if model_config.get("enabled", True):
                model = get_provider(
                    model_config["provider"],
                    model_config["model"],
                    model_config.get("temperature", 0.7),
                    model_config.get("max_tokens"),
                ).get_model()

                self.reasoning_models.append(
                    {
                        "name": model_config["name"],
                        "model": model,
                    }
                )

        logger.info(f"Initialized orchestrator and {len(self.reasoning_models)} reasoning models")

    def _init_tools(self):
        """Initialize tools."""
        tools_config = self.config.get("tools", {})
        enabled_tools = tools_config.get("enabled_tools", [])
        workspace_path = self.config.get("workspace", {}).get("path")

        chamber_tools = get_tools(enabled_tools, workspace_path)
        self.tools = [tool.get_tool() for tool in chamber_tools]

        logger.info(f"Initialized {len(self.tools)} tools")

    async def orchestrator_planning(self, state: ChamberState) -> ChamberState:
        """Orchestrator creates initial plan.

        Args:
            state: Current state

        Returns:
            Updated state with orchestrator plan
        """
        logger.info("Orchestrator planning phase")

        system_prompt = SystemMessage(
            content="""You are the orchestrator in a multi-model reasoning system.
Your role is to:
1. Analyze the user's request
2. Break it down into clear reasoning steps
3. Create a plan for how multiple AI models should approach this problem

Provide a concise plan (3-5 bullet points) that guides the reasoning process."""
        )

        messages = [system_prompt] + state["messages"]

        response = await self.orchestrator.ainvoke(messages)

        state["orchestrator_plan"] = response.content
        state["messages"].append(AIMessage(content=f"[ORCHESTRATOR PLAN]\n{response.content}"))

        return state

    async def parallel_reasoning(self, state: ChamberState) -> ChamberState:
        """Multiple models reason about the problem in parallel.

        Args:
            state: Current state

        Returns:
            Updated state with reasoning responses
        """
        logger.info(f"Parallel reasoning with {len(self.reasoning_models)} models")

        system_prompt = SystemMessage(
            content=f"""You are participating in a multi-model reasoning chamber.

The orchestrator's plan:
{state["orchestrator_plan"]}

Provide your unique perspective and reasoning approach to address the user's request.
Focus on your strengths and offer insights that might differ from other models."""
        )

        messages = [system_prompt] + state["messages"]

        reasoning_responses = {}

        # Execute reasoning in parallel (async)

        tasks = []

        for model_info in self.reasoning_models:
            task = model_info["model"].ainvoke(messages)
            tasks.append((model_info["name"], task))

        # Gather responses
        for name, task in tasks:
            try:
                response = await task
                reasoning_responses[name] = response.content
                logger.info(f"Got response from {name}")
            except Exception as e:
                logger.error(f"Error getting response from {name}: {e}")
                reasoning_responses[name] = f"Error: {str(e)}"

        state["reasoning_responses"] = reasoning_responses

        # Add all reasoning responses to messages
        for name, content in reasoning_responses.items():
            state["messages"].append(AIMessage(content=f"[{name.upper()}]\n{content}"))

        return state

    async def orchestrator_synthesis(self, state: ChamberState) -> ChamberState:
        """Orchestrator synthesizes reasoning from all models.

        Args:
            state: Current state

        Returns:
            Updated state with synthesis and next action decision
        """
        logger.info("Orchestrator synthesis phase")

        # Build synthesis prompt with all reasoning responses
        reasoning_summary = "\n\n".join(
            [f"**{name}**:\n{response}" for name, response in state["reasoning_responses"].items()]
        )

        system_prompt = SystemMessage(
            content=f"""You are the orchestrator synthesizing insights from multiple AI models.

REASONING FROM MODELS:
{reasoning_summary}

Your tasks:
1. Synthesize the key insights and reasoning approaches
2. Identify consensus and disagreements
3. Decide the next action:
   - "continue_reasoning" - Need more deliberation
   - "use_tools" - Need to use tools (web search, calculator, file ops)
   - "finalize" - Ready to provide final answer

Provide your synthesis and clearly state your decision at the end."""
        )

        messages = [system_prompt] + [state["messages"][0]]  # Include original user message

        response = await self.orchestrator.ainvoke(messages)

        synthesis = response.content
        state["orchestrator_synthesis"] = synthesis

        # Parse next action from synthesis
        synthesis_lower = synthesis.lower()
        if "finalize" in synthesis_lower or "final answer" in synthesis_lower:
            state["next_action"] = "finalize"
        elif (
            "tool" in synthesis_lower
            or "search" in synthesis_lower
            or "calculate" in synthesis_lower
        ):
            state["next_action"] = "use_tools"
        else:
            state["next_action"] = "continue_reasoning"

        state["messages"].append(AIMessage(content=f"[SYNTHESIS]\n{synthesis}"))
        state["iteration_count"] = state.get("iteration_count", 0) + 1

        logger.info(f"Next action: {state['next_action']}, iteration: {state['iteration_count']}")

        return state

    def route_next_action(
        self, state: ChamberState
    ) -> Literal["parallel_reasoning", "tool_approval", "finalize", "end"]:
        """Route to next node based on orchestrator decision.

        Args:
            state: Current state

        Returns:
            Next node name
        """
        # Prevent infinite loops
        if state.get("iteration_count", 0) >= 10:
            logger.warning("Max iterations reached, finalizing")
            return "finalize"

        next_action = state.get("next_action", "finalize")

        if next_action == "continue_reasoning":
            return "parallel_reasoning"
        elif next_action == "use_tools":
            return "tool_approval"
        else:
            return "finalize"

    async def tool_approval(self, state: ChamberState) -> ChamberState:
        """Request human approval for tool usage.

        Args:
            state: Current state

        Returns:
            Updated state with approval request
        """
        logger.info("Tool approval required")

        # In a real implementation, this would pause and wait for approval
        # For now, we'll simulate approval
        state["pending_tool_approval"] = {
            "tool_name": "web_search",
            "parameters": {"query": "example query"},
            "reasoning": "Need to search for current information",
        }

        return state

    async def finalize(self, state: ChamberState) -> ChamberState:
        """Generate final response.

        Args:
            state: Current state

        Returns:
            Final state with complete response
        """
        logger.info("Finalizing response")

        system_prompt = SystemMessage(
            content="""Based on all the reasoning and synthesis, provide a clear, comprehensive final answer to the user's request.

Synthesize the best insights from all models into a coherent response."""
        )

        # Anthropic API treats a trailing AIMessage as "continue this turn" and
        # may return empty content. Add a closing HumanMessage so the model
        # generates a proper new response.
        messages = (
            [system_prompt]
            + state["messages"]
            + [
                HumanMessage(
                    content="Please provide your final answer to the user based on the above reasoning."
                )
            ]
        )

        response = await self.orchestrator.ainvoke(messages)
        logger.info(
            f"Finalize response: type={type(response.content).__name__}, value={response.content!r}"
        )

        # response.content may be a list of content blocks — extract text
        raw = response.content
        if isinstance(raw, list):
            text = " ".join(
                block["text"] if isinstance(block, dict) else getattr(block, "text", "")
                for block in raw
                if (isinstance(block, dict) and block.get("type") == "text")
                or hasattr(block, "text")
            )
        else:
            text = str(raw)

        state["messages"].append(AIMessage(content=f"[FINAL ANSWER]\n{text}"))

        return state

    def build(self) -> StateGraph:
        """Build the LangGraph workflow.

        Returns:
            Compiled StateGraph
        """
        workflow = StateGraph(ChamberState)

        # Add nodes
        workflow.add_node("orchestrator_planning", self.orchestrator_planning)
        workflow.add_node("parallel_reasoning", self.parallel_reasoning)
        workflow.add_node("orchestrator_synthesis", self.orchestrator_synthesis)
        workflow.add_node("tool_approval", self.tool_approval)
        workflow.add_node("finalize", self.finalize)

        # Set entry point
        workflow.set_entry_point("orchestrator_planning")

        # Add edges
        workflow.add_edge("orchestrator_planning", "parallel_reasoning")
        workflow.add_edge("parallel_reasoning", "orchestrator_synthesis")

        # Conditional routing from synthesis
        workflow.add_conditional_edges(
            "orchestrator_synthesis",
            self.route_next_action,
            {
                "parallel_reasoning": "parallel_reasoning",
                "tool_approval": "tool_approval",
                "finalize": "finalize",
            },
        )

        # Tool approval leads to finalize for now (will be enhanced later)
        workflow.add_edge("tool_approval", "finalize")

        # Finalize is the end
        workflow.add_edge("finalize", END)

        return workflow.compile()
