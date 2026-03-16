"""Base Tool Interface."""

from abc import ABC, abstractmethod

from langchain_core.tools import BaseTool


class ChamberTool(ABC):
    """Base class for Chamber tools."""

    @abstractmethod
    def get_tool(self) -> BaseTool:
        """Get the LangChain tool instance.

        Returns:
            BaseTool instance
        """
        pass

    @property
    @abstractmethod
    def name(self) -> str:
        """Get tool name."""
        pass

    @property
    @abstractmethod
    def description(self) -> str:
        """Get tool description."""
        pass
