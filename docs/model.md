# The model

The v1 engine is a **static monthly birth-cohort model**, ported from the McMarcel R framework.
It is deliberately not a dynamic transmission model: the R implementation is the reference
oracle, and the rewrite reproduces its structure rather than extending it.

## Time and population

* One birth cohort per calendar year, followed from birth to **60 months of age** (0–59).
* The cohort is followed across **12 calendar months** so that seasonality in admissions is
  represented.
* The target population is `births × (1 − stillbirth rate)`.
* Pre-term birth is a modelled parameter of the cohort, not a separate population.

## Burden

For each age month and calendar month the model derives:

* **Outpatient cases** from an age-specific outpatient rate matrix with epidemiological samples
  (one column per sample). The sampled rate is scaled by the age and calendar structure.
* **Hospital admissions** from the 60 × 12 hospital matrix, and **ICU admissions** from the
  ICU matrix scaled by the ICU share of hospital admissions.
* **Deaths** from age-specific hospital case-fatality risks (`hcfr_age`, columns: hospital, ICU).
* **Non-medically-attended** cases from the `non_ma_sample_age` matrix.

When a pack omits a matrix, the engine generates a documented default asset (this is why the
validation report flags generated assets) rather than failing silently.

## Intervention

Two intervention products are modelled, and any number of scenarios may combine them:

| | Maternal vaccine | Monoclonal antibody |
|---|---|---|
| Protection window | `duration_maternal_years` | `duration_mab_years` |
| Administration | At birth | At birth (or catch-up) |
| Price | `price_maternal_min` … `price_maternal_max` | `price_mab_min` … `price_mab_max` |
| Wastage | `wastage_maternal` | `wastage_mab` |
| Administration cost | `admin_cost_maternal` | `admin_cost_mab` |

Efficacy is sampled separately for hospital, outpatient, ICU, death and non-medically-attended
outcomes (`ve_maternal_*`, `ve_mab_*`).

Each scenario defines:

* `coverage_maternal` and `coverage_mab` (0–1);
* `intervention_start` and `intervention_end` (calendar months 1–12, wrapping when the programme
  runs across the year end);
* `coverage_catch_up` (0 = no catch-up);
* optional price overrides.

Coverage is applied **by birth cohort**: a child born inside the programme window is protected
from birth; a cohort born outside the window can be covered through catch-up, scaled by
`coverage_catch_up`.

## Costs and effects

* Medical costs averted: outpatient, hospital, ICU, non-medically-attended and wheezing costs
  multiplied by the averted cases in each category.
* Intervention costs: doses delivered × price × (1 + wastage) + administration cost per dose +
  implementation cost.
* QALYs averted: utility loss per case category × averted cases, with the utility of the general
  population (`qol_mean`) applied through the life table.
* Discounting: costs and health effects are discounted with the pack's `discount_rate_cost` and
  `discount_rate_effect`. A life table (`age_years, le, le_disc, hrqale, hrqale_disc`) supplies
  the remaining life expectancy and health-related quality of life at each age.

## Probabilistic analysis

* Each draw samples the uncertain quantities (efficacy, costs, utilities, prices) with a
  **deterministic ChaCha8** stream seeded from the run seed, so results are reproducible.
* Draws are **paired**: every strategy in a run sees the same sampled inputs, which is what
  makes the incremental distributions meaningful.
* Draw-level points feed the cost-effectiveness plane directly.

## Cost-effectiveness analysis

* **ICER table** — total cost, total effect, incremental cost and effect, ICER, and a status
  (`frontier`, `dominant`, `dominated`).
* **Frontier** — the non-dominated set, ordered by cost.
* **INMB** — `INMB = WTP × ΔQALYs − Δcost` for each threshold. Thresholds are derived from GDP
  per capita and the pack's willingness-to-pay multipliers.
* **P\*** — the dose price at which INMB reaches zero at the central WTP threshold, holding the
  other sampled inputs at their means.

## Deliberate exclusions in v1

Dynamic transmission and herd immunity, EVPI/EVPPI, budget impact, caregiver spillovers and
multi-user deployment are out of scope. The interface states this on the Methods screen and in
the run panel, so a reader never has to guess what a result does not include.

## Known deviations from the R implementation

The rewrite is validated against the R code, but it is not a line-by-line transliteration. The
differences are listed here so that a reviewer never has to discover them in the numbers.

| Deviation | R implementation | Rewrite | Consequence |
|-----------|-----------------|---------|-------------|
| **Recurrent wheezing / asthma module** | Models recurrent wheezing after a first-year hospital admission, with costs and QALY losses to age 5 (asthma to 13) | Not implemented. The `cost_wheezing` and `qaly_loss_wheezing` parameters exist but are **not applied** | The engine raises the validation issue `model.wheezing.not_implemented` whenever those values are non-zero, so the omission is visible in every run |
| **Age-banded unit costs** | Under-1y and 1–4y hospital and ICU costs, five outpatient age bands, interpolated with `approx_age_by_sim` | One cost per category with a standard deviation | The rewrite cannot reproduce the R's age profile of unit costs; the parity fixture keeps the R interpolation visible as a documented deviation |
| **Caregiver QALY losses** | Optional module adding caregiver utility loss to hospital, outpatient and non-medically-attended cases | Not implemented | Results exclude caregiver spillovers (also stated on the Methods screen) |
| **Productivity costs** | Optional societal perspective with age-banded productivity loss | Not implemented; the healthcare perspective only | Results are healthcare-perspective only |
| **Fractional-year discounting of life expectancy** | `get_discounted_life_years` charges the fractional remainder at the *ceiling* year's full annuity | The fractional remainder is charged at the next year, the standard annuity approach | The R formula over-counts discounted life expectancy by roughly 40 % for fractional ages; the rewrite's value is the lower, standard one |
| **Random stream** | R's Mersenne Twister with `set.seed` | Seeded ChaCha8 stream, one stream per draw, shared by all strategies | Draw-level results are not identical to R; the *distributions* are, which is why parity is asserted on the deterministic shared machinery |
| **Wheezing probability sampling** | `rbeta` on the shape parameters from the config | Not sampled (module not implemented) | See the first row |

Everything that **is** shared is asserted against the R implementation by the parity fixture
described in [evidence-model.md](evidence-model.md).
