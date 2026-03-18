"""
AI Module
Handles OpenAI-compatible API calls with streaming support
"""

import json
from typing import AsyncGenerator, List, Dict, Any, Optional
from openai import AsyncOpenAI
from .config import config_manager


class AIClient:
    """AI Client for OpenAI-compatible APIs"""

    def __init__(self):
        self.client: Optional[AsyncOpenAI] = None
        self._initialize_client()

    def _initialize_client(self):
        """Initialize OpenAI client"""
        config = config_manager.get_config()
        if config.api.api_key and config.api.base_url:
            self.client = AsyncOpenAI(
                api_key=config.api.api_key,
                base_url=config.api.base_url
            )

    def refresh_client(self):
        """Refresh client with updated config"""
        self._initialize_client()

    async def chat(self, messages: List[Dict[str, str]],
                   temperature: float = 0.7,
                   max_tokens: Optional[int] = None) -> str:
        """
        Send chat completion request (non-streaming)

        Args:
            messages: List of message dicts with 'role' and 'content'
            temperature: Sampling temperature
            max_tokens: Maximum tokens to generate

        Returns:
            Generated text response
        """
        if not self.client:
            raise ValueError("AI client not configured. Please set API credentials.")

        config = config_manager.get_config()

        try:
            response = await self.client.chat.completions.create(
                model=config.api.model,
                messages=messages,
                temperature=temperature,
                max_tokens=max_tokens
            )
            return response.choices[0].message.content
        except Exception as e:
            raise Exception(f"AI API Error: {str(e)}")

    async def chat_stream(self, messages: List[Dict[str, str]],
                         temperature: float = 0.7,
                         max_tokens: Optional[int] = None) -> AsyncGenerator[str, None]:
        """
        Send streaming chat completion request

        Args:
            messages: List of message dicts with 'role' and 'content'
            temperature: Sampling temperature
            max_tokens: Maximum tokens to generate

        Yields:
            Text chunks as they arrive
        """
        if not self.client:
            raise ValueError("AI client not configured. Please set API credentials.")

        config = config_manager.get_config()

        try:
            stream = await self.client.chat.completions.create(
                model=config.api.model,
                messages=messages,
                temperature=temperature,
                max_tokens=max_tokens,
                stream=True
            )

            async for chunk in stream:
                if chunk.choices[0].delta.content:
                    yield chunk.choices[0].delta.content
        except Exception as e:
            raise Exception(f"AI API Error: {str(e)}")

    def _extract_json(self, response: str) -> Dict[str, Any]:
        """Extract and parse JSON from a model response, stripping any code fences."""
        if "```json" in response:
            json_str = response.split("```json")[1].split("```")[0].strip()
        elif "```" in response:
            json_str = response.split("```")[1].split("```")[0].strip()
        else:
            json_str = response.strip()
        return json.loads(json_str)

    async def generate_plan(self, user_input: str) -> Dict[str, Any]:
        """
        Generate learning plan from user input

        Args:
            user_input: User's learning goals/topics

        Returns:
            Parsed JSON plan
        """
        prompt = f"""
你是 MingDeng 学习规划助手。请严格根据用户的输入生成学习计划，不要添加用户未提及的内容。

用户输入：
{user_input}

要求：
1. 严格基于用户输入：只将用户明确提到的学习内容拆解为任务，不要自行添加额外内容
2. 尊重用户意图：保持用户的学习目标和范围，不擅自扩展
3. 任务拆解：每个任务 15min-3h，具体可执行
4. 评估难度：simple（简单概念/基础练习）/ medium（需要思考实践）/ hard（复杂概念/综合应用）
5. 时间安排：
   - 根据用户提到的时间要求安排（如果有）
   - 如果用户未指定，默认每日 2-3 小时
   - 基础内容优先，有依赖关系的按顺序排列

输出 JSON 格式：
{{
  "plan_name": "简洁的计划名称",
  "total_weeks": 2,
  "tasks": [
    {{
      "task": "具体任务描述",
      "date": "2025-10-10",
      "estimated_time": 90,
      "difficulty": "simple|medium|hard",
      "priority": "medium",
      "tags": ["相关标签"]
    }}
  ]
}}

重要：只返回 JSON，不要添加任何解释或额外文字。
"""

        messages = [
            {"role": "system", "content": "你是一个专业的学习规划助手，擅长分析知识依赖关系并制定科学的学习计划。"},
            {"role": "user", "content": prompt}
        ]

        response = await self.chat(messages, temperature=0.7)

        try:
            return self._extract_json(response)
        except Exception as e:
            raise Exception(f"Failed to parse AI response as JSON: {str(e)}\nResponse: {response}")

    async def auto_link_resource(self, resource_content: str,
                                 tasks: List[Dict[str, Any]]) -> Dict[str, Any]:
        """
        Automatically link resource to relevant tasks

        Args:
            resource_content: Resource URL or description
            tasks: List of available tasks

        Returns:
            Dict with linked_task_id and reason
        """
        tasks_summary = "\n".join([
            f"- ID: {task['id']}, Task: {task['task']}, Tags: {task.get('tags', [])}"
            for task in tasks[:20]  # Limit to 20 tasks to avoid token limits
        ])

        prompt = f"""
用户保存了以下学习资源：
{resource_content}

当前学习计划中的任务：
{tasks_summary}

请判断这个资源最适合关联到哪个任务（如果有）。

输出 JSON：
{{
  "linked_task_id": "uuid 或 null",
  "reason": "关联理由"
}}

只返回 JSON，不要其他文字。
"""

        messages = [
            {"role": "system", "content": "你是资源管理助手，帮助用户将学习资源关联到相关任务。"},
            {"role": "user", "content": prompt}
        ]

        response = await self.chat(messages, temperature=0.5)

        try:
            return self._extract_json(response)
        except Exception:
            return {"linked_task_id": None, "reason": "无法自动关联"}


# Global AI client instance
ai_client = AIClient()
