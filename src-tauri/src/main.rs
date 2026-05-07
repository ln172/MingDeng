#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{Local, NaiveDate, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, LogicalSize, Manager, Size};
use uuid::Uuid;

type AppResult<T> = Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiConfig {
    #[serde(default = "default_base_url")]
    base_url: String,
    #[serde(default)]
    api_key: String,
    #[serde(default = "default_model")]
    model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserConfig {
    #[serde(default = "default_user_name")]
    name: String,
    #[serde(default = "default_timezone")]
    timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    #[serde(default)]
    api: ApiConfig,
    #[serde(default)]
    user: UserConfig,
}

#[derive(Debug, Clone, Serialize)]
struct PublicApiConfig {
    base_url: String,
    api_key: String,
    model: String,
}

#[derive(Debug, Clone, Serialize)]
struct PublicConfig {
    api: PublicApiConfig,
    user: UserConfig,
    is_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodosData {
    #[serde(default)]
    plans: Vec<Plan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Plan {
    id: String,
    name: String,
    created_at: String,
    #[serde(default)]
    tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    #[serde(default = "new_id")]
    id: String,
    task: String,
    date: String,
    #[serde(default = "default_estimated_time")]
    estimated_time: u32,
    #[serde(default = "default_medium")]
    difficulty: String,
    #[serde(default = "default_medium")]
    priority: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default = "default_pending")]
    status: String,
    #[serde(default)]
    completed_at: Option<String>,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskCreate {
    #[serde(default)]
    plan_id: Option<String>,
    task: String,
    date: String,
    #[serde(default = "default_estimated_time")]
    estimated_time: u32,
    #[serde(default = "default_medium")]
    difficulty: String,
    #[serde(default = "default_medium")]
    priority: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TaskUpdate {
    task: Option<String>,
    date: Option<String>,
    estimated_time: Option<u32>,
    difficulty: Option<String>,
    priority: Option<String>,
    tags: Option<Vec<String>>,
    status: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LibraryData {
    #[serde(default)]
    resources: Vec<Resource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Resource {
    id: String,
    content: String,
    #[serde(default)]
    description: String,
    #[serde(rename = "type", default = "default_other")]
    resource_type: String,
    captured_at: String,
    #[serde(default)]
    linked_tasks: Vec<String>,
    #[serde(default = "default_unread")]
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResourceCreate {
    content: String,
    #[serde(default)]
    description: String,
    #[serde(rename = "type", default = "default_other")]
    resource_type: String,
    #[serde(default = "default_true")]
    auto_link: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ResourceUpdate {
    content: Option<String>,
    description: Option<String>,
    #[serde(rename = "type")]
    resource_type: Option<String>,
    status: Option<String>,
    linked_tasks: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
struct DifficultyStats {
    total: u32,
    completed: u32,
}

#[derive(Debug, Clone, Serialize)]
struct Stats {
    total_tasks: u32,
    completed: u32,
    pending: u32,
    skipped: u32,
    by_difficulty: BTreeMap<String, DifficultyStats>,
    completion_rate: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct PlanGenerateRequest {
    user_input: String,
    start_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RescheduleRequest {
    mode: String,
    plan_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RescheduleResult {
    success: bool,
    message: String,
    adjustment_reason: Option<String>,
    updated_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct DreamwalkResult {
    success: bool,
    ran: bool,
    date: String,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BackupInfo {
    id: String,
    timestamp: String,
    datetime: String,
    reason: String,
    files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Debug, Clone)]
struct Store {
    root: PathBuf,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            api_key: String::new(),
            model: default_model(),
        }
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            name: default_user_name(),
            timezone: default_timezone(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            user: UserConfig::default(),
        }
    }
}

impl Default for TodosData {
    fn default() -> Self {
        Self { plans: Vec::new() }
    }
}

impl Default for LibraryData {
    fn default() -> Self {
        Self {
            resources: Vec::new(),
        }
    }
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_user_name() -> String {
    "User".to_string()
}

fn default_timezone() -> String {
    "Asia/Shanghai".to_string()
}

fn default_estimated_time() -> u32 {
    60
}

fn default_medium() -> String {
    "medium".to_string()
}

fn default_pending() -> String {
    "pending".to_string()
}

fn default_unread() -> String {
    "unread".to_string()
}

fn default_other() -> String {
    "other".to_string()
}

fn default_true() -> bool {
    true
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn today_in_timezone(timezone: &str) -> NaiveDate {
    let tz = timezone.parse::<Tz>().unwrap_or(chrono_tz::Asia::Shanghai);
    Utc::now().with_timezone(&tz).date_naive()
}

fn app_store(app: &AppHandle) -> AppResult<Store> {
    let root = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(_) => std::env::current_dir()
            .map(|cwd| cwd.join("data"))
            .map_err(|e| e.to_string())?,
    };
    Store::new(root)
}

impl Store {
    fn new(root: PathBuf) -> AppResult<Self> {
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let store = Self { root };
        store.ensure_files()?;
        Ok(store)
    }

    fn ensure_files(&self) -> AppResult<()> {
        self.ensure_json(&self.config_path(), &AppConfig::default())?;
        self.ensure_json(&self.todos_path(), &TodosData::default())?;
        self.ensure_json(&self.library_path(), &LibraryData::default())?;
        fs::create_dir_all(self.memory_dir()).map_err(|e| e.to_string())?;
        fs::create_dir_all(self.conversations_dir()).map_err(|e| e.to_string())?;
        if !self.long_memory_path().exists() {
            self.write_long_memory(None, "# 长期记忆\n\n暂无长期记忆。")?;
        }
        Ok(())
    }

    fn ensure_json<T: Serialize>(&self, path: &Path, default_value: &T) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        if !path.exists() {
            self.write_json(path, default_value)?;
        }
        Ok(())
    }

    fn config_path(&self) -> PathBuf {
        self.root.join("config.json")
    }

    fn todos_path(&self) -> PathBuf {
        self.root.join("todos.json")
    }

    fn library_path(&self) -> PathBuf {
        self.root.join("library.json")
    }

    fn backups_dir(&self) -> PathBuf {
        self.root.join("backups")
    }

    fn memory_dir(&self) -> PathBuf {
        self.root.join("memory")
    }

    fn conversations_dir(&self) -> PathBuf {
        self.memory_dir().join("conversations")
    }

    fn long_memory_path(&self) -> PathBuf {
        self.memory_dir().join("long-term.md")
    }

    fn dreamwalk_log_path(&self) -> PathBuf {
        self.memory_dir().join("dreamwalk-log.md")
    }

    fn read_json<T>(&self, path: &Path) -> AppResult<T>
    where
        T: for<'de> Deserialize<'de> + Default,
    {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| e.to_string())
    }

    fn write_json<T: Serialize>(&self, path: &Path, value: &T) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
        self.atomic_write(path, &format!("{text}\n"))
    }

    fn atomic_write(&self, path: &Path, text: &str) -> AppResult<()> {
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, text).map_err(|e| e.to_string())?;
        fs::rename(&tmp, path).map_err(|e| e.to_string())
    }

    fn config(&self) -> AppResult<AppConfig> {
        self.read_json(&self.config_path())
    }

    fn save_config(&self, config: &AppConfig) -> AppResult<()> {
        self.write_json(&self.config_path(), config)
    }

    fn todos(&self) -> AppResult<TodosData> {
        self.read_json(&self.todos_path())
    }

    fn save_todos(&self, data: &TodosData) -> AppResult<()> {
        self.write_json(&self.todos_path(), data)
    }

    fn library(&self) -> AppResult<LibraryData> {
        self.read_json(&self.library_path())
    }

    fn save_library(&self, data: &LibraryData) -> AppResult<()> {
        self.write_json(&self.library_path(), data)
    }

    fn public_config(&self) -> AppResult<PublicConfig> {
        let config = self.config()?;
        let api_key = if config.api.api_key.is_empty() {
            String::new()
        } else if config.api.api_key.len() > 8 {
            format!(
                "{}...{}",
                &config.api.api_key[..4],
                &config.api.api_key[config.api.api_key.len() - 4..]
            )
        } else {
            "***".to_string()
        };
        Ok(PublicConfig {
            api: PublicApiConfig {
                base_url: config.api.base_url.clone(),
                api_key,
                model: config.api.model.clone(),
            },
            user: config.user.clone(),
            is_configured: is_configured(&config),
        })
    }

    fn stats(&self) -> AppResult<Stats> {
        let todos = self.todos()?;
        Ok(calculate_stats(&todos.plans))
    }

    fn create_plan(&self, name: String, mut tasks: Vec<Task>) -> AppResult<Plan> {
        for task in &mut tasks {
            if task.id.is_empty() {
                task.id = new_id();
            }
            if task.status.is_empty() {
                task.status = default_pending();
            }
            if task.priority.is_empty() {
                task.priority = default_medium();
            }
            if task.difficulty.is_empty() {
                task.difficulty = default_medium();
            }
        }
        let plan = Plan {
            id: new_id(),
            name,
            created_at: now_iso(),
            tasks,
        };
        let mut data = self.todos()?;
        data.plans.push(plan.clone());
        self.save_todos(&data)?;
        Ok(plan)
    }

    fn create_task(&self, input: TaskCreate) -> AppResult<Task> {
        let mut data = self.todos()?;
        let target_plan_id = if let Some(id) = input.plan_id.clone() {
            id
        } else if let Some(plan) = data.plans.first() {
            plan.id.clone()
        } else {
            let plan = Plan {
                id: new_id(),
                name: "快速添加任务".to_string(),
                created_at: now_iso(),
                tasks: Vec::new(),
            };
            let id = plan.id.clone();
            data.plans.push(plan);
            id
        };

        let task = Task {
            id: new_id(),
            task: input.task,
            date: input.date,
            estimated_time: input.estimated_time,
            difficulty: input.difficulty,
            priority: input.priority,
            tags: input.tags,
            status: default_pending(),
            completed_at: None,
            notes: input.notes,
        };

        let plan = data
            .plans
            .iter_mut()
            .find(|plan| plan.id == target_plan_id)
            .ok_or_else(|| "计划不存在".to_string())?;
        plan.tasks.push(task.clone());
        self.save_todos(&data)?;
        Ok(task)
    }

    fn update_task(&self, task_id: &str, updates: TaskUpdate) -> AppResult<bool> {
        let mut data = self.todos()?;
        for plan in &mut data.plans {
            if let Some(task) = plan.tasks.iter_mut().find(|task| task.id == task_id) {
                if let Some(value) = updates.task {
                    task.task = value;
                }
                if let Some(value) = updates.date {
                    task.date = value;
                }
                if let Some(value) = updates.estimated_time {
                    task.estimated_time = value;
                }
                if let Some(value) = updates.difficulty {
                    task.difficulty = value;
                }
                if let Some(value) = updates.priority {
                    task.priority = value;
                }
                if let Some(value) = updates.tags {
                    task.tags = value;
                }
                if let Some(value) = updates.notes {
                    task.notes = value;
                }
                if let Some(value) = updates.status {
                    if value == "completed" && task.completed_at.is_none() {
                        task.completed_at = Some(now_iso());
                    }
                    if value == "pending" {
                        task.completed_at = None;
                    }
                    task.status = value;
                }
                self.save_todos(&data)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn delete_task(&self, task_id: &str) -> AppResult<bool> {
        let mut data = self.todos()?;
        for plan in &mut data.plans {
            let before = plan.tasks.len();
            plan.tasks.retain(|task| task.id != task_id);
            if plan.tasks.len() != before {
                self.save_todos(&data)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn read_long_memory(&self) -> AppResult<String> {
        fs::read_to_string(self.long_memory_path()).map_err(|e| e.to_string())
    }

    fn write_long_memory(&self, date: Option<NaiveDate>, body: &str) -> AppResult<()> {
        let last = date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "never".to_string());
        let text = format!(
            "---\nschema_version: 1\nlast_dreamwalk_date: {last}\n---\n\n{}\n",
            body.trim()
        );
        self.atomic_write(&self.long_memory_path(), &text)
    }

    fn last_dreamwalk_date(&self) -> AppResult<Option<NaiveDate>> {
        let text = self.read_long_memory()?;
        for line in text.lines() {
            if let Some(raw) = line.strip_prefix("last_dreamwalk_date:") {
                let value = raw.trim();
                if value == "never" || value.is_empty() {
                    return Ok(None);
                }
                return Ok(NaiveDate::parse_from_str(value, "%Y-%m-%d").ok());
            }
        }
        Ok(None)
    }

    fn append_conversation(&self, role: &str, content: &str, timezone: &str) -> AppResult<()> {
        fs::create_dir_all(self.conversations_dir()).map_err(|e| e.to_string())?;
        let date = today_in_timezone(timezone);
        let path = self.conversations_dir().join(format!("{date}.md"));
        let mut existing = fs::read_to_string(&path).unwrap_or_default();
        if existing.is_empty() {
            existing.push_str(&format!("# 对话记录 {date}\n\n"));
        }
        existing.push_str(&format!("## {}\n\n{}\n\n", role, content.trim()));
        self.atomic_write(&path, &existing)
    }

    fn recent_conversations(&self, limit: usize) -> AppResult<String> {
        let mut files = Vec::new();
        if self.conversations_dir().exists() {
            for entry in fs::read_dir(self.conversations_dir()).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                    files.push(entry.path());
                }
            }
        }
        files.sort();
        files.reverse();
        let mut chunks = Vec::new();
        for path in files.into_iter().take(limit) {
            if let Ok(text) = fs::read_to_string(path) {
                chunks.push(text);
            }
        }
        Ok(chunks.join("\n\n---\n\n"))
    }

    fn append_dreamwalk_log(&self, date: NaiveDate, summary: &str) -> AppResult<()> {
        let path = self.dreamwalk_log_path();
        let mut existing = fs::read_to_string(&path).unwrap_or_else(|_| "# 梦游日志\n\n".to_string());
        existing.push_str(&format!("## {date}\n\n{}\n\n", summary.trim()));
        self.atomic_write(&path, &existing)
    }

    fn create_backup(&self, reason: String) -> AppResult<BackupInfo> {
        fs::create_dir_all(self.backups_dir()).map_err(|e| e.to_string())?;
        let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
        let id = format!("backup_{timestamp}");
        let backup_dir = self.backups_dir().join(&id);
        fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

        let mut files = Vec::new();
        for name in ["todos.json", "library.json", "config.json"] {
            let source = self.root.join(name);
            if source.exists() {
                fs::copy(&source, backup_dir.join(name)).map_err(|e| e.to_string())?;
                files.push(name.to_string());
            }
        }
        if self.memory_dir().exists() {
            copy_dir_all(&self.memory_dir(), &backup_dir.join("memory"))?;
            files.push("memory/".to_string());
        }

        let info = BackupInfo {
            id: id.clone(),
            timestamp,
            datetime: now_iso(),
            reason,
            files,
        };
        self.write_json(&backup_dir.join("metadata.json"), &info)?;
        self.cleanup_backups(10)?;
        Ok(info)
    }

    fn list_backups(&self) -> AppResult<Vec<BackupInfo>> {
        let mut backups = Vec::new();
        if !self.backups_dir().exists() {
            return Ok(backups);
        }
        for entry in fs::read_dir(self.backups_dir()).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let metadata = entry.path().join("metadata.json");
            if metadata.exists() {
                if let Ok(info) = self.read_json::<BackupInfo>(&metadata) {
                    backups.push(info);
                }
            }
        }
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(backups)
    }

    fn cleanup_backups(&self, keep: usize) -> AppResult<()> {
        let mut backups = self.list_backups()?;
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        for backup in backups.into_iter().skip(keep) {
            let path = self.backups_dir().join(backup.id);
            if path.exists() {
                fs::remove_dir_all(path).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    fn restore_backup(&self, backup_id: &str) -> AppResult<BackupInfo> {
        let backup_dir = self.backups_dir().join(backup_id);
        if !backup_dir.exists() {
            return Err("备份不存在".to_string());
        }
        let info = self.read_json::<BackupInfo>(&backup_dir.join("metadata.json"))?;
        let _ = self.create_backup(format!("Before restoring {backup_id}"));
        for name in ["todos.json", "library.json", "config.json"] {
            let source = backup_dir.join(name);
            if source.exists() {
                fs::copy(source, self.root.join(name)).map_err(|e| e.to_string())?;
            }
        }
        let memory_source = backup_dir.join("memory");
        if memory_source.exists() {
            if self.memory_dir().exists() {
                fs::remove_dir_all(self.memory_dir()).map_err(|e| e.to_string())?;
            }
            copy_dir_all(&memory_source, &self.memory_dir())?;
        }
        Ok(info)
    }
}

fn is_configured(config: &AppConfig) -> bool {
    !config.api.api_key.trim().is_empty()
        && !config.api.base_url.trim().is_empty()
        && !config.api.model.trim().is_empty()
}

fn calculate_stats(plans: &[Plan]) -> Stats {
    let mut by_difficulty = BTreeMap::new();
    for key in ["simple", "medium", "hard"] {
        by_difficulty.insert(
            key.to_string(),
            DifficultyStats {
                total: 0,
                completed: 0,
            },
        );
    }

    let mut stats = Stats {
        total_tasks: 0,
        completed: 0,
        pending: 0,
        skipped: 0,
        by_difficulty,
        completion_rate: 0.0,
    };

    for task in plans.iter().flat_map(|plan| &plan.tasks) {
        stats.total_tasks += 1;
        match task.status.as_str() {
            "completed" => stats.completed += 1,
            "skipped" => stats.skipped += 1,
            _ => stats.pending += 1,
        }
        let key = if stats.by_difficulty.contains_key(&task.difficulty) {
            task.difficulty.clone()
        } else {
            "medium".to_string()
        };
        if let Some(bucket) = stats.by_difficulty.get_mut(&key) {
            bucket.total += 1;
            if task.status == "completed" {
                bucket.completed += 1;
            }
        }
    }
    if stats.total_tasks > 0 {
        stats.completion_rate =
            ((stats.completed as f32 / stats.total_tasks as f32) * 1000.0).round() / 10.0;
    }
    stats
}

fn copy_dir_all(source: &Path, target: &Path) -> AppResult<()> {
    fs::create_dir_all(target).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        let dest = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn extract_json(text: &str) -> AppResult<Value> {
    let trimmed = text.trim();
    let json_text = if let Some(start) = trimmed.find("```json") {
        let after = &trimmed[start + 7..];
        after
            .split("```")
            .next()
            .ok_or_else(|| "AI JSON 代码块不完整".to_string())?
            .trim()
            .to_string()
    } else if let Some(start) = trimmed.find("```") {
        let after = &trimmed[start + 3..];
        after
            .split("```")
            .next()
            .ok_or_else(|| "AI JSON 代码块不完整".to_string())?
            .trim()
            .to_string()
    } else {
        trimmed.to_string()
    };
    serde_json::from_str(&json_text).map_err(|e| format!("AI 返回不是合法 JSON: {e}"))
}

async fn chat_completion(config: &AppConfig, messages: Vec<Value>, temperature: f32) -> AppResult<String> {
    if !is_configured(config) {
        return Err("请先配置 API Key、Base URL 和模型".to_string());
    }
    let endpoint = format!(
        "{}/chat/completions",
        config.api.base_url.trim_end_matches('/')
    );
    let client = reqwest::Client::new();
    let response = client
        .post(endpoint)
        .bearer_auth(&config.api.api_key)
        .json(&json!({
            "model": config.api.model,
            "messages": messages,
            "temperature": temperature
        }))
        .send()
        .await
        .map_err(|e| format!("AI 请求失败: {e}"))?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("AI 响应解析失败: {e}"))?;
    if !status.is_success() {
        return Err(format!("AI API 错误 {status}: {body}"));
    }
    body.pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .ok_or_else(|| format!("AI 响应缺少 content: {body}"))
}

fn task_from_value(value: &Value, fallback_date: &str) -> Task {
    Task {
        id: value
            .get("id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(new_id),
        task: value
            .get("task")
            .and_then(Value::as_str)
            .unwrap_or("学习任务")
            .to_string(),
        date: value
            .get("date")
            .and_then(Value::as_str)
            .unwrap_or(fallback_date)
            .to_string(),
        estimated_time: value
            .get("estimated_time")
            .and_then(Value::as_u64)
            .unwrap_or(60) as u32,
        difficulty: value
            .get("difficulty")
            .and_then(Value::as_str)
            .unwrap_or("medium")
            .to_string(),
        priority: value
            .get("priority")
            .and_then(Value::as_str)
            .unwrap_or("medium")
            .to_string(),
        tags: value
            .get("tags")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        status: default_pending(),
        completed_at: None,
        notes: String::new(),
    }
}

fn assign_dates(tasks: &mut [Task], start_date: NaiveDate) {
    let mut current = start_date;
    let mut daily_minutes = 0;
    for task in tasks {
        if task.date.trim().is_empty() {
            if daily_minutes + task.estimated_time > 240 {
                current = current.succ_opt().unwrap_or(current);
                daily_minutes = 0;
            }
            task.date = current.to_string();
            daily_minutes += task.estimated_time;
        }
    }
}

#[tauri::command]
fn get_config(app: AppHandle) -> AppResult<PublicConfig> {
    app_store(&app)?.public_config()
}

#[tauri::command]
fn save_config(app: AppHandle, config: AppConfig) -> AppResult<PublicConfig> {
    let store = app_store(&app)?;
    let mut next = config;
    let existing = store.config().unwrap_or_default();
    if next.api.api_key.trim().is_empty() && !existing.api.api_key.trim().is_empty() {
        next.api.api_key = existing.api.api_key;
    }
    store.save_config(&next)?;
    store.public_config()
}

#[tauri::command]
fn list_plans(app: AppHandle) -> AppResult<Vec<Plan>> {
    Ok(app_store(&app)?.todos()?.plans)
}

#[tauri::command]
fn delete_plan(app: AppHandle, plan_id: String) -> AppResult<bool> {
    let store = app_store(&app)?;
    let mut data = store.todos()?;
    let before = data.plans.len();
    data.plans.retain(|plan| plan.id != plan_id);
    store.save_todos(&data)?;
    Ok(before != data.plans.len())
}

#[tauri::command]
fn delete_plan_by_id(app: AppHandle, id: String) -> AppResult<bool> {
    delete_plan(app, id)
}

#[tauri::command]
fn tasks_by_date(app: AppHandle, date: String) -> AppResult<Vec<Task>> {
    let data = app_store(&app)?.todos()?;
    Ok(data
        .plans
        .into_iter()
        .flat_map(|plan| plan.tasks)
        .filter(|task| task.date == date)
        .collect())
}

#[tauri::command]
fn create_task(app: AppHandle, task: TaskCreate) -> AppResult<Task> {
    app_store(&app)?.create_task(task)
}

#[tauri::command]
fn update_task(app: AppHandle, task_id: String, updates: TaskUpdate) -> AppResult<bool> {
    app_store(&app)?.update_task(&task_id, updates)
}

#[tauri::command]
fn delete_task(app: AppHandle, task_id: String) -> AppResult<bool> {
    app_store(&app)?.delete_task(&task_id)
}

#[tauri::command]
fn delete_task_by_id(app: AppHandle, id: String) -> AppResult<bool> {
    delete_task(app, id)
}

#[tauri::command]
fn complete_task(app: AppHandle, task_id: String) -> AppResult<bool> {
    app_store(&app)?.update_task(
        &task_id,
        TaskUpdate {
            status: Some("completed".to_string()),
            ..Default::default()
        },
    )
}

#[tauri::command]
fn uncomplete_task(app: AppHandle, task_id: String) -> AppResult<bool> {
    app_store(&app)?.update_task(
        &task_id,
        TaskUpdate {
            status: Some("pending".to_string()),
            ..Default::default()
        },
    )
}

#[tauri::command]
async fn generate_plan(app: AppHandle, request: PlanGenerateRequest) -> AppResult<Plan> {
    let store = app_store(&app)?;
    let config = store.config()?;
    let memory = store.read_long_memory().unwrap_or_default();
    let start = request
        .start_date
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| today_in_timezone(&config.user.timezone));
    let prompt = format!(
        r#"你是 MingDeng 学习规划助手。请严格根据用户输入生成学习计划，不要添加用户未提及的学习内容。

长期记忆：
{memory}

用户输入：
{input}

要求：
1. 只拆解用户明确提到的学习内容。
2. 每个任务 15min-3h，具体可执行。
3. 难度只能是 simple、medium、hard。
4. 从 {start} 开始安排，默认每天 2-3 小时，单日不要超过 4 小时。
5. 输出 JSON，不要解释。

格式：
{{
  "plan_name": "简洁的计划名称",
  "tasks": [
    {{
      "task": "具体任务描述",
      "date": "{start}",
      "estimated_time": 90,
      "difficulty": "simple",
      "priority": "medium",
      "tags": ["标签"]
    }}
  ]
}}"#,
        memory = memory,
        input = request.user_input,
        start = start
    );
    let response = chat_completion(
        &config,
        vec![
            json!({"role": "system", "content": "你是专业学习规划助手，只返回合法 JSON。"}),
            json!({"role": "user", "content": prompt}),
        ],
        0.7,
    )
    .await?;
    let value = extract_json(&response)?;
    let name = value
        .get("plan_name")
        .and_then(Value::as_str)
        .unwrap_or("新学习计划")
        .to_string();
    let mut tasks: Vec<Task> = value
        .get("tasks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|task| task_from_value(task, &start.to_string()))
        .collect();
    assign_dates(&mut tasks, start);
    store.create_plan(name, tasks)
}

#[tauri::command]
async fn reschedule_tasks(app: AppHandle, request: RescheduleRequest) -> AppResult<RescheduleResult> {
    let store = app_store(&app)?;
    let config = store.config()?;
    let mut data = store.todos()?;
    let today = today_in_timezone(&config.user.timezone).to_string();
    let stats = calculate_stats(&data.plans);
    let memory = store.read_long_memory().unwrap_or_default();
    let mut candidates = Vec::new();

    for plan in &data.plans {
        if let Some(plan_id) = &request.plan_id {
            if &plan.id != plan_id {
                continue;
            }
        }
        for task in &plan.tasks {
            let include = match request.mode.as_str() {
                "from_today" => task.date >= today && task.status != "completed",
                "include_incomplete" => task.status != "completed",
                _ => return Err("不支持的重排模式".to_string()),
            };
            if include {
                candidates.push(task.clone());
            }
        }
    }

    if candidates.is_empty() {
        return Ok(RescheduleResult {
            success: false,
            message: "没有需要重新安排的任务".to_string(),
            adjustment_reason: None,
            updated_count: 0,
        });
    }

    let tasks_text = candidates
        .iter()
        .map(|task| {
            format!(
                "- ID: {}, 任务: {}, 日期: {}, 时长: {}min, 难度: {}, 状态: {}",
                task.id, task.task, task.date, task.estimated_time, task.difficulty, task.status
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = format!(
        r#"你是 MingDeng 学习规划助手。根据用户完成率、长期记忆和任务列表重新安排未完成任务。

今天：{today}
完成率：{rate}%
长期记忆：
{memory}

待重排任务：
{tasks}

规则：
1. 保留原任务 ID。
2. 从今天开始重新分配日期。
3. 完成率低于 50% 时降低每日任务密度；高于 80% 可略微提高密度。
4. 每天 2-4 小时，高难任务分散。
5. 只返回 JSON。

格式：
{{
  "adjustment_reason": "简短说明",
  "tasks": [
    {{
      "id": "原 ID",
      "task": "任务描述",
      "date": "{today}",
      "estimated_time": 60,
      "difficulty": "medium",
      "priority": "medium",
      "tags": []
    }}
  ]
}}"#,
        today = today,
        rate = stats.completion_rate,
        memory = memory,
        tasks = tasks_text
    );
    let response = chat_completion(
        &config,
        vec![
            json!({"role": "system", "content": "你是学习任务重排助手，只返回合法 JSON。"}),
            json!({"role": "user", "content": prompt}),
        ],
        0.5,
    )
    .await?;
    let value = extract_json(&response)?;
    let reason = value
        .get("adjustment_reason")
        .and_then(Value::as_str)
        .map(str::to_string);
    let tasks = value
        .get("tasks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut updated_count = 0;
    for updated in tasks {
        let Some(id) = updated.get("id").and_then(Value::as_str) else {
            continue;
        };
        for plan in &mut data.plans {
            if let Some(task) = plan.tasks.iter_mut().find(|task| task.id == id) {
                if let Some(value) = updated.get("task").and_then(Value::as_str) {
                    task.task = value.to_string();
                }
                if let Some(value) = updated.get("date").and_then(Value::as_str) {
                    task.date = value.to_string();
                }
                if let Some(value) = updated.get("estimated_time").and_then(Value::as_u64) {
                    task.estimated_time = value as u32;
                }
                if let Some(value) = updated.get("difficulty").and_then(Value::as_str) {
                    task.difficulty = value.to_string();
                }
                if let Some(value) = updated.get("priority").and_then(Value::as_str) {
                    task.priority = value.to_string();
                }
                if let Some(value) = updated.get("tags").and_then(Value::as_array) {
                    task.tags = value
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect();
                }
                updated_count += 1;
            }
        }
    }
    store.save_todos(&data)?;
    Ok(RescheduleResult {
        success: true,
        message: format!("已重新安排 {updated_count} 个任务"),
        adjustment_reason: reason,
        updated_count,
    })
}

#[tauri::command]
fn list_resources(app: AppHandle) -> AppResult<Vec<Resource>> {
    Ok(app_store(&app)?.library()?.resources)
}

#[tauri::command]
async fn create_resource(app: AppHandle, resource: ResourceCreate) -> AppResult<Resource> {
    let store = app_store(&app)?;
    let mut data = store.library()?;
    let linked = if resource.auto_link {
        auto_link_resource(&store, &resource).await.unwrap_or_default()
    } else {
        Vec::new()
    };
    let created = Resource {
        id: new_id(),
        content: resource.content,
        description: resource.description,
        resource_type: resource.resource_type,
        captured_at: now_iso(),
        linked_tasks: linked,
        status: default_unread(),
    };
    data.resources.push(created.clone());
    store.save_library(&data)?;
    Ok(created)
}

async fn auto_link_resource(store: &Store, resource: &ResourceCreate) -> AppResult<Vec<String>> {
    let config = store.config()?;
    if !is_configured(&config) {
        return Ok(keyword_link_resource(store, resource));
    }
    let todos = store.todos()?;
    let pending: Vec<Task> = todos
        .plans
        .iter()
        .flat_map(|plan| plan.tasks.clone())
        .filter(|task| task.status == "pending")
        .take(20)
        .collect();
    if pending.is_empty() {
        return Ok(Vec::new());
    }
    let tasks_text = pending
        .iter()
        .map(|task| format!("- ID: {}, 任务: {}, 标签: {:?}", task.id, task.task, task.tags))
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = format!(
        "资源：{}\n描述：{}\n\n当前任务：\n{}\n\n判断最相关任务，返回 JSON：{{\"linked_task_id\":\"uuid 或 null\",\"reason\":\"原因\"}}",
        resource.content, resource.description, tasks_text
    );
    let response = chat_completion(
        &config,
        vec![
            json!({"role": "system", "content": "你是资源关联助手，只返回合法 JSON。"}),
            json!({"role": "user", "content": prompt}),
        ],
        0.3,
    )
    .await?;
    let value = extract_json(&response)?;
    let id = value
        .get("linked_task_id")
        .and_then(Value::as_str)
        .filter(|id| *id != "null" && !id.is_empty())
        .map(str::to_string);
    Ok(id.into_iter().collect())
}

fn keyword_link_resource(store: &Store, resource: &ResourceCreate) -> Vec<String> {
    let Ok(todos) = store.todos() else {
        return Vec::new();
    };
    let text = format!("{} {}", resource.content, resource.description).to_lowercase();
    for task in todos.plans.iter().flat_map(|plan| &plan.tasks) {
        if task.status != "pending" {
            continue;
        }
        let task_text = format!("{} {}", task.task, task.tags.join(" ")).to_lowercase();
        let common = task_text
            .split_whitespace()
            .filter(|word| word.len() > 3 && text.contains(*word))
            .count();
        if common >= 2 {
            return vec![task.id.clone()];
        }
    }
    Vec::new()
}

#[tauri::command]
fn update_resource(app: AppHandle, resource_id: String, updates: ResourceUpdate) -> AppResult<bool> {
    let store = app_store(&app)?;
    let mut data = store.library()?;
    if let Some(resource) = data.resources.iter_mut().find(|resource| resource.id == resource_id) {
        if let Some(value) = updates.content {
            resource.content = value;
        }
        if let Some(value) = updates.description {
            resource.description = value;
        }
        if let Some(value) = updates.resource_type {
            resource.resource_type = value;
        }
        if let Some(value) = updates.status {
            resource.status = value;
        }
        if let Some(value) = updates.linked_tasks {
            resource.linked_tasks = value;
        }
        store.save_library(&data)?;
        return Ok(true);
    }
    Ok(false)
}

#[tauri::command]
fn delete_resource(app: AppHandle, resource_id: String) -> AppResult<bool> {
    let store = app_store(&app)?;
    let mut data = store.library()?;
    let before = data.resources.len();
    data.resources.retain(|resource| resource.id != resource_id);
    store.save_library(&data)?;
    Ok(before != data.resources.len())
}

#[tauri::command]
fn get_stats(app: AppHandle) -> AppResult<Stats> {
    app_store(&app)?.stats()
}

#[tauri::command]
fn create_backup(app: AppHandle, reason: String) -> AppResult<BackupInfo> {
    app_store(&app)?.create_backup(reason)
}

#[tauri::command]
fn list_backups(app: AppHandle) -> AppResult<Vec<BackupInfo>> {
    app_store(&app)?.list_backups()
}

#[tauri::command]
fn restore_backup(app: AppHandle, backup_id: String) -> AppResult<BackupInfo> {
    app_store(&app)?.restore_backup(&backup_id)
}

#[tauri::command]
fn delete_backup(app: AppHandle, backup_id: String) -> AppResult<bool> {
    let path = app_store(&app)?.backups_dir().join(backup_id);
    if path.exists() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
async fn send_chat(app: AppHandle, request: ChatRequest) -> AppResult<String> {
    let store = app_store(&app)?;
    let config = store.config()?;
    let memory = store.read_long_memory().unwrap_or_default();
    store.append_conversation("user", &request.message, &config.user.timezone)?;
    let response = chat_completion(
        &config,
        vec![
            json!({"role": "system", "content": format!("你是 MingDeng 学习助手。请结合长期记忆提供连续、具体的学习建议。\n\n{}", memory)}),
            json!({"role": "user", "content": request.message}),
        ],
        0.7,
    )
    .await?;
    store.append_conversation("assistant", &response, &config.user.timezone)?;
    Ok(response)
}

#[tauri::command]
async fn run_dreamwalk_if_needed(app: AppHandle) -> AppResult<DreamwalkResult> {
    let store = app_store(&app)?;
    let config = store.config()?;
    if !is_configured(&config) {
        return Err("请先配置 API，明灯需要完成梦游后才能进入主应用".to_string());
    }
    let today = today_in_timezone(&config.user.timezone);
    if store.last_dreamwalk_date()? == Some(today) {
        return Ok(DreamwalkResult {
            success: true,
            ran: false,
            date: today.to_string(),
            message: "今天已完成梦游".to_string(),
        });
    }
    let stats = store.stats()?;
    let long_memory = store.read_long_memory().unwrap_or_default();
    let conversations = store.recent_conversations(3)?;
    let resources = store.library()?.resources;
    let recent_resources = resources
        .iter()
        .rev()
        .take(10)
        .map(|resource| format!("- {}: {}", resource.description, resource.content))
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = format!(
        r#"今天是 {today}。请根据 MingDeng 用户最近的学习活动更新长期记忆。

现有长期记忆：
{long_memory}

统计：
- 总任务：{total}
- 已完成：{completed}
- 待完成：{pending}
- 完成率：{rate}%

最近资源：
{resources}

最近对话：
{conversations}

请输出新的长期记忆 Markdown 正文。不要输出 frontmatter。内容要保留长期稳定偏好、学习目标、卡点、节奏、完成表现和后续建议。"#,
        today = today,
        long_memory = long_memory,
        total = stats.total_tasks,
        completed = stats.completed,
        pending = stats.pending,
        rate = stats.completion_rate,
        resources = recent_resources,
        conversations = conversations
    );
    let updated = chat_completion(
        &config,
        vec![
            json!({"role": "system", "content": "你是 MingDeng 的长期记忆整理器。只输出 Markdown 正文。"}),
            json!({"role": "user", "content": prompt}),
        ],
        0.4,
    )
    .await?;
    store.write_long_memory(Some(today), &updated)?;
    store.append_dreamwalk_log(today, &format!("已更新长期记忆。\n\n{}", updated))?;
    Ok(DreamwalkResult {
        success: true,
        ran: true,
        date: today.to_string(),
        message: "梦游完成，长期记忆已更新".to_string(),
    })
}

#[tauri::command]
fn clear_memory(app: AppHandle) -> AppResult<bool> {
    let store = app_store(&app)?;
    if store.memory_dir().exists() {
        fs::remove_dir_all(store.memory_dir()).map_err(|e| e.to_string())?;
    }
    store.ensure_files()?;
    Ok(true)
}

#[tauri::command]
fn minimize_window(app: AppHandle) -> AppResult<bool> {
    if let Some(window) = app.get_webview_window("main") {
        window.minimize().map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn close_window(app: AppHandle) -> AppResult<bool> {
    if let Some(window) = app.get_webview_window("main") {
        window.close().map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn set_widget_expanded(app: AppHandle, expanded: bool) -> AppResult<bool> {
    if let Some(window) = app.get_webview_window("main") {
        let size = if expanded {
            LogicalSize {
                width: 420.0,
                height: 620.0,
            }
        } else {
            LogicalSize {
                width: 380.0,
                height: 172.0,
            }
        };
        window.set_size(Size::Logical(size)).map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            list_plans,
            delete_plan,
            delete_plan_by_id,
            tasks_by_date,
            create_task,
            update_task,
            delete_task,
            delete_task_by_id,
            complete_task,
            uncomplete_task,
            generate_plan,
            reschedule_tasks,
            list_resources,
            create_resource,
            update_resource,
            delete_resource,
            get_stats,
            create_backup,
            list_backups,
            restore_backup,
            delete_backup,
            send_chat,
            run_dreamwalk_if_needed,
            clear_memory,
            minimize_window,
            close_window,
            set_widget_expanded
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_store() -> Store {
        Store::new(tempdir().unwrap().keep()).unwrap()
    }

    #[test]
    fn initializes_json_and_markdown_files() {
        let store = test_store();
        assert!(store.config_path().exists());
        assert!(store.todos_path().exists());
        assert!(store.library_path().exists());
        assert!(store.long_memory_path().exists());
    }

    #[test]
    fn task_crud_and_stats_work() {
        let store = test_store();
        let task = store
            .create_task(TaskCreate {
                plan_id: None,
                task: "学习 Rust".to_string(),
                date: "2026-05-06".to_string(),
                estimated_time: 60,
                difficulty: "hard".to_string(),
                priority: "medium".to_string(),
                tags: vec!["rust".to_string()],
                notes: String::new(),
            })
            .unwrap();
        let stats = store.stats().unwrap();
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.pending, 1);
        assert!(store
            .update_task(
                &task.id,
                TaskUpdate {
                    status: Some("completed".to_string()),
                    ..Default::default()
                }
            )
            .unwrap());
        let stats = store.stats().unwrap();
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.completion_rate, 100.0);
        assert!(store.delete_task(&task.id).unwrap());
    }

    #[test]
    fn markdown_frontmatter_round_trips_date() {
        let store = test_store();
        let date = NaiveDate::from_ymd_opt(2026, 5, 6).unwrap();
        store.write_long_memory(Some(date), "# 记忆").unwrap();
        assert_eq!(store.last_dreamwalk_date().unwrap(), Some(date));
    }

    #[test]
    fn backup_cleanup_keeps_ten() {
        let store = test_store();
        for index in 0..12 {
            store
                .create_backup(format!("test backup {index}"))
                .unwrap();
        }
        assert!(store.list_backups().unwrap().len() <= 10);
    }

    #[test]
    fn invalid_ai_json_reports_error() {
        assert!(extract_json("not json").is_err());
        assert!(extract_json("```json\n{\"ok\":true}\n```").is_ok());
    }
}
