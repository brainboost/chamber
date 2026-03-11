"""Ollama Local Provider."""

import os
from langchain_community.chat_models import ChatOllama
from langchain_core.language_models import BaseChatModel

from chamber.models.base import BaseLLMProvider


class OllamaProvider(BaseLLMProvider):
    """Ollama local provider."""

    def __init__(self, model: str, temperature: float = 0.7, max_tokens: int | None = None):
        """Initialize Ollama provider."""
        super().__init__(model, temperature, max_tokens)

        # Get base URL from environment or use default
        self.base_url = os.getenv("OLLAMA_BASE_URL", "http://localhost:11434")

    def get_model(self) -> BaseChatModel:
        """Get ChatOllama model instance."""
        return ChatOllama(
            model=self.model,
            temperature=self.temperature,
            num_predict=self.max_tokens,
            base_url=self.base_url,
        )

    @property
    def name(self) -> str:
        """Get provider name."""
        return "ollama"
