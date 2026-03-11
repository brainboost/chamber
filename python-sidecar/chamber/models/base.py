"""Base LLM Provider Interface."""

from abc import ABC, abstractmethod
from typing import Any

from langchain_core.language_models import BaseChatModel


class BaseLLMProvider(ABC):
    """Base class for LLM providers."""

    def __init__(self, model: str, temperature: float = 0.7, max_tokens: int | None = None):
        """Initialize provider.

        Args:
            model: Model identifier
            temperature: Sampling temperature
            max_tokens: Maximum tokens to generate
        """
        self.model = model
        self.temperature = temperature
        self.max_tokens = max_tokens

    @abstractmethod
    def get_model(self) -> BaseChatModel:
        """Get the LangChain chat model instance.

        Returns:
            BaseChatModel instance
        """
        pass

    @property
    def name(self) -> str:
        """Get provider name."""
        return self.__class__.__name__.replace("Provider", "").lower()
