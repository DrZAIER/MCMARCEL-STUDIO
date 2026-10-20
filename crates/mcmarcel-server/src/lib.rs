//! Local HTTP server for McMarcel Studio.
//!
//! The server is deliberately bound to `127.0.0.1` only: the desktop shell and the
//! browser-served workflow are both local-first, and the SQLite store stays on the
//! user's machine.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use mcmarcel_core::pack::demo_pack;
use mcmarcel_core::types::{CountryPack, RunMode, RunSpec, ValidationReport};
use mcmarcel_core::{engine, validate, ENGINE_VERSION};
use mcmarcel_store::{CountryListItem, CountryRecord, RunListItem, RunRecord, Store, StoreError};

/// Configuration for a local Studio instance.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// SQLite database path.
    pub db_path: PathBuf,
    /// Directory holding the built frontend (`ui/dist`).
    pub ui_dir: PathBuf,
    /// Directory holding country packs to import at startup.
    pub packs_dir: Option<PathBuf>,
    /// TCP port; `0` asks the OS for an ephemeral port.
    pub port: u16,
    /// Seed the synthetic demo pack when the library is empty.
    pub seed_demo: bool,
}

impl ServerConfig {
    /// Defaults for a checkout at the workspace root.
    pub fn from_env() -> Self {
        let db_path = std::env::var("MCMARCEL_DB")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/mcmarcel.db"));
        let ui_dir = std::env::var("MCMARCEL_UI_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::current_dir().unwrap_or_default().join("ui").join("dist")
            });
        let packs_dir = std::env::var("MCMARCEL_PACKS_DIR")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                let candidate = std::env::current_dir().unwrap_or_default().join("packs");
                candidate.is_dir().then_some(candidate)
            });
        let port = std::env::var("MCMARCEL_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8787);
        Self { db_path, ui_dir, packs_dir, port, seed_demo: true }
    }
}

/// A running Studio server.
pub struct RunningServer {
    addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
}

impl RunningServer {
    /// Address the server is listening on.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Base URL of the local application.
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Block until the server task finishes.
    pub async fn wait(self) {
        let _ = self.handle.await;
    }
}

#[derive(Clone)]
struct AppState {
    store: Arc<Store>,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_REQUEST, message: message.into() }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self { status: StatusCode::NOT_FOUND, message: message.into() }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self { status: StatusCode::INTERNAL_SERVER_ERROR, message: message.into() }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorBody { error: self.message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl From<StoreError> for ApiError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::NotFound(message) => Self::not_found(message),
            StoreError::Model(error) => Self::from(error),
            other => Self::internal(other.to_string()),
        }
    }
}

impl From<mcmarcel_core::Error> for ApiError {
    fn from(error: mcmarcel_core::Error) -> Self {
        match error {
            mcmarcel_core::Error::Validation(_)
            | mcmarcel_core::Error::MissingParameter(_)
            | mcmarcel_core::Error::Pack(_)
            | mcmarcel_core::Error::ProductionGate(_) => Self::bad_request(error.to_string()),
            other => Self::internal(other.to_string()),
        }
    }
}

/// Build the Axum router for the given store and UI directory.
pub fn build_router(store: Store, ui_dir: &PathBuf) -> Router {
    let state = AppState { store: Arc::new(store) };
    let api = Router::new()
        .route("/api/health", get(health))
        .route("/api/meta", get(meta))
        .route("/api/questionnaire", get(questionnaire))
        .route("/api/questionnaire/pack", post(questionnaire_pack))
        .route("/api/countries", get(list_countries).post(create_country))
        .route("/api/countries/{id}", get(get_country).put(update_country).delete(delete_country))
        .route("/api/countries/{id}/validate", post(validate_country))
        .route("/api/runs", get(list_runs).post(create_run))
        .route("/api/runs/{id}", get(get_run))
        .with_state(state);

    let index = ui_dir.join("index.html");
    if index.exists() {
        let static_files = ServeDir::new(ui_dir).not_found_service(ServeFile::new(&index));
        api.fallback_service(static_files)
    } else {
        api.fallback(|| async {
            (
                StatusCode::OK,
                "McMarcel Studio API is running. Build the UI with `cd ui && npm install && npm run build`, or open the Vite dev server at http://127.0.0.1:5173.",
            )
        })
    }
    .layer(axum::middleware::from_fn(security_headers))
    .layer(TraceLayer::new_for_http())
}

/// Conservative local security headers for the served application.
async fn security_headers(request: axum::extract::Request, next: axum::middleware::Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        axum::http::header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'",
        ),
    );
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        axum::http::header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

/// Start the local server and return a handle to the running instance.
pub async fn serve(config: ServerConfig) -> Result<RunningServer, Box<dyn std::error::Error>> {
    let store = Store::open(&config.db_path)?;
    if config.seed_demo && store.list_countries()?.is_empty() {
        let pack = demo_pack();
        store.save_country(&pack, RunMode::Fixture)?;
        tracing::info!("seeded synthetic Tunisia demo pack");
    }
    import_packs(&store, config.packs_dir.as_deref())?;

    let app = build_router(store, &config.ui_dir);
    let address = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = tokio::net::TcpListener::bind(address).await?;
    let addr = listener.local_addr()?;
    tracing::info!("McMarcel Studio listening on http://{addr}");
    let handle = tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            tracing::error!("server stopped: {error}");
        }
    });
    Ok(RunningServer { addr, handle })
}

/// Import every country pack in `packs_dir` that is not in the library yet.
///
/// The directory is walked one level deep, so `packs/` and `packs/gavi/` are both
/// picked up.
fn import_packs(store: &Store, packs_dir: Option<&std::path::Path>) -> Result<(), Box<dyn std::error::Error>> {
    let Some(directory) = packs_dir else { return Ok(()) };
    let Ok(entries) = std::fs::read_dir(directory) else { return Ok(()) };
    let entries: Vec<PathBuf> = entries.filter_map(|entry| entry.ok()).map(|entry| entry.path()).collect();
    let mut paths: Vec<PathBuf> = entries
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .cloned()
        .collect();
    for entry in &entries {
        if !entry.is_dir() {
            continue;
        }
        let Ok(files) = std::fs::read_dir(entry) else { continue };
        for file in files.filter_map(Result::ok) {
            let path = file.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    for path in paths {
        let pack = match CountryPack::from_json_file(&path) {
            Ok(pack) => pack,
            Err(error) => {
                tracing::warn!("skipping {}: {error}", path.display());
                continue;
            }
        };
        if store.get_country(&pack.country.id)?.is_some() {
            continue;
        }
        // Imported packs keep their own evidence: a pack with blocking evidence
        // stays in fixture mode.
        let mode = if pack.parameters.values().any(|p| p.status.blocks_production()) {
            RunMode::Fixture
        } else {
            RunMode::Production
        };
        store.save_country(&pack, mode)?;
        tracing::info!("imported country pack {} from {}", pack.country.id, path.display());
    }
    Ok(())
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        engine_version: ENGINE_VERSION,
        database: state.store.path().display().to_string(),
    })
}

async fn meta(State(state): State<AppState>) -> Json<MetaResponse> {
    Json(MetaResponse {
        name: "McMarcel Studio",
        engine_version: ENGINE_VERSION,
        database: state.store.path().display().to_string(),
        models: vec!["static monthly birth-cohort RSV model", "probabilistic CEA"],
        license: "GPL-3.0-or-later",
    })
}

/// The country-pack questionnaire: every parameter with its definition, unit,
/// sourcing guidance and known sources.
async fn questionnaire() -> Json<mcmarcel_core::questionnaire::Catalogue> {
    Json(mcmarcel_core::questionnaire::catalogue())
}

/// Build a country pack from a completed questionnaire.
async fn questionnaire_pack(
    State(state): State<AppState>,
    Json(submission): Json<mcmarcel_core::questionnaire::Submission>,
) -> Result<Json<CountryRecord>, ApiError> {
    let pack = submission.to_pack().map_err(|error| ApiError::bad_request(error.to_string()))?;
    let mode = mcmarcel_core::questionnaire::initial_mode(&pack);
    let record = state.store.save_country(&pack, mode)?;
    Ok(Json(record))
}

async fn list_countries(
    State(state): State<AppState>,
) -> Result<Json<Vec<CountryListItem>>, ApiError> {
    Ok(Json(state.store.list_countries()?))
}

#[derive(Deserialize)]
struct CreateCountryRequest {
    name: String,
    iso3: String,
    #[serde(default)]
    template: Option<String>,
    #[serde(default)]
    mode: Option<RunMode>,
}

async fn create_country(
    State(state): State<AppState>,
    Json(request): Json<CreateCountryRequest>,
) -> Result<Json<CountryRecord>, ApiError> {
    let name = request.name.trim();
    let iso3 = request.iso3.trim().to_ascii_uppercase();
    if name.is_empty() || iso3.len() != 3 {
        return Err(ApiError::bad_request("name and a three-letter ISO3 code are required"));
    }
    let id = format!("{}-{}", iso3.to_ascii_lowercase(), mcmarcel_store::now_ms());
    let mut pack = demo_pack().with_identity(id, name, &iso3);
    if request.template.as_deref() == Some("blank") {
        for parameter in pack.parameters.values_mut() {
            parameter.value = None;
            parameter.status = mcmarcel_core::types::EvidenceStatus::Missing;
            parameter.notes = Some("Awaiting a sourced country value.".into());
        }
        pack.assets = Default::default();
        pack.scenarios = vec![mcmarcel_core::types::Scenario::reference()];
        pack.notes =
            "Blank country pack. Populate parameters and assets from documented sources before running."
                .into();
    }
    let mode = request.mode.unwrap_or(RunMode::Fixture);
    let record = state.store.save_country(&pack, mode)?;
    Ok(Json(record))
}

async fn get_country(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CountryRecord>, ApiError> {
    state
        .store
        .get_country(&id)?
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("country '{id}' not found")))
}

async fn update_country(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(pack): Json<CountryPack>,
) -> Result<Json<CountryRecord>, ApiError> {
    if pack.country.id != id {
        return Err(ApiError::bad_request("pack id does not match request path"));
    }
    let mode = if pack.parameters.values().any(|p| p.status.blocks_production()) {
        RunMode::Fixture
    } else {
        RunMode::Production
    };
    let record = state.store.save_country(&pack, mode)?;
    Ok(Json(record))
}

async fn delete_country(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.store.delete_country(&id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct ValidateRequest {
    #[serde(default)]
    mode: Option<RunMode>,
}

async fn validate_country(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<ValidateRequest>,
) -> Result<Json<ValidationReport>, ApiError> {
    let record = state
        .store
        .get_country(&id)?
        .ok_or_else(|| ApiError::not_found(format!("country '{id}' not found")))?;
    let mode = request.mode.unwrap_or(record.mode);
    Ok(Json(validate::validate_pack(&record.pack, mode)))
}

async fn list_runs(State(state): State<AppState>) -> Result<Json<Vec<RunListItem>>, ApiError> {
    Ok(Json(state.store.list_runs(50)?))
}

#[derive(Deserialize)]
struct CreateRunRequest {
    country_id: String,
    #[serde(default)]
    draws: Option<u32>,
    #[serde(default)]
    seed: Option<u64>,
    #[serde(default)]
    mode: Option<RunMode>,
}

async fn create_run(
    State(state): State<AppState>,
    Json(request): Json<CreateRunRequest>,
) -> Result<Json<RunRecord>, ApiError> {
    let country = state
        .store
        .get_country(&request.country_id)?
        .ok_or_else(|| ApiError::not_found(format!("country '{}' not found", request.country_id)))?;
    let draws = request.draws.unwrap_or(500).clamp(1, 20_000);
    let seed = request.seed.unwrap_or(20_260_120);
    let mode = request.mode.unwrap_or(country.mode);
    if mode == RunMode::Production {
        validate::assert_production_ready(&country.pack)?;
    }
    let run = state.store.create_run(&country.id, draws, seed, mode)?;
    let run_id = run.id.clone();
    let spec = RunSpec { draws, seed, mode };
    let pack = country.pack;

    let result = tokio::task::spawn_blocking(move || engine::run_pack(&pack, &spec)).await;
    match result {
        Ok(Ok(result)) => {
            state.store.complete_run(&run_id, &result)?;
            let record = state
                .store
                .get_run(&run_id)?
                .ok_or_else(|| ApiError::internal("run disappeared after completion"))?;
            Ok(Json(record))
        }
        Ok(Err(error)) => {
            let message = error.to_string();
            state.store.fail_run(&run_id, &message)?;
            Err(ApiError::from(error))
        }
        Err(join_error) => {
            let message = format!("engine task failed: {join_error}");
            state.store.fail_run(&run_id, &message)?;
            Err(ApiError::internal(message))
        }
    }
}

async fn get_run(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RunRecord>, ApiError> {
    state
        .store
        .get_run(&id)?
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("run '{id}' not found")))
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    engine_version: &'static str,
    database: String,
}

#[derive(Serialize)]
struct MetaResponse {
    name: &'static str,
    engine_version: &'static str,
    database: String,
    models: Vec<&'static str>,
    license: &'static str,
}
