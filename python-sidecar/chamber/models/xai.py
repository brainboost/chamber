"""x.ai Grok Provider."""

import os

from langchain_classic import ChatOpenAI
from langchain_core.language_models import BaseChatModel

from chamber.models.base import BaseLLMProvider


class XAIProvider(BaseLLMProvider):
    """x.ai Grok provider (using OpenAI-compatible API)."""

    def __init__(self, model: str, temperature: float = 0.7, max_tokens: int | None = None):
        """Initialize x.ai provider."""
        super().__init__(model, temperature, max_tokens)

        # Get API key from environment
        self.api_key = os.getenv("XAI_API_KEY")
        if not self.api_key:
            raise ValueError("XAI_API_KEY environment variable not set")

    def get_model(self) -> BaseChatModel:
        """Get ChatOpenAI model instance configured for x.ai."""
        return ChatOpenAI(
            model=self.model,
            temperature=self.temperature,
            max_tokens=self.max_tokens,
            api_key=self.api_key,
            base_url="https://api.x.ai/v1",
        )

    @property
    def name(self) -> str:
        """Get provider name."""
        return "xai"
