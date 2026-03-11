"""LLM Model Providers."""

from chamber.models.base import BaseLLMProvider
from chamber.models.anthropic import AnthropicProvider
from chamber.models.gemini import GeminiProvider
from chamber.models.ollama import OllamaProvider
from chamber.models.xai import XAIProvider

__all__ = [
    "BaseLLMProvider",
    "AnthropicProvider",
    "GeminiProvider",
    "OllamaProvider",
    "XAIProvider",
]


def get_provider(
    provider_name: str, model: str, temperature: float = 0.7, max_tokens: int | None = None
) -> BaseLLMProvider:
    """Get provider instance by name.

    Args:
        provider_name: Provider name (anthropic, gemini, ollama, xai)
        model: Model identifier
        temperature: Sampling temperature
        max_tokens: Maximum tokens to generate

    Returns:
        BaseLLMProvider instance

    Raises:
        ValueError: If provider name is unknown
    """
    providers = {
        "anthropic": AnthropicProvider,
        "gemini": GeminiProvider,
        "ollama": OllamaProvider,
        "xai": XAIProvider,
    }

    provider_class = providers.get(provider_name.lower())
    if not provider_class:
        raise ValueError(f"Unknown provider: {provider_name}")

    return provider_class(model, temperature, max_tokens)
