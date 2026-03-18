"""
Memory Module
Handles Mem0 integration for AI memory system
"""

from typing import List, Dict, Any, Optional
from .config import config_manager
from .paths import get_data_dir

# Mem0 integration (optional, gracefully handle if not available)
try:
    from mem0 import Memory
    MEM0_AVAILABLE = True
except ImportError:
    MEM0_AVAILABLE = False
    print("Warning: mem0ai not installed. Memory features will be limited.")


class MemoryManager:
    """Memory Manager using Mem0"""

    def __init__(self):
        self.memory: Optional[Any] = None
        self.user_id = "default_user"
        self._initialize_memory()

    def _initialize_memory(self):
        """Initialize Mem0 memory"""
        if not MEM0_AVAILABLE:
            print("Mem0 not available, using fallback memory")
            return

        config = config_manager.get_config()
        if not config.api.api_key or not config.api.base_url:
            return

        try:
            mem_config = {
                "llm": {
                    "provider": "openai",
                    "config": {
                        "model": config.api.model,
                        "api_key": config.api.api_key,
                        "openai_base_url": config.api.base_url,
                    }
                },
                "embedder": {
                    "provider": "openai",
                    "config": {
                        "model": "text-embedding-ada-002",
                        "api_key": config.api.api_key,
                        "openai_base_url": config.api.base_url,
                    }
                },
                "vector_store": {
                    "provider": "qdrant",
                    "config": {
                        "collection_name": "mingdeng_memory",
                        "path": str(get_data_dir() / "memory" / "qdrant")
                    }
                }
            }

            self.memory = Memory.from_config(mem_config)
        except Exception as e:
            print(f"Error initializing Mem0: {e}")
            print("Memory features will be limited. To enable full memory, ensure Mem0 is properly configured.")
            self.memory = None

    def refresh_memory(self):
        """Re-initialize memory with current config (call after API config changes)"""
        self.memory = None
        self._initialize_memory()

    def add_message(self, role: str, content: str, metadata: Optional[Dict[str, Any]] = None) -> bool:
        """
        Add a chat message to memory

        Args:
            role: Message role (user/assistant)
            content: Message content
            metadata: Additional metadata

        Returns:
            Success status
        """
        if not self.memory:
            return False

        try:
            message = f"{role}: {content}"
            self.memory.add(message, user_id=self.user_id, metadata=metadata or {})
            return True
        except Exception as e:
            print(f"Error adding message to memory: {e}")
            return False

    def search_memories(self, query: str, limit: int = 5) -> List[Dict[str, Any]]:
        """
        Search relevant memories

        Args:
            query: Search query
            limit: Maximum number of results

        Returns:
            List of relevant memories
        """
        if not self.memory:
            return []

        try:
            results = self.memory.search(query, user_id=self.user_id, limit=limit)
            return results
        except Exception as e:
            print(f"Error searching memories: {e}")
            return []

    def get_all_memories(self) -> List[Dict[str, Any]]:
        """
        Get all memories for the user

        Returns:
            List of all memories
        """
        if not self.memory:
            return []

        try:
            memories = self.memory.get_all(user_id=self.user_id)
            return memories
        except Exception as e:
            print(f"Error getting memories: {e}")
            return []

    def get_context_for_chat(self, current_message: str, limit: int = 3) -> str:
        """
        Get relevant context for a chat message

        Args:
            current_message: Current user message
            limit: Number of relevant memories to retrieve

        Returns:
            Formatted context string
        """
        memories = self.search_memories(current_message, limit=limit)

        if not memories:
            return "这是你们的首次对话。"

        context_parts = ["根据之前的对话记录："]
        for i, memory in enumerate(memories, 1):
            memory_text = memory.get("memory", memory.get("text", ""))
            context_parts.append(f"{i}. {memory_text}")

        return "\n".join(context_parts)

    def clear_memories(self) -> bool:
        """
        Clear all memories (use with caution)

        Returns:
            Success status
        """
        if not self.memory:
            return False

        try:
            self.memory.delete_all(user_id=self.user_id)
            return True
        except Exception as e:
            print(f"Error clearing memories: {e}")
            return False


# Global memory manager instance
memory_manager = MemoryManager()
