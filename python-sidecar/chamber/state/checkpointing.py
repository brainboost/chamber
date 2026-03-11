"""Checkpointing for LangGraph state persistence."""

import json
import logging
from pathlib import Path
from typing import Any

from langgraph.checkpoint import BaseCheckpointSaver
from langgraph.checkpoint.base import Checkpoint, CheckpointMetadata

logger = logging.getLogger(__name__)


class FileCheckpointSaver(BaseCheckpointSaver):
    """Save checkpoints to local filesystem."""

    def __init__(self, checkpoint_dir: str | Path):
        """Initialize checkpoint saver.

        Args:
            checkpoint_dir: Directory to save checkpoints
        """
        self.checkpoint_dir = Path(checkpoint_dir)
        self.checkpoint_dir.mkdir(parents=True, exist_ok=True)

    def _get_checkpoint_path(self, thread_id: str) -> Path:
        """Get path for checkpoint file."""
        return self.checkpoint_dir / f"{thread_id}.json"

    async def aput(
        self,
        config: dict,
        checkpoint: Checkpoint,
        metadata: CheckpointMetadata,
    ) -> dict:
        """Save checkpoint asynchronously."""
        thread_id = config.get("configurable", {}).get("thread_id")
        if not thread_id:
            logger.warning("No thread_id in config, skipping checkpoint save")
            return config

        checkpoint_path = self._get_checkpoint_path(thread_id)

        checkpoint_data = {
            "checkpoint": checkpoint,
            "metadata": metadata,
            "config": config,
        }

        try:
            with open(checkpoint_path, "w") as f:
                json.dump(checkpoint_data, f, indent=2, default=str)
            logger.info(f"Saved checkpoint for thread {thread_id}")
        except Exception as e:
            logger.error(f"Failed to save checkpoint: {e}")

        return config

    async def aget(self, config: dict) -> Checkpoint | None:
        """Load checkpoint asynchronously."""
        thread_id = config.get("configurable", {}).get("thread_id")
        if not thread_id:
            return None

        checkpoint_path = self._get_checkpoint_path(thread_id)

        if not checkpoint_path.exists():
            logger.info(f"No checkpoint found for thread {thread_id}")
            return None

        try:
            with open(checkpoint_path, "r") as f:
                checkpoint_data = json.load(f)
            logger.info(f"Loaded checkpoint for thread {thread_id}")
            return checkpoint_data.get("checkpoint")
        except Exception as e:
            logger.error(f"Failed to load checkpoint: {e}")
            return None

    async def alist(self, config: dict) -> list[Checkpoint]:
        """List all checkpoints for a thread."""
        thread_id = config.get("configurable", {}).get("thread_id")
        if not thread_id:
            return []

        checkpoint_path = self._get_checkpoint_path(thread_id)

        if not checkpoint_path.exists():
            return []

        try:
            checkpoint = await self.aget(config)
            return [checkpoint] if checkpoint else []
        except Exception as e:
            logger.error(f"Failed to list checkpoints: {e}")
            return []
