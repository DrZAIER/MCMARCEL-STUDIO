# McMarcel Studio

[![CI](https://github.com/DrZAIER/MCMARCEL-STUDIO/actions/workflows/ci.yml/badge.svg)](https://github.com/DrZAIER/MCMARCEL-STUDIO/actions/workflows/ci.yml)
[![Pages](https://github.com/DrZAIER/MCMARCEL-STUDIO/actions/workflows/pages.yml/badge.svg)](https://drzaier.github.io/MCMARCEL-STUDIO/)

A local-first desktop application for **country-configurable RSV burden and cost-effectiveness
modelling**. It is a Rust rewrite of the McMarcel R framework, wrapped in a React GUI, with a
local SQLite store and an evidence gate that refuses to present synthetic numbers as country
estimates.

> **Scientific contract.** Every parameter carries an evidence status. A pack containing
> `MISSING` or `SYNTHETIC` evidence cannot run in production mode, and every fixture run is
> stamped `SYNTHETIC FIXTURE — NOT FOR POLICY USE`. The demo pack shipped with the application
> is deliberately synthetic Tunisia data.

> **Repository layout.** This directory is the Studio. Inside the McMarcel project it lives in
> `mcmarcel-rs/`; as a repository of its own it is the root. The commands below assume the
> directory that holds `Cargo.toml` — run them from here.

The **R implementation of McMarcel is the validation oracle** and is not part of this
repository. The scripts under `parity/` read its input files; they locate it through the
`MCMARCEL_R_ROOT` environment variable or a sibling checkout.

---

## Quick start

### 1. Build the engine and the interface

```bash
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

# Export the country-pack questionnaire as a standalone toolkit
cargo run --release -p mcmarcel-cli -- questionnaire --out docs/questionnaire
```

### 3. Desktop shell (optional)

The Tauri v2 shell starts the same local server and points a native window at it:

```bash
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
| `packs` | Country packs: the synthetic demo, the Belgian reference case and the 58 Gavi countries |
| `countries` | The Gavi country catalogue (ISO3, currency, region, source and year) |
| `parity` | R oracle scripts, the parity fixtures and the Belgium reference test data |
| `docs` | Architecture, model, evidence, pack-schema, questionnaire and Gavi documentation |

## The packs that ship with Studio

| Pack | What it is | Use it for |
|------|------------|-------------|
| `tun-demo` — Tunisia (synthetic demo) | Entirely generated numbers | Exploring the interface; every run is stamped |
| `bel-belx` — Belgium (BELX reference) | The reference case of the framework, built from the repository's Belgian input files | Reproducing and validating the model against the upstream fixture |
| `packs/gavi/*.json` — 58 Gavi countries | The Gavi-eligible countries of the 2020 list, with the life table, the births, the life expectancy and the GDP per capita derived from public sources | Starting a country study: everything else waits for country evidence |

All of them are **synthetic in the sense that matters**: the Belgian input files in this
repository carry stochastic perturbation, and the Gavi packs have no incidence, cost or efficacy
evidence yet. Production mode stays blocked and every run is stamped. They are methodological
starting points, not country estimates.

The packs are rebuilt from the R inputs with:

```bash
Rscript mcmarcel-rs/parity/build_belgium_pack.R
Rscript mcmarcel-rs/parity/build_gavi_catalogue.R mcmarcel-rs/countries/gavi-2020.csv
Rscript mcmarcel-rs/parity/build_gavi_packs.R
```

See [docs/gavi-countries.md](docs/gavi-countries.md) for what the R implementation supports
for other countries, and what the Studio derives versus what an analyst has to source.

## Documentation

| Document | Purpose |
|----------|---------|
| [docs/architecture.md](docs/architecture.md) | Crates, data flow, API surface, security posture |
| [docs/model.md](docs/model.md) | The cohort model, sampling, discounting and cost-effectiveness analysis |
| [docs/evidence-model.md](docs/evidence-model.md) | Evidence statuses, production gates, fixture stamping |
| [docs/pack-schema.md](docs/pack-schema.md) | Country pack JSON reference and the path to production |
| [docs/questionnaire.md](docs/questionnaire.md) | The country-pack questionnaire: the guided wizard and the standalone toolkit |
| [docs/gavi-countries.md](docs/gavi-countries.md) | The Gavi-eligible countries: what the R model supports and what the Studio derives |

## The website

The static part of the project is published on GitHub Pages:
**<https://drzaier.github.io/MCMARCEL-STUDIO/>**

It carries the documentation, the **interactive questionnaire** (fill it in on any device, export
it as JSON, CSV or Markdown and import the JSON into the desktop application) and the published
results of the Belgian reference case, regenerated by the engine on every build.

The application itself is not hosted: the engine, the SQLite store and the production gate run
on your machine. A model whose country pack leaves the machine is a different product with
different obligations.

To build the site locally:

```bash
cd ui && npm install
npm run site          # → site/dist
```

## Building a country pack

The application does not ask an analyst to guess which numbers a country needs. The
**questionnaire** is the explicit contract: 77 parameters in seven sections, each with a
definition, its unit, why it matters, how to source it, the known sources of the reference
implementation and the evidence status that fits.

* **In the interface:** `Build a pack` in the sidebar opens the guided wizard, which walks
  through the sections and reports the progress and the number of blocking questions.
* **Offline:** the same content is exported as a printable Markdown document and a CSV:

  ```bash
  cargo run --release -p mcmarcel-cli -- questionnaire --out docs/questionnaire
  ```

  The exported files live in [docs/questionnaire/](docs/questionnaire/) and are generated from
  the same catalogue as the wizard, so the two cannot drift apart.

## The application in five screens

1. **Overview** — engine snapshot, recent runs, evidence posture of the active pack.
2. **Countries & scenarios** — grouped parameters, scenario design, provenance table, validation.
3. **Run model** — draws, seed, run mode, and an explicit production gate.
4. **Results** — draw-level cost-effectiveness plane, ICER, INMB, P\* tables, audit trail.
5. **Methods & evidence** — the scientific contract and the model boundary.

## Development

```bash
cargo test --workspace        # 21 tests: engine, store, R parity and the Belgian reference
cargo check --workspace
cd ui && npm run build        # type-checks and bundles the front end
```

The R implementation of the McMarcel framework is the **validation oracle**. It is never called
at runtime; parity is checked against fixed R fixtures and against the committed upstream
Belgian reference outputs (see [docs/evidence-model.md](docs/evidence-model.md)). The scripts
that build and check those fixtures live in `parity/` and locate the R project through
`MCMARCEL_R_ROOT` or a sibling checkout.

## Licence

GNU General Public License v3.0 or later — see [NOTICE](NOTICE) and the repository `LICENSE`.
