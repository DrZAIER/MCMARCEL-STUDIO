//! Desktop shell for McMarcel Studio.
//!
//! The shell owns no modelling logic: it starts the local Axum server (which
//! serves the same React application and the local SQLite store) and points a
//! native window at it. Everything stays on the machine.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::mpsc;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use mcmarcel_server::{serve, ServerConfig};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcmarcel_desktop=info,mcmarcel_server=info".into()),
        )
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let config = resolve_config(&handle);
            let (tx, rx) = mpsc::channel::<String>();

            // The engine server runs on its own thread with its own runtime so a
            // long probabilistic run never blocks the UI event loop.
            std::thread::Builder::new()
                .name("mcmarcel-server".into())
                .spawn(move || {
                    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
                        Ok(runtime) => runtime,
                        Err(error) => {
                            tracing::error!("failed to start the engine runtime: {error}");
                            return;
                        }
                    };
                    runtime.block_on(async move {
                        let server = match serve(config).await {
                            Ok(server) => server,
                            Err(error) => {
                                tracing::error!("failed to start the local server: {error}");
                                let _ = tx.send(String::new());
                                return;
                            }
                        };
                        let url = server.base_url();
                        tracing::info!("McMarcel Studio is available at {url}");
                        let _ = tx.send(url);
                        server.wait().await;
                    });
                })
                .map_err(|error| format!("could not spawn the server thread: {error}"))?;

            let url = rx
                .recv()
                .map_err(|error| format!("the local server did not start: {error}"))?;
            if url.is_empty() {
                return Err("the local server failed to start".into());
            }

            let external_url = tauri::Url::parse(&url).map_err(|error| format!("invalid local URL: {error}"))?;
            WebviewWindowBuilder::new(&handle, "main", WebviewUrl::External(external_url))
            .title("McMarcel Studio")
            .inner_size(1360.0, 900.0)
            .min_inner_size(960.0, 640.0)
            .center()
            .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("McMarcel Studio failed to start");
}

/// Resolve the database and frontend locations for this installation.
fn resolve_config(app: &tauri::AppHandle) -> ServerConfig {
    let data_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("data"));
    let db_path = std::env::var("MCMARCEL_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| data_dir.join("mcmarcel.db"));

    let ui_dir = std::env::var("MCMARCEL_UI_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| resolve_ui_dir(app));

    ServerConfig { db_path, ui_dir, port: 0, seed_demo: true }
}

/// Find the built frontend: bundled resources first, then a development checkout.
fn resolve_ui_dir(app: &tauri::AppHandle) -> PathBuf {
    let mut candidates = Vec::new();
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("ui").join("dist"));
    }
    if let Ok(executable) = std::env::current_exe() {
        if let Some(directory) = executable.parent() {
            candidates.push(directory.join("ui").join("dist"));
            candidates.push(directory.join("dist"));
        }
    }
    if let Ok(current) = std::env::current_dir() {
        candidates.push(current.join("ui").join("dist"));
        candidates.push(current.join("..").join("ui").join("dist"));
    }
    candidates
        .into_iter()
        .find(|candidate| candidate.join("index.html").is_file())
        .unwrap_or_else(|| PathBuf::from("ui").join("dist"))
}
