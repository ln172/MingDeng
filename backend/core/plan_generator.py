"""
Plan Generator Module
Handles AI-powered learning plan generation
"""

import uuid
from typing import Dict, Any
from datetime import datetime, timedelta, date
from .ai import ai_client
from .todo_manager import todo_manager


class PlanGenerator:
    """Learning plan generator using AI"""

    def __init__(self):
        self.ai_client = ai_client
        self.todo_manager = todo_manager

    async def generate_plan(self, user_input: str, start_date: str = None) -> Dict[str, Any]:
        """
        Generate a learning plan from user input

        Args:
            user_input: User's learning goals/topics
            start_date: Optional start date (defaults to today)

        Returns:
            Generated plan with tasks
        """
        # Get AI-generated plan structure
        plan_data = await self.ai_client.generate_plan(user_input)

        # Adjust dates if start_date is provided
        if start_date:
            plan_data = self._adjust_plan_dates(plan_data, start_date)
        else:
            # Default: start from today
            plan_data = self._adjust_plan_dates(plan_data, datetime.now().date().isoformat())

        # Create the plan in storage
        created_plan = self.todo_manager.create_plan(
            name=plan_data.get("plan_name", "新学习计划"),
            tasks=plan_data.get("tasks", [])
        )

        return {
            "success": True,
            "plan": created_plan,
            "message": f"成功生成学习计划：{created_plan['name']}，包含 {len(created_plan['tasks'])} 个任务"
        }

    def _adjust_plan_dates(self, plan_data: Dict[str, Any], start_date: str) -> Dict[str, Any]:
        """
        Adjust task dates to start from a specific date

        Args:
            plan_data: Plan data from AI
            start_date: Start date (ISO format)

        Returns:
            Plan data with adjusted dates
        """
        try:
            base_date = datetime.fromisoformat(start_date).date()
        except:
            base_date = datetime.now().date()

        tasks = plan_data.get("tasks", [])

        # If tasks already have dates, calculate offset
        if tasks and "date" in tasks[0]:
            try:
                first_task_date = datetime.fromisoformat(tasks[0]["date"]).date()
                offset = (base_date - first_task_date).days

                # Apply offset to all tasks
                for task in tasks:
                    if "date" in task:
                        task_date = datetime.fromisoformat(task["date"]).date()
                        new_date = task_date + timedelta(days=offset)
                        task["date"] = new_date.isoformat()
            except:
                # If date parsing fails, use sequential dates
                self._assign_sequential_dates(tasks, base_date)
        else:
            # No dates provided, assign sequential dates
            self._assign_sequential_dates(tasks, base_date)

        return plan_data

    def _assign_sequential_dates(self, tasks: list, start_date: date):
        """
        Assign sequential dates to tasks

        Args:
            tasks: List of tasks
            start_date: Starting date
        """
        current_date = start_date
        daily_time = 0  # Track daily time in minutes

        for task in tasks:
            estimated_time = task.get("estimated_time", 60)

            # If adding this task exceeds 4 hours, move to next day
            if daily_time + estimated_time > 240:  # 4 hours = 240 minutes
                current_date += timedelta(days=1)
                daily_time = 0

            task["date"] = current_date.isoformat()
            daily_time += estimated_time

    async def regenerate_plan(self, plan_id: str, modifications: str) -> Dict[str, Any]:
        """
        Regenerate a plan with modifications

        Args:
            plan_id: Existing plan ID
            modifications: User's modification requests

        Returns:
            Updated plan
        """
        # Get existing plan
        existing_plan = self.todo_manager.get_plan(plan_id)
        if not existing_plan:
            return {
                "success": False,
                "message": "计划不存在"
            }

        # Generate prompt for modification
        prompt = f"""
现有学习计划：{existing_plan['name']}
包含 {len(existing_plan['tasks'])} 个任务

用户的修改要求：
{modifications}

请根据用户要求调整学习计划。保持原有任务的合理部分，根据要求进行调整。

输出 JSON 格式（与原格式相同）：
{{
  "plan_name": "调整后的计划名称",
  "tasks": [...]
}}
"""

        # Get AI-generated modifications
        plan_data = await self.ai_client.generate_plan(prompt)

        # Update the existing plan
        existing_plan["name"] = plan_data.get("plan_name", existing_plan["name"])
        existing_plan["tasks"] = plan_data.get("tasks", existing_plan["tasks"])

        # Ensure all tasks have IDs
        for task in existing_plan["tasks"]:
            if "id" not in task:
                task["id"] = str(uuid.uuid4())

        # Save updated plan
        self.todo_manager.update_plan(plan_id, existing_plan)

        return {
            "success": True,
            "plan": existing_plan,
            "message": "计划已更新"
        }

    async def reschedule_tasks(self, mode: str = "from_today", plan_id: str = None) -> Dict[str, Any]:
        """
        Reschedule tasks using AI with memory-based optimization

        Args:
            mode: Reschedule mode - "from_today" or "include_incomplete"
            plan_id: Optional specific plan ID to reschedule

        Returns:
            Rescheduled plan result
        """
        from .memory import memory_manager

        # Get all tasks
        all_tasks = self.todo_manager.get_all_tasks()
        today = datetime.now().date()
        today_str = today.isoformat()

        # Filter tasks based on mode
        if mode == "from_today":
            # Get tasks from today onwards
            tasks_to_reschedule = [
                task for task in all_tasks
                if task.get("date", "") >= today_str and task.get("status") != "completed"
            ]
            mode_description = "从今天开始重新安排任务"
        elif mode == "include_incomplete":
            # Get all incomplete tasks (including overdue)
            tasks_to_reschedule = [
                task for task in all_tasks
                if task.get("status") != "completed"
            ]
            mode_description = "重新安排所有未完成任务（包括过期任务）"
        else:
            return {
                "success": False,
                "message": f"不支持的模式: {mode}"
            }

        if not tasks_to_reschedule:
            return {
                "success": False,
                "message": "没有需要重新安排的任务"
            }

        # Get user's learning statistics for AI context
        stats = self.todo_manager.get_stats()

        # Search relevant memories about user's learning pace and preferences
        memory_context = memory_manager.get_context_for_chat(
            f"用户的学习速度和完成情况，已完成{stats['completed']}个任务，完成率{stats['completion_rate']}%"
        )

        # Build AI prompt with memory context
        prompt = f"""
你是 MingDeng 学习规划助手。需要根据用户的实际学习进度重新安排任务。

**用户学习统计**：
- 总任务数：{stats['total_tasks']}
- 已完成：{stats['completed']}
- 完成率：{stats['completion_rate']}%
- 简单任务完成：{stats['by_difficulty']['simple']['completed']}/{stats['by_difficulty']['simple']['total']}
- 中等任务完成：{stats['by_difficulty']['medium']['completed']}/{stats['by_difficulty']['medium']['total']}
- 困难任务完成：{stats['by_difficulty']['hard']['completed']}/{stats['by_difficulty']['hard']['total']}

**历史学习记忆**：
{memory_context}

**重新安排模式**：{mode_description}

**待重新安排的任务**（共{len(tasks_to_reschedule)}个）：
{self._format_tasks_for_prompt(tasks_to_reschedule)}

**要求**：
1. 根据用户的完成率和历史表现调整任务安排：
   - 如果完成率高（>80%），可以适当增加难度或密度
   - 如果完成率低（<50%），应该降低难度，延长时间，给予更多缓冲
2. 保持任务的依赖关系和学习顺序
3. 从今天（{today_str}）开始重新分配日期
4. 每日总时长控制在 2-4 小时
5. 合理分散高难度任务
6. 保留任务的原始ID

输出 JSON 格式：
{{
  "plan_name": "重新安排的学习计划",
  "tasks": [
    {{
      "id": "原任务ID",
      "task": "任务描述",
      "date": "YYYY-MM-DD",
      "estimated_time": 分钟数,
      "difficulty": "simple|medium|hard",
      "priority": "high|medium|low",
      "tags": ["标签"],
      "status": "pending"
    }}
  ],
  "adjustment_reason": "简要说明调整的原因（根据用户的学习速度和完成情况）"
}}

只返回 JSON，不要其他文字。
"""

        # Get AI rescheduling — call chat() directly to avoid wrapping inside generate_plan's template
        try:
            messages = [
                {"role": "system", "content": "你是 MingDeng 学习规划助手，擅长根据用户学习进度智能重新安排任务。"},
                {"role": "user", "content": prompt},
            ]
            response = await self.ai_client.chat(messages, temperature=0.5)
            plan_data = self.ai_client._extract_json(response)

            # Update tasks in storage
            rescheduled_tasks = plan_data.get("tasks", [])

            # Update each task
            for new_task in rescheduled_tasks:
                task_id = new_task.get("id")
                if task_id:
                    # Preserve original task data, only update date and related fields
                    updates = {
                        "date": new_task.get("date"),
                        "estimated_time": new_task.get("estimated_time"),
                        "difficulty": new_task.get("difficulty"),
                        "priority": new_task.get("priority"),
                    }
                    self.todo_manager.update_task(task_id, updates)

            # Save adjustment reason to memory
            adjustment_reason = plan_data.get("adjustment_reason", "根据学习进度重新安排了任务")
            memory_manager.add_message(
                "system",
                f"任务重新安排：{adjustment_reason}。重新安排了{len(rescheduled_tasks)}个任务。",
                metadata={"action": "reschedule", "mode": mode}
            )

            return {
                "success": True,
                "rescheduled_count": len(rescheduled_tasks),
                "adjustment_reason": adjustment_reason,
                "message": f"成功重新安排 {len(rescheduled_tasks)} 个任务"
            }
        except Exception as e:
            return {
                "success": False,
                "message": f"重新安排失败: {str(e)}"
            }

    def _format_tasks_for_prompt(self, tasks: list) -> str:
        """Format tasks for AI prompt"""
        lines = []
        for i, task in enumerate(tasks, 1):
            lines.append(
                f"{i}. [{task.get('difficulty', 'medium')}] {task.get('task')} "
                f"({task.get('estimated_time', 60)}分钟) - 原定日期: {task.get('date', 'N/A')}"
            )
        return "\n".join(lines)


# Global plan generator instance
plan_generator = PlanGenerator()
