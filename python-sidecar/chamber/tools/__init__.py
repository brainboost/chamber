"""Chamber Tools."""

from chamber.tools.base import ChamberTool
from chamber.tools.web_search import WebSearchTool
from chamber.tools.calculator import CalculatorTool
from chamber.tools.file_ops import FileOpsTool

__all__ = [
    "ChamberTool",
    "WebSearchTool",
    "CalculatorTool",
    "FileOpsTool",
]


def get_tools(enabled_tools: list[str], workspace_path: str | None = None) -> list[ChamberTool]:
    """Get enabled tools.

    Args:
        enabled_tools: List of tool names to enable
        workspace_path: Path to workspace (required for file_ops)

    Returns:
        List of ChamberTool instances
    """
    available_tools = {
        "web_search": WebSearchTool,
        "calculator": CalculatorTool,
    }

    # Add file_ops if workspace_path provided
    if workspace_path:
        available_tools["file_ops"] = lambda: FileOpsTool(workspace_path)

    tools = []
    for tool_name in enabled_tools:
        if tool_name in available_tools:
            tool_class = available_tools[tool_name]
            tools.append(tool_class() if callable(tool_class) else tool_class)

    return tools
