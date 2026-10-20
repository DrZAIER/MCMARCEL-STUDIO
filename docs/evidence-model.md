# The evidence model

This is the part of McMarcel Studio that decides whether a result may be described as a country
estimate. It is enforced in the Rust engine and in the API, not only in the interface.

## Evidence statuses

Every parameter in a country pack carries exactly one status:

| Status | Meaning | Blocks production |
|--------|---------|--------------------|
| `OK` | Direct, reviewed country evidence with a citation | No |
| `DERIVED` | Calculated from documented country inputs; the derivation is recorded | No |
| `TRANSFER` | Borrowed from another setting and explicitly flagged | No |
| `CONVENTION` | A modelling choice rather than an empirical estimate (discount rate, perspective) | No |
| `MISSING` | No defensible value is available | **Yes** |
| `SYNTHETIC` | Demo or fixture data | **Yes** |

`MISSING` and `SYNTHETIC` are the only blocking statuses. Everything else is a claim the analyst
is making, and the pack records the source, a source reference, a URL and a reviewer note next to
it.

## The production gate

`validate::validate_pack(pack, RunMode::Production)` returns a report with a list of issues. Each
issue has a severity (`error`, `warning`, `info`), a stable code, a message, the parameter it
concerns and a `blocks_production` flag.

The gate covers:

* **Structure** — schema version, matrix shapes (60 × 12 hospital and ICU matrices, 60 × samples
  outpatient matrix, 60 × 2 hCFR matrix, 60 × 5 life table).
* **Presence** — the 40 parameters the engine cannot run without.
* **Type and finiteness** — numeric values that are finite.
* **Range** — positive births, wastage within 0–1, coverage within 0–1, intervention months within
  1–12, catch-up coverage within 0–1.
* **Scenario integrity** — at least one scenario, exactly one reference strategy, unique scenario
  ids.
* **Model assembly** — the pack must be able to build a complete set of model inputs.

A pack is production-ready only when it has no blocking issue. The API refuses a production run
against a pack that fails the gate and returns the blocking reasons in the error body.

## Fixture stamping

Fixture mode is not a weaker production mode — it is a different claim. When a pack contains
`MISSING` or `SYNTHETIC` evidence, or when the run is explicitly requested in fixture mode:

* the validation report carries `fixture_stamp = "SYNTHETIC FIXTURE — NOT FOR POLICY USE"`;
* the run result embeds the same stamp;
* the interface shows the stamp above the results and on the run header;
* the CLI prints the stamp to standard output and writes it to `manifest.json`.

There is no way to produce an unstamped result from a non-production-ready pack.

## The demo pack

`packs/tunisia-demo.json` is **synthetic**. Its numbers are generated, not observed, and it
exists so the application can be explored without anyone mistaking demo output for evidence. It
carries `SYNTHETIC` statuses throughout, so it is blocked from production mode from the moment
it is created.

## Taking a pack to production

1. **Start from the blank template**, not the demo: *New → Blank evidence template*.
2. **Fill the epidemiology** — births, stillbirth rate, pre-term rate, incidence parameters,
   age decay, seasonality, life expectancy.
3. **Replace the matrices** with country data where they exist. A pack that relies on generated
   assets is valid but must be described as such in the notes.
4. **Enter local costs** in local currency per case or admission, with the price year recorded in
   the country metadata.
5. **Enter utility losses** per case category, with the general population quality of life.
6. **Enter intervention evidence** — efficacy, duration, coverage, administration cost, wastage
   and the price range for both the maternal vaccine and the monoclonal antibody.
7. **Set the evidence status of every parameter**, and fill the source field with a citation.
   A parameter without a source should stay `MISSING` or be marked `TRANSFER` or `CONVENTION`
   explicitly.
8. **Validate** in the country editor. The issue list is the work list.
9. **Run** in production mode. The run is stored with its pack hash, seed and validation report.

## The R oracle

The R implementation in the parent directory is the validation oracle and is never a runtime
dependency. Parity is established on two levels.

### 1. Formula parity

`parity/oracle_case_001.R` recomputes a set of shared quantities by calling the R
implementation's own functions (`time_age_to_cohort_age`, `get_protection_by_cohort`,
`compute_unit_price`, `sample_rgamma`, `approx_age_by_sim`, `get_net_benefit`) and writes
`parity/expected/case_001.json`, recording the R version that produced it.
`crates/mcmarcel-core/tests/parity.rs` recomputes the same quantities with the Rust engine and
asserts that they agree, so the fixture is part of the normal test cycle:

```bash
cargo test -p mcmarcel-core --test parity
```

To refresh the fixture after an intentional change:

```bash
Rscript parity/oracle_case_001.R
```

### 2. The Belgian reference case

The Belgian base case is the reference of the framework: the R implementation reproduces it
byte-identically (`tests/test_regression_belgium.R`) and the committed upstream fixture
(`tests/reference/belgium_upstream/`) holds the expected numbers.

`parity/build_belgium_pack.R` reads the same Belgian input files with the same transformations
as the R model and writes `packs/belgium-belx.json`: the 60 × 12 hospital and ICU matrices, the
outpatient and non-medically-attended samples, the hCFR profile, the Belgian 2023 life table, the
age-banded unit costs, the age-specific efficacy profiles and the ten upstream scenarios.

`parity/extract_belgium_reference.R` copies the upstream means into
`parity/expected/belgium_reference.json`, and
`crates/mcmarcel-core/tests/belgium_reference.rs` runs the Rust engine on the pack and compares
it with those numbers:

```bash
cargo test -p mcmarcel-core --test belgium_reference
```

The tolerances (5 % on cases, hospital admissions and QALYs, 10 % on deaths and the incremental
cost) cover the difference between the two implementations — independent random streams, the
corrected fractional-year discounting of life expectancy — and the Monte Carlo error of the 20
upstream simulations. The intervention cost has to match to within 0.1 %, because it is
deterministic.

The comparison found two real defects in the rewrite, both now fixed and covered by the test:

* the maternal vaccine wrongly received the monoclonal antibody's catch-up coverage, which
  overstated the averted burden of every catch-up programme by about a third;
* the cost-effectiveness layer applied the extended-dominance rule before removing the simply
  dominated strategies, which mislabelled the whole frontier of a ten-scenario comparison.

### 3. The perturbed public-release inputs

The Belgian input files in this repository carry stochastic perturbation (the files with an `X`
in their name). The pack therefore marks every perturbed value as `SYNTHETIC`: production mode
stays blocked and every run carries the fixture stamp. The pack is a **methodological reference**,
not a country estimate.

The R code additionally documents the C1–C16 defect fixes that the rewrite had to preserve. The
differences that the rewrite deliberately introduces are listed in
[model.md](model.md#known-deviations-from-the-r-implementation).
