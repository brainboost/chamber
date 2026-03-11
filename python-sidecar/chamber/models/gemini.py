"""Google Gemini Provider."""

import os
from langchain_google_genai import ChatGoogleGenerativeAI
from langchain_core.language_models import BaseChatModel

from chamber.models.base import BaseLLMProvider


class GeminiProvider(BaseLLMProvider):
    """Google Gemini provider."""

    def __init__(self, model: str, temperature: float = 0.7, max_tokens: int | None = None):
        """Initialize Gemini provider."""
        super().__init__(model, temperature, max_tokens)

        # Get API key from environment
        self.api_key = os.getenv("GOOGLE_API_KEY")
        if not self.api_key:
            raise ValueError("GOOGLE_API_KEY environment variable not set")

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
