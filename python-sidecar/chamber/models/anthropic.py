"""Anthropic Claude Provider."""

import os

from langchain_anthropic import ChatAnthropic
from langchain_core.language_models import BaseChatModel

from chamber.models.base import BaseLLMProvider


class AnthropicProvider(BaseLLMProvider):
    """Anthropic Claude provider."""

    def __init__(
        self,
        model: str,
        temperature: float = 0.7,
        max_tokens: int | None = None,
        api_key: str | None = None,
        oauth_token: str | None = None,
    ):
        """Initialize Anthropic provider.

        Args:
            model: Model identifier
            temperature: Sampling temperature
            max_tokens: Maximum tokens to generate
            api_key: Optional API key (injected from Rust or from environment)
            oauth_token: Optional OAuth bearer token (injected from Rust or from environment)
        """
        super().__init__(model, temperature, max_tokens, api_key)

        # Prefer injected oauth_token, then ANTHROPIC_AUTH_TOKEN env var
        self.oauth_token = oauth_token or os.getenv("ANTHROPIC_AUTH_TOKEN")

        if not self.oauth_token:
            if not self.api_key:
                self.api_key = os.getenv("ANTHROPIC_API_KEY")
            if not self.api_key:
                raise ValueError("Neither ANTHROPIC_API_KEY nor ANTHROPIC_AUTH_TOKEN is set")

    def get_model(self) -> BaseChatModel:
        """Get ChatAnthropic model instance."""
        if self.oauth_token:
            # OAuth / subscription token:
            # Set ANTHROPIC_AUTH_TOKEN so the underlying anthropic SDK uses
            # Authorization: Bearer header (not x-api-key).
            os.environ["ANTHROPIC_AUTH_TOKEN"] = self.oauth_token
            os.environ.pop("ANTHROPIC_API_KEY", None)
            return ChatAnthropic(
                model_name=self.model,
                temperature=self.temperature,
                max_tokens=self.max_tokens or 4096,
                default_headers={"anthropic-beta": "oauth-2025-04-20"},
            )

        return ChatAnthropic(
            model_name=self.model,
            temperature=self.temperature,
            max_tokens=self.max_tokens or 4096,
            api_key=self.api_key,
        )

    @property
    def name(self) -> str:
        """Get provider name."""
        return "anthropic"
