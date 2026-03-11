"""Anthropic Claude Provider."""

import os

from langchain_anthropic import ChatAnthropic
from langchain_core.language_models import BaseChatModel

from chamber.models.base import BaseLLMProvider


class AnthropicProvider(BaseLLMProvider):
    """Anthropic Claude provider."""

    def __init__(self, model: str, temperature: float = 0.7, max_tokens: int | None = None):
        """Initialize Anthropic provider."""
        super().__init__(model, temperature, max_tokens)

        # Get API key from environment
        self.api_key = os.getenv("ANTHROPIC_API_KEY")
        if not self.api_key:
            raise ValueError("ANTHROPIC_API_KEY environment variable not set")

    def get_model(self) -> BaseChatModel:
        """Get ChatAnthropic model instance."""
        return ChatAnthropic(
            model_name=self.model,
            temperature=self.temperature,
            max_tokens=self.max_tokens or 4096,
            # api_key=self.api_key,
        )

    @property
    def name(self) -> str:
        """Get provider name."""
        return "anthropic"
