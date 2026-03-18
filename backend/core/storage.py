"""
Storage Module
Handles JSON file operations for todos and library
"""

import json
import os
import tempfile
from pathlib import Path
from typing import Dict, Any, List, Optional
from datetime import datetime
from .paths import get_data_dir


class Storage:
    """Generic JSON storage handler"""

    def __init__(self, file_path: str):
        self.file_path = Path(file_path)
        self._ensure_file()

    def _ensure_file(self):
        """Ensure file and directory exist"""
        self.file_path.parent.mkdir(parents=True, exist_ok=True)
        if not self.file_path.exists():
            self._write_data(self._get_default_data())

    def _get_default_data(self) -> Dict[str, Any]:
        """Get default data structure (override in subclasses)"""
        return {}

    def _read_data(self) -> Dict[str, Any]:
        """Read data from file"""
        try:
            with open(self.file_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception as e:
            print(f"Error reading {self.file_path}: {e}")
            return self._get_default_data()

    def _write_data(self, data: Dict[str, Any]) -> bool:
        """Write data to file atomically (write to temp file then replace)"""
        tmp_path = None
        try:
            dir_path = self.file_path.parent
            with tempfile.NamedTemporaryFile(
                mode='w', encoding='utf-8', dir=dir_path,
                delete=False, suffix='.tmp'
            ) as tmp:
                json.dump(data, tmp, indent=2, ensure_ascii=False)
                tmp_path = tmp.name
            os.replace(tmp_path, self.file_path)
            return True
        except Exception as e:
            print(f"Error writing {self.file_path}: {e}")
            if tmp_path:
                try:
                    os.unlink(tmp_path)
                except Exception:
                    pass
            return False

    def get_all(self) -> Dict[str, Any]:
        """Get all data"""
        return self._read_data()

    def save_all(self, data: Dict[str, Any]) -> bool:
        """Save all data"""
        return self._write_data(data)


class TodoStorage(Storage):
    """Todo storage handler"""

    def __init__(self, file_path: Optional[str] = None):
        data_dir = get_data_dir()
        super().__init__(file_path or data_dir / "todos.json")

    def _get_default_data(self) -> Dict[str, Any]:
        """Get default todos structure"""
        return {"plans": []}

    def get_plans(self) -> List[Dict[str, Any]]:
        """Get all plans"""
        data = self.get_all()
        return data.get("plans", [])

    def save_plans(self, plans: List[Dict[str, Any]]) -> bool:
        """Save all plans"""
        return self.save_all({"plans": plans})

    def add_plan(self, plan: Dict[str, Any]) -> bool:
        """Add a new plan"""
        plans = self.get_plans()
        plans.append(plan)
        return self.save_plans(plans)

    def get_plan_by_id(self, plan_id: str) -> Optional[Dict[str, Any]]:
        """Get plan by ID"""
        plans = self.get_plans()
        for plan in plans:
            if plan.get("id") == plan_id:
                return plan
        return None

    def update_plan(self, plan_id: str, updated_plan: Dict[str, Any]) -> bool:
        """Update a plan"""
        plans = self.get_plans()
        for i, plan in enumerate(plans):
            if plan.get("id") == plan_id:
                plans[i] = updated_plan
                return self.save_plans(plans)
        return False

    def delete_plan(self, plan_id: str) -> bool:
        """Delete a plan"""
        plans = self.get_plans()
        plans = [p for p in plans if p.get("id") != plan_id]
        return self.save_plans(plans)

    def get_tasks_by_date(self, date: str) -> List[Dict[str, Any]]:
        """Get all tasks for a specific date"""
        plans = self.get_plans()
        tasks = []
        for plan in plans:
            for task in plan.get("tasks", []):
                if task.get("date") == date:
                    tasks.append(task)
        return tasks

    def update_task(self, task_id: str, updated_fields: Dict[str, Any]) -> bool:
        """Update a task across all plans"""
        plans = self.get_plans()
        for plan in plans:
            for i, task in enumerate(plan.get("tasks", [])):
                if task.get("id") == task_id:
                    plan["tasks"][i].update(updated_fields)
                    return self.save_plans(plans)
        return False


class LibraryStorage(Storage):
    """Library storage handler"""

    def __init__(self, file_path: Optional[str] = None):
        data_dir = get_data_dir()
        super().__init__(file_path or data_dir / "library.json")

    def _get_default_data(self) -> Dict[str, Any]:
        """Get default library structure"""
        return {"resources": []}

    def get_resources(self) -> List[Dict[str, Any]]:
        """Get all resources"""
        data = self.get_all()
        return data.get("resources", [])

    def save_resources(self, resources: List[Dict[str, Any]]) -> bool:
        """Save all resources"""
        return self.save_all({"resources": resources})

    def add_resource(self, resource: Dict[str, Any]) -> bool:
        """Add a new resource"""
        resources = self.get_resources()
        resources.append(resource)
        return self.save_resources(resources)

    def get_resource_by_id(self, resource_id: str) -> Optional[Dict[str, Any]]:
        """Get resource by ID"""
        resources = self.get_resources()
        for resource in resources:
            if resource.get("id") == resource_id:
                return resource
        return None

    def update_resource(self, resource_id: str, updated_fields: Dict[str, Any]) -> bool:
        """Update a resource"""
        resources = self.get_resources()
        for i, resource in enumerate(resources):
            if resource.get("id") == resource_id:
                resources[i].update(updated_fields)
                return self.save_resources(resources)
        return False

    def delete_resource(self, resource_id: str) -> bool:
        """Delete a resource"""
        resources = self.get_resources()
        resources = [r for r in resources if r.get("id") != resource_id]
        return self.save_resources(resources)


# Global storage instances
todo_storage = TodoStorage()
library_storage = LibraryStorage()
