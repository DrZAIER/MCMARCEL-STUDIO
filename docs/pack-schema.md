# Country pack schema

A country pack is a single JSON document that holds everything the engine needs for one country.
It is the unit of exchange between the interface, the API, the CLI and the SQLite store.

```jsonc
{
  "schema_version": 1,
  "country":  { "id": "tun-demo", "name": "…", "iso3": "TUN", … },
  "parameters": { "births": { "key": "births", "label": "…", "value": 62000, … } },
  "assets":   { "hospital_age_calendar": { "rows": 60, "cols": 12, "data": [ … ] } },
  "scenarios": [ { "id": "maternal", "name": "Maternal vaccine", … } ],
  "notes": "Free-text provenance summary."
}
```

## `country`

| Field | Type | Meaning |
|-------|------|---------|
| `id` | string | Stable identifier, also the primary key in SQLite and the file name stem |
| `name` | string | Display name |
| `iso3` | string | ISO 3166-1 alpha-3 code |
| `currency` | string | ISO 4217 code used for all monetary values |
| `price_year` | integer | Year the prices refer to |
| `locale` | string | Formatting hint for the interface |
| `perspective` | string | `healthcare` or `societal` |
| `region` | string | World region label |
| `notes` | string | Free-text note |

## `parameters`

A map from parameter key to parameter object. The key is the identifier used by the engine; the
label is what the interface shows.

| Field | Type | Meaning |
|-------|------|---------|
| `key` | string | Parameter identifier (matches the map key) |
| `label` | string | Human-readable name |
| `unit` | string | Unit of the value, e.g. `local currency/dose` |
| `value` | number \| string \| null | Point estimate; `null` means "not populated" |
| `low` | number \| null | Lower bound of the sampling range (prices) |
| `high` | number \| null | Upper bound of the sampling range (prices) |
| `status` | `ok` \| `derived` \| `transfer` \| `convention` \| `missing` \| `synthetic` | Evidence status |
| `source` | string \| null | Citation or source description |
| `source_ref` | string \| null | Internal reference (DOI, table, sheet) |
| `url` | string \| null | Link to the source |
| `notes` | string \| null | Reviewer note |
| `blocks_outputs` | string[] | Outputs that must not be reported while this parameter is unresolved |

### Parameter reference

**Epidemiology and demography**

| Key | Unit | Notes |
|-----|------|-------|
| `births` | births/year | Cohort size; must be positive |
| `stillbirth_rate` | proportion | Reduces the cohort size |
| `pre_term_rate` | proportion | Share of the cohort born pre-term |
| `incidence_hospital_0_5m` | cases per infant | Hospital incidence, 0–5 months |
| `incidence_hospital_6_11m` | cases per infant | Hospital incidence, 6–11 months |
| `ratio_outpatient_hospital` | ratio | Outpatient to hospital ratio |
| `proportion_non_medical_attended` | proportion | Share of cases not medically attended |
| `icu_share_of_hospital` | proportion | ICU admissions per hospital admission |
| `cfr_hospital` | proportion | Hospital case-fatality risk |
| `cfr_icu` | proportion | ICU case-fatality risk |
| `age_decay_hospital` | per month | Age decay of the hospital rate |
| `age_decay_outpatient` | per month | Age decay of the outpatient rate |
| `age_decay_cfr` | per month | Age decay of the case-fatality risk |
| `seasonality_amplitude` | 0–0.8 | Strength of calendar seasonality |
| `life_expectancy_years` | years | Life expectancy at birth |
| `qol_mean` | utility | Mean population utility |

**Health system costs** (each with an optional `_sd` companion)

`cost_outpatient`, `cost_hospital`, `cost_icu`, `cost_non_ma`, `cost_wheezing`.

**QALY losses** (each with an optional `_sd` companion)

`qaly_loss_outpatient`, `qaly_loss_hospital`, `qaly_loss_non_ma`, `qaly_loss_wheezing`.

**Maternal vaccine**

`ve_maternal_hospital`, `ve_maternal_outpatient`, `ve_maternal_icu`, `ve_maternal_death`,
`ve_maternal_non_ma` (each with a `_sd`), `duration_maternal_years`, `coverage_maternal`,
`price_maternal_min`, `price_maternal_max`, `admin_cost_maternal`,
`implementation_cost_maternal`, `wastage_maternal`.

**Monoclonal antibody**

`ve_mab_hospital`, `ve_mab_outpatient`, `ve_mab_icu`, `ve_mab_death`, `ve_mab_non_ma` (each with a
`_sd`), `duration_mab_years`, `coverage_mab`, `price_mab_min`, `price_mab_max`, `admin_cost_mab`,
`implementation_cost_mab`, `wastage_mab`.

**Analysis conventions**

`discount_rate_cost`, `discount_rate_effect`, `gdp_per_capita`, `gdp_fx_usd`,
`wtp_multiplier_0_5`, `wtp_multiplier_1`, `wtp_multiplier_3`, `currency`,
`analysis_perspective`.

## `assets`

| Asset | Shape | Orientation |
|-------|-------|-------------|
| `hospital_age_calendar` | 60 × 12 | Row = age month (0–59), column = calendar month (0–11) |
| `icu_age_calendar` | 60 × 12 | Same as hospital |
| `outpatient_rate_age_sample` | 60 × n | Row = age month, column = epidemiological sample |
| `hcfr_age` | 60 × 2 | Column 0 = hospital, column 1 = ICU |
| `non_ma_sample_age` | n × 60 | Row = sample, column = age month |
| `life_table` | 60 × 5 | `age_years, le, le_disc, hrqale, hrqale_disc` |
| `cost_bands` | structure | Age-banded unit costs (see below) |
| `efficacy_profile` | structure | Age-specific efficacy (see below) |

Every asset is optional. When one is absent the engine generates a documented default and the
validation report records that the pack relies on a generated asset.

### `cost_bands`

Unit costs can be defined per age band, the way the Belgian reference does. The values live in
the parameter map (so they keep their evidence status and source); the structure only says which
parameter belongs to which band:

```json
"cost_bands": {
  "outpatient": [
    { "max_age_months": 11, "mean_key": "cost_outpatient_under1y", "sd_key": "cost_outpatient_under1y_sd" }
  ],
  "hospital": [ ... ],
  "icu": [ ... ],
  "non_ma": [ ... ]
}
```

The engine samples one unit cost per band and interpolates it over the age range, mirroring the
`approx_age_by_sim` logic of the R implementation: a band applies from the previous break up to
and including its own break.

### `efficacy_profile`

Efficacy can be age-specific, the way the trial-derived bootstrap files of the Belgian reference
are:

```json
"efficacy_profile": {
  "maternal": {
    "hospital": { "mean": [ ... 60 values ... ], "sd": [ ... 60 values ... ] },
    "outpatient": { ... }, "icu": { ... }, "death": { ... }, "non_ma": { ... }
  },
  "mab": { ... }
}
```

A mean of zero means "no protection at that age", which is how the protection window is encoded.
Without a profile the engine samples one value per outcome and applies it for the whole
protection window (`duration_*_years`).

## `scenarios`

| Field | Type | Meaning |
|-------|------|---------|
| `id` | string | Unique within the pack |
| `name` | string | Display name |
| `is_reference` | boolean | Exactly one scenario is the comparator |
| `coverage_maternal` | 0–1 | Maternal coverage for this programme |
| `coverage_mab` | 0–1 | mAb coverage for this programme |
| `intervention_start` | 1–12 | First calendar month of the programme |
| `intervention_end` | 1–12 | Last calendar month of the programme |
| `coverage_catch_up` | 0–1 | Catch-up coverage for cohorts born outside the window |
| `price_maternal` | number \| null | Optional price override |
| `price_mab` | number \| null | Optional price override |
| `notes` | string | Free-text note |

## Versioning

`schema_version` is currently `1`. The engine rejects a pack whose version it does not know, and
the validation report raises `schema.version` with the same effect. Bump the constant in
`crates/mcmarcel-core/src/types.rs` when the shape changes, and add a migration path rather than
silently reinterpreting old packs.

## Import and export

* The interface edits packs in place and stores them in SQLite; exports are produced by the
  headless CLI (`mcmarcel-cli demo --out …`) or by copying the JSON from the store.
* `CountryPack::from_json_file` and `CountryPack::save_json_file` are the canonical round trip.
* The pack hash (BLAKE3 over the canonical JSON) is stored with every run, so a result can always
  be traced to the exact pack that produced it.
