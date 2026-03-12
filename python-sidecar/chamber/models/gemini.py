"""Google Gemini Provider."""

import os
from langchain_google_genai import ChatGoogleGenerativeAI
from langchain_core.language_models import BaseChatModel

from chamber.models.base import BaseLLMProvider


class GeminiProvider(BaseLLMProvider):
    """Google Gemini provider."""

    def __init__(
        self,
        model: str,
        temperature: float = 0.7,
        max_tokens: int | None = None,
        api_key: str | None = None,
    ):
        """Initialize Gemini provider.

        Args:
            model: Model identifier
            temperature: Sampling temperature
            max_tokens: Maximum tokens to generate
            api_key: Optional API key (injected from Rust or from environment)
        """
        super().__init__(model, temperature, max_tokens, api_key)

        # Use injected api_key or fall back to environment variable
        if not self.api_key:
            self.api_key = os.getenv("GOOGLE_API_KEY")
            if not self.api_key:
                raise ValueError("GOOGLE_API_KEY not provided and not found in environment")

    def get_model(self) -> BaseChatModel:
        """Get ChatGoogleGenerativeAI model instance."""
        return ChatGoogleGenerativeAI(
            model=self.model,
            temperature=self.temperature,
            max_tokens=self.max_tokens,
            google_api_key=self.api_key,
        )

    @property
    def name(self) -> str:
        """Get provider name."""
        return "gemini"
