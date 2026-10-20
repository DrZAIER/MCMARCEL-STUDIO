# McMarcel Studio

A local-first desktop application for **country-configurable RSV burden and cost-effectiveness
modelling**. It is a Rust rewrite of the McMarcel R framework, wrapped in a React GUI, with a
local SQLite store and an evidence gate that refuses to present synthetic numbers as country
estimates.

> **Scientific contract.** Every parameter carries an evidence status. A pack containing
> `MISSING` or `SYNTHETIC` evidence cannot run in production mode, and every fixture run is
> stamped `SYNTHETIC FIXTURE — NOT FOR POLICY USE`. The demo pack shipped with the application
> is deliberately synthetic Tunisia data.

---

## Quick start

### 1. Build the engine and the interface

```bash
cd mcmarcel-rs
cargo build --release                 # engine, API server and CLI
cd ui && npm install && npm run build # React application
```

### 2. Run it

```bash
# Local web application (browser)
cargo run --release -p mcmarcel-server
# → http://127.0.0.1:8787

# Headless run of the synthetic demo pack
cargo run --release -p mcmarcel-cli -- demo --out packs/tunisia-demo.json
cargo run --release -p mcmarcel-cli -- run --pack packs/tunisia-demo.json --out output --draws 500
```

### 3. Desktop shell (optional)

The Tauri v2 shell starts the same local server and points a native window at it:

```bash
cd mcmarcel-rs
cargo build --release --manifest-path src-tauri/Cargo.toml
./src-tauri/target/release/mcmarcel-studio.exe
```

The desktop app stores its SQLite database in the per-user application data directory
(`%APPDATA%\org.chermid.mcmarcel.studio\` on Windows). Override it with `MCMARCEL_DB`; override
the frontend location with `MCMARCEL_UI_DIR`.

---

## Repository layout

| Path | Contents |
|------|----------|
| `crates/mcmarcel-core` | Domain types, country packs, model, RNG, engine, CEA, validation gates |
| `crates/mcmarcel-store` | SQLite schema (WAL, `STRICT`, foreign keys) and persistence |
| `crates/mcmarcel-server` | Local Axum API and static hosting of the built UI |
| `crates/mcmarcel-cli` | Headless `demo` / `validate` / `run` commands |
| `src-tauri` | Tauri v2 desktop shell (excluded from the fast core workspace build) |
| `ui` | React 19 + TypeScript + Vite front end |
| `packs` | Exported country packs, including the synthetic demo |
| `docs` | Architecture, model, evidence and pack-schema documentation |

## Documentation

| Document | Purpose |
|----------|---------|
| [docs/architecture.md](docs/architecture.md) | Crates, data flow, API surface, security posture |
| [docs/model.md](docs/model.md) | The cohort model, sampling, discounting and cost-effectiveness analysis |
| [docs/evidence-model.md](docs/evidence-model.md) | Evidence statuses, production gates, fixture stamping |
| [docs/pack-schema.md](docs/pack-schema.md) | Country pack JSON reference and the path to production |

## The application in five screens

1. **Overview** — engine snapshot, recent runs, evidence posture of the active pack.
2. **Countries & scenarios** — grouped parameters, scenario design, provenance table, validation.
3. **Run model** — draws, seed, run mode, and an explicit production gate.
4. **Results** — draw-level cost-effectiveness plane, ICER, INMB, P\* tables, audit trail.
5. **Methods & evidence** — the scientific contract and the model boundary.

## Development

```bash
cargo test --workspace        # 11 engine tests, store round-trip, CLI behaviour
cargo check --workspace
cd ui && npm run build        # type-checks and bundles the front end
```

The R implementation in the parent directory remains the **validation oracle**. It is never called
at runtime; parity is checked against fixed R fixtures (see [docs/evidence-model.md](docs/evidence-model.md)).

## Licence

GNU General Public License v3.0 or later — see [NOTICE](NOTICE) and the repository `LICENSE`.
