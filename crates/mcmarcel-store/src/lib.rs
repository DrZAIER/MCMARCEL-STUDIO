use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use mcmarcel_core::types::{CountryPack, RunMode, RunResult};

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("model error: {0}")]
    Model(#[from] mcmarcel_core::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("store lock poisoned")]
    Poisoned,
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryRecord {
    pub id: String,
    pub name: String,
    pub iso3: String,
    pub mode: RunMode,
    pub pack: CountryPack,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryListItem {
    pub id: String,
    pub name: String,
    pub iso3: String,
    pub mode: RunMode,
    pub scenario_count: usize,
    pub parameter_count: usize,
    pub updated_at_ms: i64,
    pub production_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub id: String,
    pub country_id: String,
    pub country_name: String,
    pub status: String,
    pub draws: u32,
    pub seed: u64,
    pub mode: RunMode,
    pub created_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub error: Option<String>,
    pub result: Option<RunResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunListItem {
    pub id: String,
    pub country_id: String,
    pub country_name: String,
    pub status: String,
    pub draws: u32,
    pub seed: u64,
    pub mode: RunMode,
    pub created_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub error: Option<String>,
    pub pack_name: Option<String>,
    pub fixture_stamp: Option<String>,
}

pub struct Store {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        let store = Self { conn: Mutex::new(conn), path };
        store.migrate()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let store = Self { conn: Mutex::new(conn), path: PathBuf::from(":memory:") };
        store.migrate()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| StoreError::Poisoned)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.connection()?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS countries (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                iso3            TEXT NOT NULL,
                mode            TEXT NOT NULL CHECK (mode IN ('fixture','production')),
                pack_json       TEXT NOT NULL,
                created_at_ms   INTEGER NOT NULL,
                updated_at_ms   INTEGER NOT NULL
            ) STRICT;

            CREATE TABLE IF NOT EXISTS runs (
                id              TEXT PRIMARY KEY,
                country_id      TEXT NOT NULL REFERENCES countries(id) ON DELETE CASCADE,
                status          TEXT NOT NULL CHECK (status IN ('running','complete','failed')),
                draws           INTEGER NOT NULL,
                seed            INTEGER NOT NULL,
                mode            TEXT NOT NULL CHECK (mode IN ('fixture','production')),
                created_at_ms   INTEGER NOT NULL,
                completed_at_ms INTEGER,
                error           TEXT,
                result_json     TEXT
            ) STRICT;

            CREATE TABLE IF NOT EXISTS audit_events (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                ts_ms           INTEGER NOT NULL,
                action          TEXT NOT NULL,
                country_id      TEXT,
                run_id          TEXT,
                detail          TEXT
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_runs_country_created
                ON runs(country_id, created_at_ms DESC);
            CREATE INDEX IF NOT EXISTS idx_runs_created
                ON runs(created_at_ms DESC);
            CREATE INDEX IF NOT EXISTS idx_audit_ts
                ON audit_events(ts_ms DESC);
            "#,
        )?;
        Ok(())
    }

    pub fn save_country(&self, pack: &CountryPack, mode: RunMode) -> Result<CountryRecord> {
        let now = now_ms();
        let pack_json = serde_json::to_string(pack)?;
        let mode_text = mode_text(mode);
        let conn = self.connection()?;
        let existing_created: Option<i64> = conn
            .query_row("SELECT created_at_ms FROM countries WHERE id = ?1", params![pack.country.id], |row| row.get(0))
            .optional()?;
        let created_at = existing_created.unwrap_or(now);
        conn.execute(
            r#"
            INSERT INTO countries (id, name, iso3, mode, pack_json, created_at_ms, updated_at_ms)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                iso3 = excluded.iso3,
                mode = excluded.mode,
                pack_json = excluded.pack_json,
                updated_at_ms = excluded.updated_at_ms
            "#,
            params![pack.country.id, pack.country.name, pack.country.iso3, mode_text, pack_json, created_at, now],
        )?;
        self.audit(&conn, "country.save", Some(&pack.country.id), None, Some(&format!("{} parameters", pack.parameters.len())))?;
        Ok(CountryRecord {
            id: pack.country.id.clone(),
            name: pack.country.name.clone(),
            iso3: pack.country.iso3.clone(),
            mode,
            pack: pack.clone(),
            created_at_ms: created_at,
            updated_at_ms: now,
        })
    }

    pub fn get_country(&self, id: &str) -> Result<Option<CountryRecord>> {
        let conn = self.connection()?;
        let record = conn
            .query_row(
                "SELECT id, name, iso3, mode, pack_json, created_at_ms, updated_at_ms FROM countries WHERE id = ?1",
                params![id],
                |row| {
                    let mode_text: String = row.get(3)?;
                    let pack_json: String = row.get(4)?;
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, mode_text, pack_json, row.get::<_, i64>(5)?, row.get::<_, i64>(6)?))
                },
            )
            .optional()?;
        match record {
            None => Ok(None),
            Some((id, name, iso3, mode_text, pack_json, created_at_ms, updated_at_ms)) => {
                let pack: CountryPack = serde_json::from_str(&pack_json)?;
                Ok(Some(CountryRecord {
                    id,
                    name,
                    iso3,
                    mode: parse_mode(&mode_text),
                    pack,
                    created_at_ms,
                    updated_at_ms,
                }))
            }
        }
    }

    pub fn list_countries(&self) -> Result<Vec<CountryListItem>> {
        let conn = self.connection()?;
        let mut statement = conn.prepare(
            "SELECT id, name, iso3, mode, pack_json, updated_at_ms FROM countries ORDER BY updated_at_ms DESC, name ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?;
        let mut items = Vec::new();
        for row in rows {
            let (id, name, iso3, mode_text, pack_json, updated_at_ms) = row?;
            let pack: CountryPack = serde_json::from_str(&pack_json)?;
            let validation = mcmarcel_core::validate::validate_pack(&pack, RunMode::Production);
            items.push(CountryListItem {
                id,
                name,
                iso3,
                mode: parse_mode(&mode_text),
                scenario_count: pack.scenarios.len(),
                parameter_count: pack.parameters.len(),
                updated_at_ms,
                production_ready: validation.production_ready,
            });
        }
        Ok(items)
    }

    pub fn delete_country(&self, id: &str) -> Result<()> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM countries WHERE id = ?1", params![id])?;
        self.audit(&conn, "country.delete", Some(id), None, None)?;
        Ok(())
    }

    pub fn create_run(&self, country_id: &str, draws: u32, seed: u64, mode: RunMode) -> Result<RunRecord> {
        let country_name: String = self
            .connection()?
            .query_row("SELECT name FROM countries WHERE id = ?1", params![country_id], |row| row.get(0))
            .optional()?
            .ok_or_else(|| StoreError::NotFound(country_id.to_string()))?;
        let id = Uuid::now_v7().to_string();
        let now = now_ms();
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO runs (id, country_id, status, draws, seed, mode, created_at_ms) VALUES (?1, ?2, 'running', ?3, ?4, ?5, ?6)",
            params![id, country_id, draws as i64, seed as i64, mode_text(mode), now],
        )?;
        self.audit(&conn, "run.start", Some(country_id), Some(&id), Some(&format!("draws={draws}, seed={seed}")))?;
        Ok(RunRecord {
            id,
            country_id: country_id.to_string(),
            country_name,
            status: "running".into(),
            draws,
            seed,
            mode,
            created_at_ms: now,
            completed_at_ms: None,
            error: None,
            result: None,
        })
    }

    pub fn complete_run(&self, run_id: &str, result: &RunResult) -> Result<()> {
        let result_json = serde_json::to_string(result)?;
        let conn = self.connection()?;
        conn.execute(
            "UPDATE runs SET status = 'complete', completed_at_ms = ?1, result_json = ?2, error = NULL WHERE id = ?3",
            params![now_ms(), result_json, run_id],
        )?;
        self.audit(&conn, "run.complete", Some(&result.pack_id), Some(run_id), None)?;
        Ok(())
    }

    pub fn fail_run(&self, run_id: &str, error: &str) -> Result<()> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE runs SET status = 'failed', completed_at_ms = ?1, error = ?2 WHERE id = ?3",
            params![now_ms(), error, run_id],
        )?;
        self.audit(&conn, "run.fail", None, Some(run_id), Some(error))?;
        Ok(())
    }

    pub fn get_run(&self, id: &str) -> Result<Option<RunRecord>> {
        let conn = self.connection()?;
        let row = conn
            .query_row(
                r#"
                SELECT r.id, r.country_id, c.name, r.status, r.draws, r.seed, r.mode,
                       r.created_at_ms, r.completed_at_ms, r.error, r.result_json
                FROM runs r JOIN countries c ON c.id = r.country_id
                WHERE r.id = ?1
                "#,
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, i64>(7)?,
                        row.get::<_, Option<i64>>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, Option<String>>(10)?,
                    ))
                },
            )
            .optional()?;
        match row {
            None => Ok(None),
            Some((id, country_id, country_name, status, draws, seed, mode, created_at_ms, completed_at_ms, error, result_json)) => {
                let result = match result_json {
                    Some(json) => Some(serde_json::from_str(&json)?),
                    None => None,
                };
                Ok(Some(RunRecord {
                    id,
                    country_id,
                    country_name,
                    status,
                    draws: draws.max(0) as u32,
                    seed: seed.max(0) as u64,
                    mode: parse_mode(&mode),
                    created_at_ms,
                    completed_at_ms,
                    error,
                    result,
                }))
            }
        }
    }

    pub fn list_runs(&self, limit: usize) -> Result<Vec<RunListItem>> {
        let conn = self.connection()?;
        let mut statement = conn.prepare(
            r#"
            SELECT r.id, r.country_id, c.name, r.status, r.draws, r.seed, r.mode,
                   r.created_at_ms, r.completed_at_ms, r.error,
                   json_extract(r.result_json, '$.pack_name'),
                   json_extract(r.result_json, '$.fixture_stamp')
            FROM runs r JOIN countries c ON c.id = r.country_id
            ORDER BY r.created_at_ms DESC
            LIMIT ?1
            "#,
        )?;
        let rows = statement.query_map(params![limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, Option<i64>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<String>>(11)?,
            ))
        })?;
        let mut items = Vec::new();
        for row in rows {
            let (id, country_id, country_name, status, draws, seed, mode, created_at_ms, completed_at_ms, error, pack_name, fixture_stamp) = row?;
            items.push(RunListItem {
                id,
                country_id,
                country_name,
                status,
                draws: draws.max(0) as u32,
                seed: seed.max(0) as u64,
                mode: parse_mode(&mode),
                created_at_ms,
                completed_at_ms,
                error,
                pack_name,
                fixture_stamp,
            });
        }
        Ok(items)
    }

    pub fn audit(&self, conn: &Connection, action: &str, country_id: Option<&str>, run_id: Option<&str>, detail: Option<&str>) -> Result<()> {
        conn.execute(
            "INSERT INTO audit_events (ts_ms, action, country_id, run_id, detail) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![now_ms(), action, country_id, run_id, detail],
        )?;
        Ok(())
    }
}

fn mode_text(mode: RunMode) -> &'static str {
    match mode {
        RunMode::Fixture => "fixture",
        RunMode::Production => "production",
    }
}

fn parse_mode(text: &str) -> RunMode {
    if text == "production" { RunMode::Production } else { RunMode::Fixture }
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcmarcel_core::pack::demo_pack;

    #[test]
    fn round_trip_country() {
        let store = Store::open_in_memory().unwrap();
        let pack = demo_pack();
        store.save_country(&pack, RunMode::Fixture).unwrap();
        let loaded = store.get_country("tun-demo").unwrap().unwrap();
        assert_eq!(loaded.pack.country.iso3, "TUN");
        let list = store.list_countries().unwrap();
        assert_eq!(list.len(), 1);
    }
}
