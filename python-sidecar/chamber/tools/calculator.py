"""Calculator Tool."""

import ast
import operator

from langchain_core.tools import Tool

from chamber.tools.base import ChamberTool


class CalculatorTool(ChamberTool):
    """Calculator tool for mathematical operations."""

    # Safe operators mapping
    OPERATORS = {
        ast.Add: operator.add,
        ast.Sub: operator.sub,
        ast.Mult: operator.mul,
        ast.Div: operator.truediv,
        ast.Pow: operator.pow,
        ast.Mod: operator.mod,
        ast.USub: operator.neg,
    }

    def _safe_eval(self, node):
        """Safely evaluate AST node.

        Args:
            node: AST node

        Returns:
            Evaluated result

        Raises:
            ValueError: If expression contains unsafe operations
        """
        if isinstance(node, ast.Num):
            return node.n
        elif isinstance(node, ast.BinOp):
            left = self._safe_eval(node.left)
            right = self._safe_eval(node.right)
            op = self.OPERATORS.get(type(node.op))
            if op is None:
                raise ValueError(f"Unsupported operator: {type(node.op).__name__}")
            return op(left, right)
        elif isinstance(node, ast.UnaryOp):
            operand = self._safe_eval(node.operand)
            op = self.OPERATORS.get(type(node.op))
            if op is None:
                raise ValueError(f"Unsupported operator: {type(node.op).__name__}")
            return op(operand)
        else:
            raise ValueError(f"Unsupported expression: {type(node).__name__}")

    def _calculate(self, expression: str) -> str:
        """Evaluate mathematical expression.

        Args:
            expression: Mathematical expression to evaluate

        Returns:
            Result as string
        """
        try:
            # Parse expression into AST
            tree = ast.parse(expression, mode='eval')

            # Evaluate using safe AST traversal
            result = self._safe_eval(tree.body)
            return str(result)
        except Exception as e:
            return f"Error evaluating expression: {str(e)}"

    def get_tool(self) -> Tool:
        """Get LangChain tool instance."""
        return Tool(
            name=self.name,
            description=self.description,
            func=self._calculate,
        )

    @property
    def name(self) -> str:
        """Get tool name."""
        return "calculator"

    @property
    def description(self) -> str:
        """Get tool description."""
        return (
            "Perform mathematical calculations. "
            "Input should be a mathematical expression as a string. "
            "Supports basic arithmetic operations: +, -, *, /, ** (power), % (modulo). "
            "Example: '2 + 2' or '2 ** 3 + 5'"
        )
