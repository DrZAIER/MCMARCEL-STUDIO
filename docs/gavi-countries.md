# The Gavi countries in McMarcel Studio

The McMarcel framework was conceived to study maternal vaccination and monoclonal antibody
prevention in the **Gavi-eligible countries** (the README of the R project refers to the 73
countries Gavi supported in 2018; Li et al. 2020 applied the model to 72 of them). This page
records what the R implementation actually supports, what McMarcel Studio does with it, and what
still has to be supplied by an analyst.

## What the R implementation supports

The R model is country-agnostic by construction. `get_rsv_ce_config()` reads a configuration row
and resolves:

| Input | Where it comes from |
|-------|---------------------|
| `population_code`, `year` | the scenario row of the configuration workbook (`config/RSV_BELX_basecase.xlsx`) |
| births, stillbirth rate, pre-term rate | `input/<country>_country_details.csv`, keyed by `population_code` and `year` |
| hospital and ICU admissions by age and calendar month | per-country input files |
| outpatient incidence, non-medically-attended cases, case fatality | per-country input files |
| unit costs by perspective and region | per-country cost files |
| **life table** | `get_life_table(country_iso, year)`: UN World Population Prospects 2019 mortality (`wpp2019`), combined with the EQ-5D population norms of `input/eq5d/popnormINDEX.rds` |

So the country machinery of the R implementation is generic: any country that has a
`population_code` in the configuration and a life table in the WPP data can be run. What is
*not* generic is the **data**: the public release of this repository only ships the Belgian
input files, so the other countries cannot be reproduced from it.

Two findings came out of the investigation:

* **The WPP life table is the one part that is genuinely global.** The `wpp2019` package covers
  every country, and the R implementation builds a monthly life table from it. That is the part
  McMarcel Studio can generate for any country.
* **The upstream country dictionary disagrees with the WPP data for three countries.** The
  dictionary in `input/dictionary_UN_countrynames.rds` uses the long UN names
  ("Dem. People's Republic of Korea", "Democratic Republic of the Congo",
  "Lao People's Democratic Republic") while the WPP data abbreviates them
  ("Dem. People's Rep. of Korea", "Dem. Republic of the Congo", "Lao People's Dem. Republic").
  `get_life_table()` looks the country up by the dictionary name, so it returns NA for exactly
  those three. The Gavi pack builder works on a copy of the input tree with a patched
  dictionary; the R code itself is untouched.

## What the Studio does with it

`countries/gavi-2020.csv` is the catalogue: the 58 countries that were eligible for Gavi
support in 2020, taken from the Gavi eligibility page (archived on 15 August 2020), each with
its ISO3 code, its UN name, its currency and its region.

`packs/gavi/` holds one country pack per catalogue entry. Each pack is a *starting point*, not
a result:

| Parameter | Status | Source |
|-----------|--------|--------|
| `births` | `derived` | UN WPP 2019 medium variant: total fertility rate × women aged 15–49 ÷ 35 |
| `life_expectancy_years` | `derived` | the life table asset of the same pack |
| life table (asset) | `derived` | UN WPP 2019 mortality, built with the R implementation's own `get_life_table()` |
| `qol_mean` | `transfer` | the EQ-5D Belgian population norms — the same norms the R implementation applies to every country, which its own code warns about |
| `gdp_per_capita`, `gdp_fx_usd` | `ok` | World Bank WDI (current local currency, and the implied exchange rate) |
| `currency` | `ok` | ISO 4216 |
| discount rates, perspective | `convention` | the upstream McMarcel defaults |
| everything else | `missing` | **country evidence is required** |

The estimates are deliberately labelled. The births estimate is accurate to about 5 % for
high-fertility countries (India 22.7 M against 23.0 M published; Nigeria 7.7 M against 7.4 M)
and drifts for low-fertility ones, where the fertility is concentrated in a few years of age
(Belgium 122 k against 108,680 registered; Tunisia 181 k against 126,401). The national birth
registration is the value an analyst should enter.

**Nothing is fabricated.** Incidence, severity, costs, utilities, efficacy, coverage and prices
stay `MISSING`, so every Gavi pack is blocked from production mode and every run made with it
is stamped. A run fails with a clear message ("missing parameter: cost_outpatient") instead of
inventing numbers.

## Working with a Gavi pack in the Studio

1. Open **Countries** and pick the country (the library filters by name or ISO3 code).
2. The editor opens on the parameter list: the derived values are filled in and carry their
   source; everything else is empty and marked `MISSING`.
3. Work through the sections — epidemiology, costs, utilities, the two interventions, the
   analysis conventions — and use **Build a pack** (the questionnaire) for the definitions, the
   sourcing guidance and the known sources of the reference implementation.
4. Add the intervention scenarios: the pack ships with the comparator arm only, because
   coverage and price are country decisions.
5. When every gated parameter carries a value, a status and a source, the pack becomes
   production-ready and the run is no longer stamped.

## Regenerating the catalogue and the packs

```bash
# The catalogue (58 countries, with the ISO3, currency and UN name resolved)
Rscript mcmarcel-rs/parity/build_gavi_catalogue.R mcmarcel-rs/countries/gavi-2020.csv

# One pack per country
Rscript mcmarcel-rs/parity/build_gavi_packs.R
```

The generator needs the R implementation (for the life table), the `wpp2019` and `countrycode`
packages, and network access to the World Bank API.

## What would still be needed for a full study

The packs above are the *derivable* part. A publishable country analysis additionally needs,
per country:

* hospital and ICU admissions by age and calendar month (or an incidence model);
* outpatient incidence and the non-medically-attended proportion;
* age-specific case fatality;
* unit costs by perspective, with the uncertainty behind them;
* QALY losses per case category;
* efficacy, protection duration, coverage and prices for the maternal vaccine and the monoclonal
  antibody.

That evidence is exactly what the questionnaire asks for, and the Gavi packs are the place to
record it: the same pack, filled in.
