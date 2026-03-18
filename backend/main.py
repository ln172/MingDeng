"""
MingDeng Backend API
FastAPI server for MingDeng learning assistant
"""

from fastapi import FastAPI, HTTPException, Body
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import StreamingResponse
from pydantic import BaseModel, Field
from typing import Optional, List, Dict, Any
import uvicorn

from core.config import config_manager
from core.todo_manager import todo_manager
from core.plan_generator import plan_generator
from core.library_manager import library_manager
from core.backup_manager import backup_manager
from core.memory import memory_manager
from core.ai import ai_client

app = FastAPI(title="MingDeng API", version="0.1.0")

# Enable CORS for Tauri and local web dev
# Note: allow_origins=["*"] is incompatible with allow_credentials=True per CORS spec.
# List explicit local origins instead.
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:8000",
        "http://127.0.0.1:8000",
        "tauri://localhost",
        "https://tauri.localhost",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# ============ Pydantic Models ============

class ConfigUpdate(BaseModel):
    base_url: Optional[str] = None
    api_key: Optional[str] = None
    model: Optional[str] = None
    user_name: Optional[str] = None
    timezone: Optional[str] = None


class TaskCreate(BaseModel):
    plan_id: str
    task: str
    date: str
    estimated_time: int = 60
    difficulty: str = "medium"
    priority: str = "medium"
    tags: List[str] = []


class TaskUpdate(BaseModel):
    task: Optional[str] = None
    date: Optional[str] = None
    estimated_time: Optional[int] = None
    difficulty: Optional[str] = None
    priority: Optional[str] = None
    tags: Optional[List[str]] = None
    status: Optional[str] = None
    notes: Optional[str] = None


class PlanGenerate(BaseModel):
    user_input: str
    start_date: Optional[str] = None


class ChatMessage(BaseModel):
    message: str
    stream: bool = False


class ResourceCreate(BaseModel):
    content: str
    description: str = ""
    type: str = "other"
    auto_link: bool = True


class ResourceUpdate(BaseModel):
    content: Optional[str] = None
    description: Optional[str] = None
    type: Optional[str] = None
    status: Optional[str] = None


# ============ Config Endpoints ============

@app.get("/api/config")
async def get_config():
    """Get current configuration"""
    config = config_manager.get_config()
    # Don't expose full API key
    api_key = config.api.api_key
    if api_key and len(api_key) > 8:
        masked_key = api_key[:4] + "..." + api_key[-4:]
    else:
        masked_key = "***" if api_key else ""

    return {
        "api": {
            "base_url": config.api.base_url,
            "api_key": masked_key,
            "model": config.api.model
        },
        "user": {
            "name": config.user.name,
            "timezone": config.user.timezone
        },
        "is_configured": config_manager.is_configured()
    }


@app.post("/api/config")
async def update_config(config_update: ConfigUpdate):
    """Update configuration"""
    try:
        if config_update.base_url or config_update.api_key or config_update.model:
            config_manager.update_api_config(
                base_url=config_update.base_url,
                api_key=config_update.api_key,
                model=config_update.model
            )
            # Refresh AI client with new config
            ai_client.refresh_client()

        if config_update.user_name or config_update.timezone:
            config_manager.update_user_config(
                name=config_update.user_name,
                timezone=config_update.timezone
            )

        return {"success": True, "message": "配置已更新"}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# ============ Task Endpoints ============

@app.get("/api/today")
async def get_today_tasks():
    """Get today's tasks"""
    try:
        tasks = todo_manager.get_today_tasks()
        return {"success": True, "tasks": tasks}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/tasks/date/{date}")
async def get_tasks_by_date(date: str):
    """Get tasks by date"""
    try:
        tasks = todo_manager.get_tasks_by_date(date)
        return {"success": True, "tasks": tasks}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/tasks")
async def create_task(task_create: TaskCreate):
    """Create a new task"""
    try:
        task_data = task_create.model_dump(exclude={"plan_id"})
        task = todo_manager.create_task(task_create.plan_id, task_data)

        if task:
            return {"success": True, "task": task}
        else:
            raise HTTPException(status_code=404, detail="计划不存在")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.put("/api/tasks/{task_id}")
async def update_task(task_id: str, task_update: TaskUpdate):
    """Update a task"""
    try:
        updates = task_update.model_dump(exclude_none=True)
        success = todo_manager.update_task(task_id, updates)

        if success:
            return {"success": True, "message": "任务已更新"}
        else:
            raise HTTPException(status_code=404, detail="任务不存在")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.delete("/api/tasks/{task_id}")
async def delete_task(task_id: str):
    """Delete a task"""
    try:
        success = todo_manager.delete_task(task_id)

        if success:
            return {"success": True, "message": "任务已删除"}
        else:
            raise HTTPException(status_code=404, detail="任务不存在")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/tasks/{task_id}/complete")
async def complete_task(task_id: str):
    """Mark task as completed"""
    try:
        success = todo_manager.complete_task(task_id)

        if success:
            return {"success": True, "message": "任务已完成"}
        else:
            raise HTTPException(status_code=404, detail="任务不存在")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/tasks/{task_id}/uncomplete")
async def uncomplete_task(task_id: str):
    """Mark task as pending"""
    try:
        success = todo_manager.uncomplete_task(task_id)

        if success:
            return {"success": True, "message": "任务已取消完成"}
        else:
            raise HTTPException(status_code=404, detail="任务不存在")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# ============ Plan Endpoints ============

@app.get("/api/plans")
async def get_all_plans():
    """Get all plans"""
    try:
        plans = todo_manager.get_all_plans()
        return {"success": True, "plans": plans}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/plans/{plan_id}")
async def get_plan(plan_id: str):
    """Get a specific plan"""
    try:
        plan = todo_manager.get_plan(plan_id)
        if plan:
            return {"success": True, "plan": plan}
        else:
            raise HTTPException(status_code=404, detail="计划不存在")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.delete("/api/plans/{plan_id}")
async def delete_plan(plan_id: str):
    """Delete a plan"""
    try:
        success = todo_manager.delete_plan(plan_id)

        if success:
            return {"success": True, "message": "计划已删除"}
        else:
            raise HTTPException(status_code=404, detail="计划不存在")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/plan/generate")
async def generate_plan(plan_gen: PlanGenerate):
    """Generate a learning plan using AI"""
    try:
        if not config_manager.is_configured():
            raise HTTPException(status_code=400, detail="请先配置 API 密钥")

        result = await plan_generator.generate_plan(
            plan_gen.user_input,
            plan_gen.start_date
        )
        return result
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


class RescheduleRequest(BaseModel):
    mode: str  # "from_today" or "include_incomplete"
    plan_id: Optional[str] = None


@app.post("/api/plan/reschedule")
async def reschedule_tasks(reschedule_req: RescheduleRequest):
    """Reschedule tasks using AI with memory-based optimization"""
    try:
        if not config_manager.is_configured():
            raise HTTPException(status_code=400, detail="请先配置 API 密钥")

        result = await plan_generator.reschedule_tasks(
            mode=reschedule_req.mode,
            plan_id=reschedule_req.plan_id
        )
        return result
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# ============ Chat Endpoints ============

@app.post("/api/chat")
async def chat(chat_msg: ChatMessage):
    """Chat with AI (with optional streaming)"""
    try:
        if not config_manager.is_configured():
            raise HTTPException(status_code=400, detail="请先配置 API 密钥")

        # Get context from memory
        context = memory_manager.get_context_for_chat(chat_msg.message)

        messages = [
            {"role": "system", "content": f"你是 MingDeng 学习助手。{context}"},
            {"role": "user", "content": chat_msg.message}
        ]

        if chat_msg.stream:
            # Streaming response
            async def generate():
                async for chunk in ai_client.chat_stream(messages):
                    yield chunk

            # Save to memory (non-blocking)
            memory_manager.add_message("user", chat_msg.message)

            return StreamingResponse(generate(), media_type="text/plain")
        else:
            # Non-streaming response
            response = await ai_client.chat(messages)

            # Save to memory
            memory_manager.add_message("user", chat_msg.message)
            memory_manager.add_message("assistant", response)

            return {"success": True, "response": response}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# ============ Resource Endpoints ============

@app.get("/api/resources")
async def get_all_resources():
    """Get all resources"""
    try:
        resources = library_manager.get_all_resources()
        return {"success": True, "resources": resources}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/resources/{resource_id}")
async def get_resource(resource_id: str):
    """Get a specific resource"""
    try:
        resource = library_manager.get_resource(resource_id)
        if resource:
            return {"success": True, "resource": resource}
        else:
            raise HTTPException(status_code=404, detail="资源不存在")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/resources")
async def create_resource(resource_create: ResourceCreate):
    """Create a new resource"""
    try:
        resource = library_manager.create_resource(
            content=resource_create.content,
            description=resource_create.description,
            resource_type=resource_create.type,
            auto_link=resource_create.auto_link
        )
        return {"success": True, "resource": resource}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.put("/api/resources/{resource_id}")
async def update_resource(resource_id: str, resource_update: ResourceUpdate):
    """Update a resource"""
    try:
        updates = resource_update.model_dump(exclude_none=True)
        success = library_manager.update_resource(resource_id, updates)

        if success:
            return {"success": True, "message": "资源已更新"}
        else:
            raise HTTPException(status_code=404, detail="资源不存在")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.delete("/api/resources/{resource_id}")
async def delete_resource(resource_id: str):
    """Delete a resource"""
    try:
        success = library_manager.delete_resource(resource_id)

        if success:
            return {"success": True, "message": "资源已删除"}
        else:
            raise HTTPException(status_code=404, detail="资源不存在")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/resources/task/{task_id}")
async def get_resources_for_task(task_id: str):
    """Get resources linked to a task"""
    try:
        resources = library_manager.get_resources_for_task(task_id)
        return {"success": True, "resources": resources}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# ============ Stats Endpoints ============

@app.get("/api/stats")
async def get_stats():
    """Get learning statistics"""
    try:
        stats = todo_manager.get_stats()
        return {"success": True, "stats": stats}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# ============ Backup Endpoints ============

@app.post("/api/backup")
async def create_backup(reason: str = Body("Manual backup", embed=True)):
    """Create a backup"""
    try:
        result = backup_manager.create_backup(reason)
        return result
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/backups")
async def list_backups():
    """List all backups"""
    try:
        backups = backup_manager.list_backups()
        return {"success": True, "backups": backups}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/restore/{backup_id}")
async def restore_backup(backup_id: str):
    """Restore from a backup"""
    try:
        result = backup_manager.restore_backup(backup_id)
        return result
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.delete("/api/backups/{backup_id}")
async def delete_backup(backup_id: str):
    """Delete a backup"""
    try:
        success = backup_manager.delete_backup(backup_id)

        if success:
            return {"success": True, "message": "备份已删除"}
        else:
            raise HTTPException(status_code=404, detail="备份不存在")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# ============ Health Check ============

@app.get("/")
async def root():
    """Health check"""
    return {
        "status": "running",
        "app": "MingDeng API",
        "version": "0.1.0"
    }


if __name__ == "__main__":
    uvicorn.run(app, host="127.0.0.1", port=8765)
