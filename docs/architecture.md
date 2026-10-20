# Architecture

McMarcel Studio is a local-first application. Every component runs on the user's machine; the
only network listener binds to `127.0.0.1`.

```
                 ┌────────────────────────────────────────────┐
                 │  React 19 + TypeScript (ui/)               │
                 │  Overview · Countries · Run · Results ·    │
                 │  Methods                                   │
                 └───────────────┬────────────────────────────┘
                                 │ JSON over HTTP (loopback)
                 ┌───────────────▼────────────────────────────┐
                 │  mcmarcel-server (Axum)                     │
                 │  /api/countries · /api/runs · /api/health  │
                 └───────┬──────────────────────┬─────────────┘
                         │                      │
        ┌────────────────▼────────┐   ┌─────────▼──────────────────┐
        │  mcmarcel-core          │   │  mcmarcel-store            │
        │  model · engine · CEA   │   │  SQLite (WAL, STRICT)      │
        │  validation gates       │   │  countries · runs · audit  │
        └─────────────────────────┘   └────────────────────────────┘
```

## Crates

### `mcmarcel-core`

The engine has no I/O dependencies and is the only place where modelling decisions live.

| Module | Responsibility |
|--------|----------------|
| `types` | `CountryPack`, `Parameter`, `Scenario`, `Matrix`, `Assets`, `RunSpec`, `RunResult`, `ValidationReport` |
| `pack` | Demo pack construction and country-pack helpers (identity, hashing, JSON I/O) |
| `model` | Assembles model inputs from a pack, generates derived assets when a pack omits them, resolves age-banded costs |
| `rng` | Deterministic ChaCha8 stream, gamma/beta/normal sampling, paired draws |
| `engine` | Monthly cohort simulation, burden, costs, QALYs, discounting, aggregation |
| `analysis` | ICER table, dominance and frontier detection, INMB grid, P\* thresholds |
| `validate` | Structural, range and evidence checks; the production gate |

The engine is deterministic: the same pack, seed and draw count always produce byte-identical
results, which is what makes the R parity fixtures meaningful.

### `mcmarcel-store`

SQLite persistence with:

* `WAL` journalling and `synchronous = NORMAL`;
* `STRICT` tables with `CHECK` constraints on enumerations;
* foreign keys with `ON DELETE CASCADE` from runs to countries;
* indices on run creation time and audit timestamps;
* parameterised statements only — no string-built SQL.

Three tables: `countries` (pack JSON + mode), `runs` (status, seed, draws, result JSON) and
`audit_events` (append-only trail of saves, deletes, run starts, completions and failures).

### `mcmarcel-server`

* `build_router(store, ui_dir)` returns the Axum router; `serve(config)` binds a listener and
  returns the address, which is how the desktop shell discovers an ephemeral port.
* Engine work runs inside `spawn_blocking` so a 20 000-draw run never blocks the event loop.
* The production gate is enforced on the server, not only in the interface: a production run
  against a pack that is not production-ready is rejected with `400`.
* Responses carry `Content-Security-Policy`, `X-Content-Type-Options: nosniff` and
  `Referrer-Policy: no-referrer`.
* On startup the server imports every country pack in `packs/` (or `MCMARCEL_PACKS_DIR`) that is
  not in the library yet, so the shipped packs — the synthetic demo and the Belgian reference —
  are available in the interface straight away.

### `mcmarcel-cli`

Headless entry point for reproducible artefacts: `demo`, `validate` and `run`. A run writes
`results.json`, `summary.csv` and `manifest.json` (engine version, pack hash, mode, draws, seed,
fixture stamp) so a result can be traced back to the exact pack that produced it.

### `src-tauri` (desktop shell)

The Tauri v2 shell contains no modelling logic. It starts `mcmarcel-server` on an ephemeral
loopback port, then opens a native window pointing at it. The webview is granted **no Tauri IPC
permissions** — the interface talks to the local API over loopback HTTP only, exactly as it does
in the browser.

## Data flow of a run

1. The user edits a pack; the interface keeps a draft and shows an *unsaved changes* badge.
2. `PUT /api/countries/{id}` stores the pack and recomputes the pack mode: any parameter with a
   blocking evidence status forces fixture mode.
3. `POST /api/countries/{id}/validate` returns the validation report, the issue list and the
   pack hash.
4. `POST /api/runs` validates the requested mode, creates a `running` row, runs the engine and
   writes the completed result (or the failure) back to the row.
5. The result carries its own validation report, warnings and fixture stamp, so a stored run
   remains self-describing even if the pack changes afterwards.

## API surface

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/api/health` | Liveness, engine version, database path |
| `GET` | `/api/meta` | Engine metadata, model list, licence |
| `GET` | `/api/countries` | Library index with production readiness |
| `POST` | `/api/countries` | Create a pack from the synthetic demo or the blank template |
| `GET` | `/api/countries/{id}` | Full country record including the pack |
| `PUT` | `/api/countries/{id}` | Save a pack (mode is recomputed server-side) |
| `DELETE` | `/api/countries/{id}` | Delete a pack and its runs |
| `POST` | `/api/countries/{id}/validate` | Validation report for a given mode |
| `GET` | `/api/runs` | Recent runs (50) |
| `POST` | `/api/runs` | Execute a probabilistic run |
| `GET` | `/api/runs/{id}` | Full run record with its result payload |

## Security posture

* The server binds to loopback only; there is no remote deployment in v1.
* No authentication is required because there is no network exposure; the PostgreSQL
  multi-user server is explicitly out of scope for v1.
* The front end never evaluates model logic; every number shown comes from the Rust engine.
* The desktop shell grants no IPC permissions to the webview, and the packaged capability file
  only allows loopback origins.
