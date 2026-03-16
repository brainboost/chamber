"""File Operations Tool."""

from pathlib import Path

from langchain_core.tools import Tool

from chamber.tools.base import ChamberTool


class FileOpsTool(ChamberTool):
    """File operations tool for reading and writing files."""

    def __init__(self, workspace_path: str):
        """Initialize file operations tool.

        Args:
            workspace_path: Path to workspace directory (for safety)
        """
        self.workspace_path = Path(workspace_path)

    def _is_safe_path(self, path: str) -> bool:
        """Check if path is within workspace (safety check).

        Args:
            path: Path to check

        Returns:
            True if path is safe, False otherwise
        """
        try:
            full_path = self.workspace_path / path
            full_path.resolve().relative_to(self.workspace_path.resolve())
            return True
        except (ValueError, RuntimeError):
            return False

    def _file_operation(self, operation: str) -> str:
        """Perform file operation.

        Args:
            operation: Operation string in format "action:path[:content]"

        Returns:
            Result as string
        """
        try:
            parts = operation.split(":", 2)
            action = parts[0].lower()
            path = parts[1] if len(parts) > 1 else ""

            if not self._is_safe_path(path):
                return "Error: Path is outside workspace directory"

            full_path = self.workspace_path / path

            if action == "read":
                if not full_path.exists():
                    return f"Error: File does not exist: {path}"
                return full_path.read_text()

            elif action == "write":
                content = parts[2] if len(parts) > 2 else ""
                full_path.parent.mkdir(parents=True, exist_ok=True)
                full_path.write_text(content)
                return f"Successfully wrote to {path}"

            elif action == "list":
                if not full_path.exists():
                    return f"Error: Directory does not exist: {path}"
                if not full_path.is_dir():
                    return f"Error: Path is not a directory: {path}"

                items = [item.name for item in full_path.iterdir()]
                return "\n".join(items)

            else:
                return f"Error: Unknown action '{action}'. Use 'read', 'write', or 'list'"

        except Exception as e:
            return f"Error performing file operation: {str(e)}"

    def get_tool(self) -> Tool:
        """Get LangChain tool instance."""
        return Tool(
            name=self.name,
            description=self.description,
            func=self._file_operation,
        )

    @property
    def name(self) -> str:
        """Get tool name."""
        return "file_ops"

    @property
    def description(self) -> str:
        """Get tool description."""
        return (
            "Perform file operations within the workspace. "
            "Input format: 'action:path[:content]' where action is 'read', 'write', or 'list'. "
            "Examples: "
            "'read:data.txt' - Read file contents, "
            "'write:output.txt:Hello World' - Write content to file, "
            "'list:subdir' - List directory contents. "
            "All paths are relative to workspace directory."
        )
