// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use portpicker::pick_unused_port;
use std::{
    env,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
    time::Duration,
};
use tauri::{AppHandle, Manager, State, WindowEvent};

struct BackendHandle {
    child: Child,
    port: u16,
}

struct BackendState(Mutex<Option<BackendHandle>>);

impl BackendState {
    fn set(&self, child: Child, port: u16) {
        let mut guard = self.0.lock().expect("failed to lock backend state");
        *guard = Some(BackendHandle { child, port });
    }

    fn origin(&self) -> Option<String> {
        self.0
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|backend| format!("http://127.0.0.1:{}", backend.port)))
    }

    fn shutdown(&self) {
        if let Ok(mut guard) = self.0.lock() {
            if let Some(mut backend) = guard.take() {
                let _ = backend.child.kill();
            }
        }
    }
}

#[tauri::command]
async fn ensure_backend(app: AppHandle, state: State<'_, BackendState>) -> Result<String, String> {
    if let Some(origin) = state.origin() {
        return Ok(origin);
    }

    let backend_dir = resolve_backend_dir(&app)?;
    let python = resolve_python(&backend_dir)?;
    let port = pick_unused_port().ok_or_else(|| "No free port found".to_string())?;
    let data_dir = resolve_data_dir(&app, &backend_dir)?;
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let mut cmd = Command::new(python);
    cmd.current_dir(&backend_dir);
    cmd.env("MINGDENG_DATA_DIR", &data_dir);
    cmd.env("PYTHONUNBUFFERED", "1");
    cmd.args([
        "-m",
        "uvicorn",
        "main:app",
        "--host",
        "127.0.0.1",
        "--port",
        &port.to_string(),
    ]);
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start Python backend: {e}"))?;

    // Poll the backend's health endpoint until it responds (up to 5 seconds).
    let health_url = format!("http://127.0.0.1:{port}/");
    let client = reqwest::Client::new();
    let mut ready = false;
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if client.get(&health_url).send().await.is_ok() {
            ready = true;
            break;
        }
    }
    if !ready {
        return Err("Backend did not become ready within 5 seconds".to_string());
    }

    state.set(child, port);

    Ok(format!("http://127.0.0.1:{port}"))
}

fn resolve_backend_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let packaged = resource_dir.join("backend");
        if packaged.exists() {
            return Ok(packaged);
        }
    }

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(parent) = base.parent() {
        let dev_path = parent.join("backend");
        if dev_path.exists() {
            return Ok(dev_path);
        }
    }

    let sibling = base.join("backend");
    if sibling.exists() {
        return Ok(sibling);
    }

    Err("Backend directory not found. Ensure it is bundled or present next to src-tauri.".into())
}

fn resolve_python(backend_dir: &Path) -> Result<PathBuf, String> {
    if let Ok(env_path) = env::var("MINGDENG_PYTHON") {
        return Ok(PathBuf::from(env_path));
    }

    let mut candidates = Vec::new();
    if let Some(parent) = backend_dir.parent() {
        candidates.push(parent.join("backend-venv").join("bin").join("python"));
        candidates.push(parent.join("backend-venv").join("Scripts").join("python.exe"));
    }

    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    let default = if cfg!(windows) { "python" } else { "python3" };
    Ok(PathBuf::from(default))
}

fn resolve_data_dir(app: &AppHandle, backend_dir: &Path) -> Result<PathBuf, String> {
    if let Ok(dir) = app.path().app_data_dir() {
        return Ok(dir);
    }

    Ok(backend_dir.join("data"))
}

fn main() {
    tauri::Builder::default()
        .manage(BackendState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![ensure_backend])
        .on_window_event(|window, event| {
            if matches!(event, WindowEvent::Destroyed) {
                let state = window.state::<BackendState>();
                state.shutdown();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
