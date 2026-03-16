"""Web Search Tool."""

from langchain_core.tools import Tool
from langchain_community.utilities import DuckDuckGoSearchAPIWrapper

from chamber.tools.base import ChamberTool


class WebSearchTool(ChamberTool):
    """Web search tool using DuckDuckGo."""

    def __init__(self):
        """Initialize web search tool."""
        self.search = DuckDuckGoSearchAPIWrapper()

    def _search(self, query: str) -> str:
        """Perform web search.

        Args:
            query: Search query

        Returns:
            Search results as formatted string
        """
        try:
            results = self.search.run(query)
            return results
        except Exception as e:
            return f"Error performing search: {str(e)}"

    def get_tool(self) -> Tool:
        """Get LangChain tool instance."""
        return Tool(
            name=self.name,
            description=self.description,
            func=self._search,
        )

    @property
    def name(self) -> str:
        """Get tool name."""
        return "web_search"

    @property
    def description(self) -> str:
        """Get tool description."""
        return (
            "Search the web for current information. "
            "Input should be a search query string. "
            "Returns search results with snippets and URLs."
        )
