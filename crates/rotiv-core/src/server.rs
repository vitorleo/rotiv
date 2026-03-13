use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderName, HeaderValue, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
};
use rotiv_orm::auto_migrate;

use crate::error::RotivError;
use crate::models::discover_models;
use crate::proxy::{InvokeRequest, invoke_route};
use crate::router::{SharedRegistry, new_shared_registry};
use crate::watcher::{FileWatcher, WatchEvent};
use crate::worker::RouteWorker;

/// Configuration for the development server.
pub struct DevServerConfig {
    pub port: u16,
    pub host: String,
    pub project_dir: PathBuf,
    pub worker_port: u16,
    pub json_output: bool,
}

/// Shared application state passed to axum handlers.
#[derive(Clone)]
struct AppState {
    registry: SharedRegistry,
    worker_port: u16,
    client: Arc<reqwest::Client>,
}

/// The Rotiv development server.
pub struct DevServer {
    config: DevServerConfig,
}

impl DevServer {
    pub fn new(config: DevServerConfig) -> Self {
        Self { config }
    }

    pub async fn start(self) -> Result<(), RotivError> {
        let routes_dir = self.config.project_dir.join("app").join("routes");

        // --- Step 1: Discover routes ---
        let registry = new_shared_registry(routes_dir.clone());
        {
            let mut reg = registry.write().await;
            reg.load()?;
            print_routes(&reg.entries(), self.config.json_output);
        }

        // --- Step 2: Start route worker ---
        let worker_port = find_available_port(self.config.worker_port).await;
        let mut worker = RouteWorker::new(self.config.project_dir.clone(), worker_port)?;
        worker.start().await?;

        if self.config.json_output {
            println!(r#"{{"event":"worker_starting","worker_port":{worker_port}}}"#);
        } else {
            println!("  worker    starting on :{worker_port}...");
        }

        worker
            .wait_ready(Duration::from_secs(15))
            .await
            .map_err(|e| {
                // Kill worker before returning error
                let _ = worker.stop();
                e
            })?;

        if self.config.json_output {
            println!(r#"{{"event":"worker_ready","worker_port":{worker_port}}}"#);
        } else {
            println!("  worker    ready");
        }

        // Auto-migrate if app/models/ exists (non-fatal — dev server continues on error)
        let models_dir = self.config.project_dir.join("app").join("models");
        if models_dir.exists() {
            match auto_migrate(&self.config.project_dir) {
                Ok(result) if result.migrations_applied > 0 => {
                    if self.config.json_output {
                        let n = result.migrations_applied;
                        println!(r#"{{"event":"migrated","migrations_applied":{n}}}"#);
                    } else {
                        println!("  migrate   {} migration(s) applied", result.migrations_applied);
                    }
                }
                Ok(_) => {
                    if !self.config.json_output {
                        println!("  migrate   up to date");
                    }
                }
                Err(e) => {
                    eprintln!("  [migrate] warning: {e}");
                }
            }
        }

        // Print model count (non-fatal if models dir doesn't exist)
        if let Ok(models) = discover_models(&self.config.project_dir) {
            if !models.is_empty() {
                if self.config.json_output {
                    let count = models.len();
                    println!(r#"{{"event":"models_loaded","count":{count}}}"#);
                } else {
                    println!("  models    {} model(s) found", models.len());
                }
            }
        }

        let worker = Arc::new(tokio::sync::Mutex::new(worker));

        // --- Step 3: Build axum router ---
        let client = Arc::new(reqwest::Client::new());
        let state = AppState {
            registry: registry.clone(),
            worker_port,
            client,
        };

        let app = Router::new()
            .route("/", any(route_handler))
            .route("/{*path}", any(route_handler))
            .with_state(state);

        // --- Step 4: Start file watcher in background task ---
        let watcher_registry = registry.clone();
        let watcher_worker = worker.clone();
        let project_dir = self.config.project_dir.clone();
        let json_output = self.config.json_output;

        tokio::spawn(async move {
            let watcher = match FileWatcher::new(&routes_dir) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("  [watcher] failed to start: {}", e.message);
                    return;
                }
            };

            loop {
                // Use try_recv + async sleep to avoid blocking the tokio thread.
                match watcher.try_recv() {
                    Some(event) => {
                        let path_str = match &event {
                            WatchEvent::Changed(p) | WatchEvent::Created(p) | WatchEvent::Deleted(p) => {
                                p.strip_prefix(&project_dir)
                                    .unwrap_or(p)
                                    .display()
                                    .to_string()
                            }
                        };

                        if json_output {
                            println!(r#"{{"event":"file_changed","file":"{path_str}"}}"#);
                        } else {
                            println!("\n  [watch]   {} changed", path_str);
                        }

                        // Reload registry on any change
                        let mut reg = watcher_registry.write().await;
                        if let Err(e) = reg.reload() {
                            eprintln!("  [watcher] route reload failed: {}", e.message);
                        } else if json_output {
                            let route_count = reg.entries().len();
                            println!(r#"{{"event":"routes_reloaded","count":{route_count}}}"#);
                        } else {
                            println!("  [watch]   routes reloaded ({} routes)", reg.entries().len());
                        }
                        drop(reg);

                        // Restart worker
                        let mut w = watcher_worker.lock().await;
                        w.stop().await;
                        if let Err(e) = w.start().await {
                            eprintln!("  [worker]  restart failed: {}", e.message);
                        } else {
                            match w.wait_ready(Duration::from_secs(10)).await {
                                Ok(_) => {
                                    if json_output {
                                        println!(r#"{{"event":"worker_restarted"}}"#);
                                    } else {
                                        println!("  [worker]  restarted");
                                    }
                                }
                                Err(e) => eprintln!("  [worker]  failed to become ready: {}", e.message),
                            }
                        }
                    }
                    None => {
                        // No event — yield to other tasks before polling again
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                }
            }
        });

        // --- Step 5: Register Ctrl+C handler ---
        let shutdown_worker = worker.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            if json_output {
                println!(r#"{{"event":"shutting_down"}}"#);
            } else {
                println!("\n  Stopping...");
            }
            let mut w = shutdown_worker.lock().await;
            w.stop().await;
            std::process::exit(0);
        });

        // --- Step 6: Bind and serve ---
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
            RotivError::new("E_BIND", format!("failed to bind to {addr}: {e}"))
                .with_suggestion(format!(
                    "Try a different port with --port, or check if port {} is already in use",
                    self.config.port
                ))
        })?;

        if self.config.json_output {
            println!(
                r#"{{"event":"listening","url":"http://{}"}}"#,
                addr
            );
        } else {
            println!("  listening on http://{}", addr);
            println!("  watching   app/routes/ (Ctrl+C to stop)\n");
        }

        axum::serve(listener, app)
            .await
            .map_err(|e| RotivError::new("E_SERVER", e.to_string()))
    }
}

/// Axum catch-all handler — resolves route from registry, proxies to worker.
async fn route_handler(State(state): State<AppState>, req: Request<Body>) -> Response {
    let path = req.uri().path().to_string();
    let method = req.method().to_string();
    let search_params = req
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();

    // Collect request headers
    let headers: HashMap<String, String> = req
        .headers()
        .iter()
        .filter_map(|(k, v)| {
            v.to_str()
                .ok()
                .map(|v| (k.to_string(), v.to_string()))
        })
        .collect();

    // Read body if present
    let body = match axum::body::to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(bytes) if !bytes.is_empty() => Some(String::from_utf8_lossy(&bytes).to_string()),
        _ => None,
    };

    // Look up route in registry
    let registry = state.registry.read().await;
    let entry = match registry.find_by_path(&path) {
        Some(e) => e.clone(),
        None => {
            return (StatusCode::NOT_FOUND, format!("No route for {}", path)).into_response();
        }
    };

    let params = registry.extract_params(&entry, &path);
    drop(registry);

    let invoke_req = InvokeRequest {
        route_file: entry.file_path.display().to_string(),
        method,
        params,
        search_params,
        headers,
        body,
    };

    match invoke_route(&state.client, state.worker_port, invoke_req).await {
        Ok(resp) => build_response(resp.status, resp.headers, resp.body),
        Err(e) => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let body = serde_json::json!({ "error": e }).to_string();
            (status, [("content-type", "application/json")], body).into_response()
        }
    }
}

fn build_response(
    status: u16,
    headers: HashMap<String, String>,
    body: String,
) -> Response {
    let status_code = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut builder = axum::http::Response::builder().status(status_code);

    for (k, v) in &headers {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(k.as_bytes()),
            HeaderValue::from_str(v),
        ) {
            builder = builder.header(name, value);
        }
    }

    builder
        .body(Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// Try to find an available port starting from `preferred`.
async fn find_available_port(preferred: u16) -> u16 {
    for port in preferred..preferred + 10 {
        if tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
        {
            return port;
        }
    }
    preferred // fallback — will fail at bind time with a clear error
}

fn print_routes(entries: &[crate::router::RouteEntry], json: bool) {
    if json {
        let routes: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "path": e.route_path,
                    "file": e.file_path.display().to_string(),
                })
            })
            .collect();
        println!("{}", serde_json::json!({"event": "routes_discovered", "routes": routes}));
    } else {
        println!("\n  Rotiv dev server");
        if entries.is_empty() {
            println!("  routes    none (add files to app/routes/)");
        } else {
            for entry in entries {
                let label = if entry.is_api_only { "API" } else { "GET" };
                let file = entry
                    .file_path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                println!(
                    "  {}  {}  →  app/routes/{}",
                    label, entry.route_path, file
                );
            }
        }
    }
}
