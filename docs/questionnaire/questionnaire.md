# McMarcel Studio — country pack questionnaire

This questionnaire lists every parameter a McMarcel Studio country pack needs, with the definition, the unit, where a defensible value comes from and how its evidence has to be classified.

Fill one row per parameter. A parameter is only usable for a production run when it carries a value, an evidence status and a source. `MISSING` and `SYNTHETIC` block production mode; the engine stamps every run made with them.

## Evidence statuses

| Status | Meaning | Blocks production |
|---|---|---|
| `ok` | direct, reviewed country evidence with a citation | no |
| `derived` | calculated from documented country inputs; the derivation is recorded | no |
| `transfer` | borrowed from another setting and explicitly flagged | no |
| `convention` | a modelling choice rather than an empirical estimate | no |
| `missing` | no defensible value available | yes |
| `synthetic` | demo or fixture data | yes |

## 1. Demography

The size and shape of the birth cohort the model follows from birth to five years.

### births — Annual births

- **Unit:** births per year
- **Definition:** The number of live births in the country in the price year of the analysis. The model follows a single birth cohort of this size from birth to 60 months of age.
- **Why it matters:** Every burden estimate scales with it: cases, admissions, deaths, costs and QALYs are all proportional to the cohort size.
- **How to source it:** National statistics office (birth registrations) or the UN World Population Prospects; a World Bank crude birth rate applied to the population of the same year also works, but say so in the source.
- **Range:** 1 to 100000000
- **Evidence:** Direct national count. Use `ok` with the statistics-office citation; `transfer` if a regional estimate stands in.
- **Drives:** cases, costs, qalys, doses
- **Known sources:**
  - input/20250106_BE_country_details.csv (target_population, BEL 2024) — The reference implementation reads the cohort size from a country-details file.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### stillbirth_rate — Stillbirth rate

- **Unit:** proportion
- **Definition:** The share of pregnancies that end in a stillbirth. The cohort that enters the model is births × (1 − stillbirth rate).
- **Why it matters:** A stillbirth rate that is too low inflates every burden estimate. It also changes the number of doses the maternal programme has to deliver, because the vaccine is given during pregnancy.
- **How to source it:** National perinatal registry or vital statistics; WHO and UNICEF country estimates are an acceptable transfer.
- **Range:** 0 to 0.5
- **Evidence:** National registry: `ok`. Regional estimate: `transfer`.
- **Drives:** cases, costs, maternal doses
- **Known sources:**
  - input/20250106_BE_country_details.csv (stillbirth_rate) — The reference implementation takes it from the same country-details file as the births.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### pre_term_rate — Pre-term birth rate

- **Unit:** proportion
- **Definition:** The share of live births before 37 weeks of gestation. The maternal vaccine is given to the mother, so a pre-term infant is effectively unprotected: maternal coverage is scaled by (1 − pre-term rate).
- **Why it matters:** Too low a value overstates the benefit of the maternal vaccine; too high a value understates it.
- **How to source it:** National perinatal registry; WHO/UNICEF estimates; the McMarcel configuration reads it as the incomplete maternal transfer rate.
- **Range:** 0 to 0.5
- **Evidence:** National registry: `ok`. Regional estimate: `transfer`.
- **Drives:** maternal efficacy
- **Known sources:**
  - input/20250106_BE_country_details.csv (incomplete_maternal_transfer_rate) — The reference implementation reads the pre-term rate from the country-details file.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### life_expectancy_years — Life expectancy at birth

- **Unit:** years
- **Definition:** Life expectancy at birth, used only when the pack does not carry a life table asset and the engine has to generate one.
- **Why it matters:** It drives the years of life lost by an RSV death and therefore the QALY loss of deaths.
- **How to source it:** National life table, or the UN World Population Prospects for the same year and country.
- **Range:** 30 to 100
- **Evidence:** Official life table: `ok`. UN estimate: `transfer`.
- **Drives:** qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### qol_mean — Mean population utility

- **Unit:** utility
- **Definition:** The average health-related quality of life of the general population. It is used to turn life expectancy into health-related QALYs when the life table has to be generated.
- **Why it matters:** Deaths are converted into QALYs with the life table, so this value scales the mortality-related part of the QALY loss.
- **How to source it:** A national or regional EQ-5D / EQ-VAS population norm set (the reference implementation uses the Belgian 2018 EQ-5D norms).
- **Range:** 0.5 to 1
- **Evidence:** Population norms: `ok` with the survey citation. Norms from another country: `transfer`.
- **Drives:** qalys
- **Known sources:**
  - input/eq5d/popnormINDEX.rds — EQ-5D Belgian general population norms (BE, 2018) used by the reference implementation.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

## 2. Epidemiology

How often RSV occurs in the first year of life, how severe it is and how it is distributed over the calendar year.

### incidence_hospital_0_5m — Hospital incidence, 0–5 months

- **Unit:** cases per infant
- **Definition:** The number of hospital admissions for RSV per infant in the first five months of life. The engine scales it to the cohort and distributes it over the calendar months with the seasonality profile.
- **Why it matters:** It is the single largest driver of the burden: every downstream outcome (deaths, costs, QALYs) is proportional to it.
- **How to source it:** National hospital discharge data (ICD-10 J21 or the local RSV code) per age in months, for at least two seasons so that a seasonal profile can be derived.
- **Range:** 0 to 0.5
- **Evidence:** National discharge data: `ok` with the extraction described. Adjusted or modelled rates: `derived` with the derivation recorded.
- **Drives:** cases, deaths, costs, qalys
- **Known sources:**
  - input/BELX_totalnrhospadmissions_byage_bycalendarmonth_perc_S2018_2019_4seasons.csv — The reference implementation reads a 60 × 12 age-by-calendar-month admission matrix and scales it to the cohort.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### incidence_hospital_6_11m — Hospital incidence, 6–11 months

- **Unit:** cases per infant
- **Definition:** The number of hospital admissions for RSV per infant between six and eleven months of age.
- **Why it matters:** The second RSV season is much smaller than the first, so this value determines how much of the burden sits outside the first year — where catch-up programmes and the protection duration matter most.
- **How to source it:** Same source as the 0–5 month incidence, split by age in months.
- **Range:** 0 to 0.5
- **Evidence:** National discharge data: `ok`. Modelled: `derived`.
- **Drives:** cases, deaths, costs, qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ratio_outpatient_hospital — Outpatient to hospital ratio

- **Unit:** ratio
- **Definition:** The number of outpatient consultations per hospital admission. The engine converts the hospital admission profile into outpatient cases and spreads them over the calendar months with the hospital seasonality.
- **Why it matters:** Outpatient cases dominate the number of cases and carry a unit cost and a QALY loss each, so the ratio drives most of the total cost.
- **How to source it:** Ambulatory (outpatient) data for the same population and period, divided by the admission count of the same period.
- **Range:** 0 to 100
- **Evidence:** National ambulatory data: `ok`. Ratio borrowed from another country: `transfer`.
- **Drives:** cases, costs, qalys
- **Known sources:**
  - input/ESPX_outpatient_incidence_1000_infant_months.csv — The reference implementation uses outpatient incidence per 1,000 infant-months, with epidemiological samples for the uncertainty.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### proportion_non_medical_attended — Non-medically-attended proportion

- **Unit:** proportion
- **Definition:** The share of medically attended RSV cases that never reach a clinician (pharmacy advice, telephone advice, no care). The engine adds these cases on top of the attended ones.
- **Why it matters:** They add cases and costs but are invisible in hospital and outpatient data, so they are often underestimated.
- **How to source it:** Household surveys, retail pharmacy sales data or sentinel primary-care data for the same period.
- **Range:** 0 to 1
- **Evidence:** National survey: `ok`. Regional pharmacy data: `transfer`.
- **Drives:** cases, costs, qalys
- **Known sources:**
  - input/RESCEUX_nonMA_cases_0-59m.csv — The reference implementation reads a sample of non-medically-attended proportions by age month.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### icu_share_of_hospital — ICU share of hospital admissions

- **Unit:** proportion
- **Definition:** The number of ICU admissions per hospital admission. Only used when the pack does not carry an ICU matrix.
- **Why it matters:** ICU cases carry the highest unit cost and the highest case-fatality risk, so this value moves the cost and the death estimates.
- **How to source it:** National ICU registry or hospital discharge data with an intensive-care flag.
- **Range:** 0 to 1
- **Evidence:** National registry: `ok`. Literature estimate: `transfer`.
- **Drives:** deaths, costs
- **Known sources:**
  - input/BELX_totalnrICUadmissions_byage_bycalendarmonth_perc_S2018_2019_4seasons.csv — The reference implementation reads a separate 60 × 12 ICU matrix.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### cfr_hospital — Hospital case-fatality risk

- **Unit:** proportion
- **Definition:** The probability that a child hospitalised with RSV dies, by age in months. The engine multiplies the admissions with this risk to obtain deaths.
- **Why it matters:** Deaths drive the life-years and health-related QALYs lost, so this value has an outsized effect on the QALY loss and on every cost-effectiveness ratio.
- **How to source it:** National hospital mortality data by age (discharge diagnosis RSV), or the published case-fatality series used by the framework.
- **Range:** 0 to 1
- **Evidence:** National mortality data: `ok`. Published series from another country: `transfer`.
- **Drives:** deaths, qalys
- **Known sources:**
  - input/BELX_hCFR_byAge.csv — The reference implementation reads the age-specific hospital and ICU case-fatality risks from one file.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### cfr_icu — ICU case-fatality risk

- **Unit:** proportion
- **Definition:** The probability that a child admitted to intensive care with RSV dies, by age in months.
- **Why it matters:** ICU deaths are the most expensive and the most damaging in QALY terms, so this value can dominate the mortality estimate.
- **How to source it:** National ICU registry or hospital mortality data by age.
- **Range:** 0 to 1
- **Evidence:** National registry: `ok`. Published series: `transfer`.
- **Drives:** deaths, qalys
- **Known sources:**
  - input/BELX_hCFR_byAge.csv — Same file as the hospital case-fatality risk.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### age_decay_hospital — Hospital age decay

- **Unit:** per month
- **Definition:** The monthly factor by which the hospital admission rate falls with age. A factor of 1 means the rate does not decay; 0.97 means three per cent less each month.
- **Why it matters:** The engine uses it to build an age profile when no admission matrix is supplied, so it shapes the whole burden curve.
- **How to source it:** Fit the decay to the national age profile of RSV admissions, or set it to 1 when a full age-by-month matrix is supplied.
- **Range:** 0.5 to 1
- **Evidence:** Fitted to national data: `derived`, with the fit described. A default: `convention`.
- **Drives:** cases, deaths, costs

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### age_decay_outpatient — Outpatient age decay

- **Unit:** per month
- **Definition:** The monthly factor by which the outpatient rate falls with age, used when no outpatient matrix is supplied.
- **Why it matters:** Together with the hospital profile it sets the shape of the outpatient burden, which is the largest cost component.
- **How to source it:** Fit to the national age profile of outpatient consultations, or set it to 1 when a matrix is supplied.
- **Range:** 0.5 to 1
- **Evidence:** Fitted to national data: `derived`. A default: `convention`.
- **Drives:** cases, costs

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### age_decay_cfr — Case-fatality age decay

- **Unit:** per month
- **Definition:** The monthly factor by which the case-fatality risk falls with age, used when no case-fatality profile is supplied.
- **Why it matters:** It changes how the deaths are distributed over the first year of life.
- **How to source it:** Fit to the national age profile of RSV deaths, or set it to 1 when an age profile is supplied.
- **Range:** 0.5 to 1
- **Evidence:** Fitted to national data: `derived`. A default: `convention`.
- **Drives:** deaths

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### seasonality_amplitude — Seasonality amplitude

- **Unit:** 0 to 0.8
- **Definition:** The strength of the calendar-month seasonality of RSV admissions. Zero is a flat year, 0.8 is a very peaked year with a December maximum.
- **Why it matters:** Seasonality decides which birth cohorts are inside a seasonal programme window, and therefore how much a catch-up programme can add.
- **How to source it:** Derive it from the national age-by-calendar-month admission matrix (or set it to 0 and supply the matrix itself).
- **Range:** 0 to 0.8
- **Evidence:** Derived from national data: `derived`. A default: `convention`.
- **Drives:** cases, programme coverage

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

## 3. Health system costs

What one case of each type costs the health system, in the currency and the price year of the analysis.

### cost_outpatient — Outpatient cost per case

- **Unit:** local currency per case
- **Definition:** The cost of one outpatient RSV consultation, in the currency of the analysis and in the price year.
- **Why it matters:** Outpatient cases are the most frequent, so this value contributes a large part of the medical cost that prevention can avert.
- **How to source it:** Insurance claims, national tariff or costing study for the price year; convert with the price-year index and state the index used.
- **Range:** 0 to 1000000
- **Evidence:** National tariff or claims for the price year: `ok` with the citation. Cost from another country: `transfer`. An international reference price: `transfer` with the country named.
- **Drives:** medical cost, incremental cost
- **Known sources:**
  - input/BELX_cost_perspectives_regions.csv (payer perspective) — The reference implementation reads the unit costs per perspective and region, with a mean and a confidence interval or standard deviation.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### cost_outpatient_sd — Outpatient cost per case, standard deviation

- **Unit:** local currency per case
- **Definition:** The standard deviation of the sampled unit cost. The engine samples the cost from a gamma distribution with this mean and standard deviation.
- **Why it matters:** A cost without uncertainty understates the width of the cost-effectiveness plane and can hide a strategy that is not robust.
- **How to source it:** Derive it from the published confidence interval of the cost as (upper − lower) / (2 × 1.96), or from the variability of the claims.
- **Range:** 0 to 1000000
- **Evidence:** Derived from a published interval: `derived`, with the interval cited. A zero standard deviation is a convention: say so.
- **Drives:** medical cost, incremental cost

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### cost_hospital — Hospital cost per admission

- **Unit:** local currency per case
- **Definition:** The cost of one hospital admission for RSV, excluding intensive care, in the currency of the analysis and the price year.
- **Why it matters:** Hospital cost is the largest cost per case and the value that the cost-effectiveness result is most sensitive to.
- **How to source it:** Insurance claims or a national tariff; state whether the cost includes the admission and the ward stay only.
- **Range:** 0 to 1000000
- **Evidence:** National tariff or claims for the price year: `ok` with the citation. Cost from another country: `transfer`. An international reference price: `transfer` with the country named.
- **Drives:** medical cost, incremental cost
- **Known sources:**
  - input/BELX_cost_perspectives_regions.csv (payer perspective) — The reference implementation reads the unit costs per perspective and region, with a mean and a confidence interval or standard deviation.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### cost_hospital_sd — Hospital cost per admission, standard deviation

- **Unit:** local currency per case
- **Definition:** The standard deviation of the sampled unit cost. The engine samples the cost from a gamma distribution with this mean and standard deviation.
- **Why it matters:** A cost without uncertainty understates the width of the cost-effectiveness plane and can hide a strategy that is not robust.
- **How to source it:** Derive it from the published confidence interval of the cost as (upper − lower) / (2 × 1.96), or from the variability of the claims.
- **Range:** 0 to 1000000
- **Evidence:** Derived from a published interval: `derived`, with the interval cited. A zero standard deviation is a convention: say so.
- **Drives:** medical cost, incremental cost

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### cost_icu — ICU cost per admission

- **Unit:** local currency per case
- **Definition:** The cost of one intensive-care admission for RSV, in the currency of the analysis and the price year.
- **Why it matters:** ICU cases are few but expensive; they dominate the tail of the cost distribution.
- **How to source it:** Insurance claims or a national tariff for intensive care.
- **Range:** 0 to 1000000
- **Evidence:** National tariff or claims for the price year: `ok` with the citation. Cost from another country: `transfer`. An international reference price: `transfer` with the country named.
- **Drives:** medical cost, incremental cost
- **Known sources:**
  - input/BELX_cost_perspectives_regions.csv (payer perspective) — The reference implementation reads the unit costs per perspective and region, with a mean and a confidence interval or standard deviation.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### cost_icu_sd — ICU cost per admission, standard deviation

- **Unit:** local currency per case
- **Definition:** The standard deviation of the sampled unit cost. The engine samples the cost from a gamma distribution with this mean and standard deviation.
- **Why it matters:** A cost without uncertainty understates the width of the cost-effectiveness plane and can hide a strategy that is not robust.
- **How to source it:** Derive it from the published confidence interval of the cost as (upper − lower) / (2 × 1.96), or from the variability of the claims.
- **Range:** 0 to 1000000
- **Evidence:** Derived from a published interval: `derived`, with the interval cited. A zero standard deviation is a convention: say so.
- **Drives:** medical cost, incremental cost

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### cost_non_ma — Non-medically-attended cost per case

- **Unit:** local currency per case
- **Definition:** The cost of managing one case that never reaches a clinician, in the currency of the analysis and the price year.
- **Why it matters:** Small per case, but these cases are frequent, so the total is not negligible.
- **How to source it:** Pharmacy costs, telephone consultation tariffs or a costing study of the non-attended pathway.
- **Range:** 0 to 1000000
- **Evidence:** National tariff or claims for the price year: `ok` with the citation. Cost from another country: `transfer`. An international reference price: `transfer` with the country named.
- **Drives:** medical cost
- **Known sources:**
  - input/BELX_cost_perspectives_regions.csv (payer perspective) — The reference implementation reads the unit costs per perspective and region, with a mean and a confidence interval or standard deviation.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### cost_non_ma_sd — Non-medically-attended cost per case, standard deviation

- **Unit:** local currency per case
- **Definition:** The standard deviation of the sampled unit cost. The engine samples the cost from a gamma distribution with this mean and standard deviation.
- **Why it matters:** A cost without uncertainty understates the width of the cost-effectiveness plane and can hide a strategy that is not robust.
- **How to source it:** Derive it from the published confidence interval of the cost as (upper − lower) / (2 × 1.96), or from the variability of the claims.
- **Range:** 0 to 1000000
- **Evidence:** Derived from a published interval: `derived`, with the interval cited. A zero standard deviation is a convention: say so.
- **Drives:** medical cost

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### cost_wheezing — Wheezing sequelae cost

- **Unit:** local currency per case
- **Definition:** The cost per child-year of recurrent wheezing and asthma after an RSV hospital admission.
- **Why it matters:** The recurrent-wheezing module is not implemented in this engine version, so this value is not applied to the results.
- **How to source it:** If you have a value, keep it and document that the engine cannot apply it yet; otherwise set it to zero and record the omission.
- **Range:** 0 to 1000000
- **Evidence:** National tariff or claims for the price year: `ok` with the citation. Cost from another country: `transfer`. An international reference price: `transfer` with the country named.
- **Known sources:**
  - input/BELX_cost_perspectives_regions.csv (payer perspective) — The reference implementation reads the unit costs per perspective and region, with a mean and a confidence interval or standard deviation.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### cost_wheezing_sd — Wheezing sequelae cost, standard deviation

- **Unit:** local currency per case
- **Definition:** The standard deviation of the sampled unit cost. The engine samples the cost from a gamma distribution with this mean and standard deviation.
- **Why it matters:** A cost without uncertainty understates the width of the cost-effectiveness plane and can hide a strategy that is not robust.
- **How to source it:** Derive it from the published confidence interval of the cost as (upper − lower) / (2 × 1.96), or from the variability of the claims.
- **Range:** 0 to 1000000
- **Evidence:** Derived from a published interval: `derived`, with the interval cited. A zero standard deviation is a convention: say so.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

## 4. QALY losses

The health-related quality of life lost per case, used to convert cases into QALYs.

### qaly_loss_outpatient — QALY loss, outpatient

- **Unit:** QALY per case
- **Definition:** The health-related quality of life lost by one outpatient RSV case, in QALYs per case.
- **Why it matters:** Outpatient cases are the most frequent, so this value determines most of the non-fatal QALY loss.
- **How to source it:** Published utility decrements for acute respiratory infection in children, adapted to the local setting, or an EQ-5D study of RSV cases.
- **Range:** 0 to 1
- **Evidence:** Published decrements for the same country: `ok` with the citation. Decrement from another country: `transfer`, with the adaptation described.
- **Drives:** qalys
- **Known sources:**
  - functions/RSV_config.R (QALYloss_hosp_mean, QALYloss_outpatient_mean, QALY_non_ma_mean) — The reference implementation hard-codes the Belgian QALY losses and derives their standard error from the published confidence interval.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### qaly_loss_outpatient_sd — QALY loss, outpatient, standard deviation

- **Unit:** QALY per case
- **Definition:** The standard deviation of the sampled utility loss, derived from the published confidence interval.
- **Why it matters:** It controls how wide the QALY uncertainty is and therefore how often a strategy changes rank.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96).
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### qaly_loss_hospital — QALY loss, hospital

- **Unit:** QALY per case
- **Definition:** The health-related quality of life lost by one hospitalised RSV case, in QALYs per case. The same value is applied to ICU cases.
- **Why it matters:** It converts the most severe and most expensive cases into QALYs.
- **How to source it:** Published utility decrements for hospitalised RSV in infants, or a local EQ-5D follow-up of hospitalised cases.
- **Range:** 0 to 1
- **Evidence:** Published decrements for the same country: `ok` with the citation. Decrement from another country: `transfer`, with the adaptation described.
- **Drives:** qalys
- **Known sources:**
  - functions/RSV_config.R (QALYloss_hosp_mean, QALYloss_outpatient_mean, QALY_non_ma_mean) — The reference implementation hard-codes the Belgian QALY losses and derives their standard error from the published confidence interval.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### qaly_loss_hospital_sd — QALY loss, hospital, standard deviation

- **Unit:** QALY per case
- **Definition:** The standard deviation of the sampled utility loss, derived from the published confidence interval.
- **Why it matters:** It controls how wide the QALY uncertainty is and therefore how often a strategy changes rank.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96).
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### qaly_loss_non_ma — QALY loss, non-medically-attended

- **Unit:** QALY per case
- **Definition:** The health-related quality of life lost by one medically unattended RSV case, in QALYs per case.
- **Why it matters:** These cases are frequent, so a plausible but wrong value can dominate the QALY loss.
- **How to source it:** Published decrements for ambulatory respiratory infection, or a survey of families of children with RSV.
- **Range:** 0 to 1
- **Evidence:** Published decrements for the same country: `ok` with the citation. Decrement from another country: `transfer`, with the adaptation described.
- **Drives:** qalys
- **Known sources:**
  - functions/RSV_config.R (QALYloss_hosp_mean, QALYloss_outpatient_mean, QALY_non_ma_mean) — The reference implementation hard-codes the Belgian QALY losses and derives their standard error from the published confidence interval.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### qaly_loss_non_ma_sd — QALY loss, non-medically-attended, standard deviation

- **Unit:** QALY per case
- **Definition:** The standard deviation of the sampled utility loss, derived from the published confidence interval.
- **Why it matters:** It controls how wide the QALY uncertainty is and therefore how often a strategy changes rank.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96).
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### qaly_loss_wheezing — QALY loss, wheezing

- **Unit:** QALY per case
- **Definition:** The health-related quality of life lost per child-year of recurrent wheezing and asthma after an RSV admission.
- **Why it matters:** The recurrent-wheezing module is not implemented in this engine version, so this value is not applied to the results.
- **How to source it:** If you have a value, keep it and document that the engine cannot apply it yet; otherwise set it to zero and record the omission.
- **Range:** 0 to 1
- **Evidence:** Published decrements for the same country: `ok` with the citation. Decrement from another country: `transfer`, with the adaptation described.
- **Drives:** qalys
- **Known sources:**
  - functions/RSV_config.R (QALYloss_hosp_mean, QALYloss_outpatient_mean, QALY_non_ma_mean) — The reference implementation hard-codes the Belgian QALY losses and derives their standard error from the published confidence interval.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### qaly_loss_wheezing_sd — QALY loss, wheezing, standard deviation

- **Unit:** QALY per case
- **Definition:** The standard deviation of the sampled utility loss, derived from the published confidence interval.
- **Why it matters:** It controls how wide the QALY uncertainty is and therefore how often a strategy changes rank.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96).
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

## 5. Maternal RSV vaccine

Efficacy, protection duration, coverage, price and delivery costs of the maternal vaccine.

### ve_maternal_hospital — Maternal RSV vaccine efficacy, hospitalisation

- **Unit:** proportion
- **Definition:** The efficacy of the product against RSV hospitalisation.
- **Why it matters:** Hospital admissions are the most expensive outcome, so this value drives the cost-effectiveness of the programme.
- **How to source it:** The pivotal trial or its pooled estimate for the same age group and outcome, with its confidence interval.
- **Range:** 0 to 1
- **Evidence:** Trial estimate for the same country or a multi-country trial including it: `ok` with the citation. Estimate from another setting: `transfer`, with the population named.
- **Drives:** averted cases, averted costs, averted qalys
- **Known sources:**
  - input/VE_maternal_CI_MA.csv — Efficacy against medically attended infection, by age, with bootstrap samples.
  - input/VE_maternal_CI_Hosp_MA.csv — Efficacy against hospitalisation, by age, with bootstrap samples.
  - input/VE_maternal_CI_Seve_MA.csv — Efficacy against the severe composite, by age, with bootstrap samples.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_maternal_hospital_sd — Maternal RSV vaccine efficacy, hospitalisation: standard deviation

- **Unit:** proportion
- **Definition:** The standard deviation of the sampled efficacy, derived from the confidence interval of the trial estimate.
- **Why it matters:** It decides how often the programme misses its expected effect, and therefore how often the cost-effectiveness result changes.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96), or from the bootstrap distribution in the efficacy file.
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** averted cases, averted costs, averted qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_maternal_outpatient — Maternal RSV vaccine efficacy, outpatient (medically attended) infection

- **Unit:** proportion
- **Definition:** The efficacy of the product against medically attended RSV infection of any severity.
- **Why it matters:** Most cases are outpatient, so this value determines the number of cases the programme averts.
- **How to source it:** The pivotal trial estimate for the same outcome and age group, with its confidence interval.
- **Range:** 0 to 1
- **Evidence:** Trial estimate for the same country or a multi-country trial including it: `ok` with the citation. Estimate from another setting: `transfer`, with the population named.
- **Drives:** averted cases, averted costs, averted qalys
- **Known sources:**
  - input/VE_maternal_CI_MA.csv — Efficacy against medically attended infection, by age, with bootstrap samples.
  - input/VE_maternal_CI_Hosp_MA.csv — Efficacy against hospitalisation, by age, with bootstrap samples.
  - input/VE_maternal_CI_Seve_MA.csv — Efficacy against the severe composite, by age, with bootstrap samples.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_maternal_outpatient_sd — Maternal RSV vaccine efficacy, outpatient (medically attended) infection: standard deviation

- **Unit:** proportion
- **Definition:** The standard deviation of the sampled efficacy, derived from the confidence interval of the trial estimate.
- **Why it matters:** It decides how often the programme misses its expected effect, and therefore how often the cost-effectiveness result changes.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96), or from the bootstrap distribution in the efficacy file.
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** averted cases, averted costs, averted qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_maternal_icu — Maternal RSV vaccine efficacy, intensive-care admission

- **Unit:** proportion
- **Definition:** The efficacy of the product against RSV intensive-care admission.
- **Why it matters:** ICU cases drive the mortality and the tail of the cost distribution.
- **How to source it:** The pivotal trial or its severe-outcome estimate for the same age group, with its confidence interval.
- **Range:** 0 to 1
- **Evidence:** Trial estimate for the same country or a multi-country trial including it: `ok` with the citation. Estimate from another setting: `transfer`, with the population named.
- **Drives:** averted cases, averted costs, averted qalys
- **Known sources:**
  - input/VE_maternal_CI_MA.csv — Efficacy against medically attended infection, by age, with bootstrap samples.
  - input/VE_maternal_CI_Hosp_MA.csv — Efficacy against hospitalisation, by age, with bootstrap samples.
  - input/VE_maternal_CI_Seve_MA.csv — Efficacy against the severe composite, by age, with bootstrap samples.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_maternal_icu_sd — Maternal RSV vaccine efficacy, intensive-care admission: standard deviation

- **Unit:** proportion
- **Definition:** The standard deviation of the sampled efficacy, derived from the confidence interval of the trial estimate.
- **Why it matters:** It decides how often the programme misses its expected effect, and therefore how often the cost-effectiveness result changes.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96), or from the bootstrap distribution in the efficacy file.
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** averted cases, averted costs, averted qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_maternal_death — Maternal RSV vaccine efficacy, death

- **Unit:** proportion
- **Definition:** The efficacy of the product against RSV death. The reference implementation sets this equal to the intensive-care efficacy.
- **Why it matters:** Deaths are the outcome with the largest quality-of-life consequence, so this value matters even though it is not observed directly in most trials.
- **How to source it:** The trial estimate for the severe composite, or the intensive-care efficacy with the reasoning recorded.
- **Range:** 0 to 1
- **Evidence:** Trial estimate for the same country or a multi-country trial including it: `ok` with the citation. Estimate from another setting: `transfer`, with the population named.
- **Drives:** averted cases, averted costs, averted qalys
- **Known sources:**
  - input/VE_maternal_CI_MA.csv — Efficacy against medically attended infection, by age, with bootstrap samples.
  - input/VE_maternal_CI_Hosp_MA.csv — Efficacy against hospitalisation, by age, with bootstrap samples.
  - input/VE_maternal_CI_Seve_MA.csv — Efficacy against the severe composite, by age, with bootstrap samples.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_maternal_death_sd — Maternal RSV vaccine efficacy, death: standard deviation

- **Unit:** proportion
- **Definition:** The standard deviation of the sampled efficacy, derived from the confidence interval of the trial estimate.
- **Why it matters:** It decides how often the programme misses its expected effect, and therefore how often the cost-effectiveness result changes.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96), or from the bootstrap distribution in the efficacy file.
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** averted cases, averted costs, averted qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_maternal_non_ma — Maternal RSV vaccine efficacy, medically unattended infection

- **Unit:** proportion
- **Definition:** The efficacy of the product against RSV infection that never reaches a clinician.
- **Why it matters:** These cases are frequent; an efficacy of zero here means the programme only prevents visible disease.
- **How to source it:** The trial estimate for the same outcome, or the all-cause efficacy with the reasoning recorded.
- **Range:** 0 to 1
- **Evidence:** Trial estimate for the same country or a multi-country trial including it: `ok` with the citation. Estimate from another setting: `transfer`, with the population named.
- **Drives:** averted cases, averted costs, averted qalys
- **Known sources:**
  - input/VE_maternal_CI_MA.csv — Efficacy against medically attended infection, by age, with bootstrap samples.
  - input/VE_maternal_CI_Hosp_MA.csv — Efficacy against hospitalisation, by age, with bootstrap samples.
  - input/VE_maternal_CI_Seve_MA.csv — Efficacy against the severe composite, by age, with bootstrap samples.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_maternal_non_ma_sd — Maternal RSV vaccine efficacy, medically unattended infection: standard deviation

- **Unit:** proportion
- **Definition:** The standard deviation of the sampled efficacy, derived from the confidence interval of the trial estimate.
- **Why it matters:** It decides how often the programme misses its expected effect, and therefore how often the cost-effectiveness result changes.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96), or from the bootstrap distribution in the efficacy file.
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** averted cases, averted costs, averted qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### duration_maternal_years — Maternal RSV vaccine protection duration

- **Unit:** years
- **Definition:** How long the protection lasts after the dose is given. The engine applies the efficacy for that many months of age.
- **Why it matters:** A duration that is too long credits the programme with protection it does not give, and a duration that is too short understates it.
- **How to source it:** The trial follow-up that supports the duration, or the regulatory label. State whether the protection wanes over time; if it does, supply an age-specific efficacy profile instead.
- **Range:** 0 to 5
- **Evidence:** Trial follow-up: `ok` with the citation. Regulatory label: `ok`.
- **Drives:** averted cases, averted costs

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### coverage_maternal — Maternal RSV vaccine coverage

- **Unit:** proportion
- **Definition:** The share of the eligible cohort that receives the product. Every scenario can override it; this value is the default.
- **Why it matters:** Coverage scales the whole benefit linearly, so it is the assumption an analyst is most often asked to defend.
- **How to source it:** The coverage assumed in the study protocol, the expected uptake in the first year, or the national immunisation coverage for a comparable programme.
- **Range:** 0 to 1
- **Evidence:** Planned programme coverage from the study protocol: `ok`. Observed coverage from another country: `transfer`.
- **Drives:** averted cases, averted costs, doses

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### price_maternal_min — Maternal RSV vaccine price, low

- **Unit:** local currency per dose
- **Definition:** The lower bound of the dose price. When the two price bounds are equal, the price is deterministic.
- **Why it matters:** The price drives the intervention cost, which is the dominant term in the incremental cost, and it drives the price threshold P*.
- **How to source it:** The confidential price from the manufacturer, the price used in the study protocol, or a published reference price. Always record the price year and whether the price is confidential.
- **Range:** 0 to 100000
- **Evidence:** Study price: `ok` with the citation. List price: `ok`. Reference price from another country: `transfer`.
- **Drives:** intervention cost, icer, pstar

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### price_maternal_max — Maternal RSV vaccine price, high

- **Unit:** local currency per dose
- **Definition:** The upper bound of the dose price, sampled uniformly between the low and the high bound.
- **Why it matters:** A price range widens the cost-effectiveness plane and shows how much the conclusion depends on the price.
- **How to source it:** The same source as the low bound; a scenario analysis range is a legitimate choice if it is documented as such.
- **Range:** 0 to 100000
- **Evidence:** Same status as the low bound.
- **Drives:** intervention cost, icer, pstar

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### admin_cost_maternal — Maternal RSV vaccine administration cost

- **Unit:** local currency per dose
- **Definition:** The cost of delivering one dose: the health-care professional time and the consumables, per dose.
- **Why it matters:** It is added to the price of every dose, so it matters most for expensive products with small cohorts.
- **How to source it:** The programme budget or the costing study of the delivery pathway.
- **Range:** 0 to 100000
- **Evidence:** Programme budget: `ok`. Unit-costing study: `ok`.
- **Drives:** intervention cost

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### implementation_cost_maternal — Maternal RSV vaccine implementation cost

- **Unit:** local currency
- **Definition:** The fixed cost of setting the programme up (training, information, cold chain), divided over the doses of the cohort.
- **Why it matters:** It is a fixed cost spread over one cohort, so it matters most for small cohorts and for expensive products.
- **How to source it:** The programme budget for the first year.
- **Range:** 0 to 1000000000
- **Evidence:** Programme budget: `ok`.
- **Drives:** intervention cost

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### wastage_maternal — Maternal RSV vaccine wastage

- **Unit:** proportion
- **Definition:** The share of doses that are lost: multi-dose vials, missed appointments, wastage at the end of the campaign.
- **Why it matters:** Wastage raises the number of doses that have to be bought, and therefore the intervention cost.
- **How to source it:** The programme's own wastage rate, or the wastage rate of a comparable immunisation programme in the same health system.
- **Range:** 0 to 1
- **Evidence:** Programme data: `ok`. Comparable programme: `transfer`.
- **Drives:** intervention cost, doses

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

## 6. Monoclonal antibody

Efficacy, protection duration, coverage, price and delivery costs of the monoclonal antibody.

### ve_mab_hospital — Monoclonal antibody efficacy, hospitalisation

- **Unit:** proportion
- **Definition:** The efficacy of the product against RSV hospitalisation.
- **Why it matters:** Hospital admissions are the most expensive outcome, so this value drives the cost-effectiveness of the programme.
- **How to source it:** The pivotal trial or its pooled estimate for the same age group and outcome, with its confidence interval.
- **Range:** 0 to 1
- **Evidence:** Trial estimate for the same country or a multi-country trial including it: `ok` with the citation. Estimate from another setting: `transfer`, with the population named.
- **Drives:** averted cases, averted costs, averted qalys
- **Known sources:**
  - input/VE_mab_CI_MA.csv — Efficacy against medically attended infection, by age, with bootstrap samples.
  - input/VE_mab_CI_Hosp_MA.csv — Efficacy against hospitalisation, by age, with bootstrap samples.
  - input/VE_mab_CI_Seve_MA.csv — Efficacy against the severe composite, by age, with bootstrap samples.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_mab_hospital_sd — Monoclonal antibody efficacy, hospitalisation: standard deviation

- **Unit:** proportion
- **Definition:** The standard deviation of the sampled efficacy, derived from the confidence interval of the trial estimate.
- **Why it matters:** It decides how often the programme misses its expected effect, and therefore how often the cost-effectiveness result changes.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96), or from the bootstrap distribution in the efficacy file.
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** averted cases, averted costs, averted qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_mab_outpatient — Monoclonal antibody efficacy, outpatient (medically attended) infection

- **Unit:** proportion
- **Definition:** The efficacy of the product against medically attended RSV infection of any severity.
- **Why it matters:** Most cases are outpatient, so this value determines the number of cases the programme averts.
- **How to source it:** The pivotal trial estimate for the same outcome and age group, with its confidence interval.
- **Range:** 0 to 1
- **Evidence:** Trial estimate for the same country or a multi-country trial including it: `ok` with the citation. Estimate from another setting: `transfer`, with the population named.
- **Drives:** averted cases, averted costs, averted qalys
- **Known sources:**
  - input/VE_mab_CI_MA.csv — Efficacy against medically attended infection, by age, with bootstrap samples.
  - input/VE_mab_CI_Hosp_MA.csv — Efficacy against hospitalisation, by age, with bootstrap samples.
  - input/VE_mab_CI_Seve_MA.csv — Efficacy against the severe composite, by age, with bootstrap samples.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_mab_outpatient_sd — Monoclonal antibody efficacy, outpatient (medically attended) infection: standard deviation

- **Unit:** proportion
- **Definition:** The standard deviation of the sampled efficacy, derived from the confidence interval of the trial estimate.
- **Why it matters:** It decides how often the programme misses its expected effect, and therefore how often the cost-effectiveness result changes.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96), or from the bootstrap distribution in the efficacy file.
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** averted cases, averted costs, averted qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_mab_icu — Monoclonal antibody efficacy, intensive-care admission

- **Unit:** proportion
- **Definition:** The efficacy of the product against RSV intensive-care admission.
- **Why it matters:** ICU cases drive the mortality and the tail of the cost distribution.
- **How to source it:** The pivotal trial or its severe-outcome estimate for the same age group, with its confidence interval.
- **Range:** 0 to 1
- **Evidence:** Trial estimate for the same country or a multi-country trial including it: `ok` with the citation. Estimate from another setting: `transfer`, with the population named.
- **Drives:** averted cases, averted costs, averted qalys
- **Known sources:**
  - input/VE_mab_CI_MA.csv — Efficacy against medically attended infection, by age, with bootstrap samples.
  - input/VE_mab_CI_Hosp_MA.csv — Efficacy against hospitalisation, by age, with bootstrap samples.
  - input/VE_mab_CI_Seve_MA.csv — Efficacy against the severe composite, by age, with bootstrap samples.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_mab_icu_sd — Monoclonal antibody efficacy, intensive-care admission: standard deviation

- **Unit:** proportion
- **Definition:** The standard deviation of the sampled efficacy, derived from the confidence interval of the trial estimate.
- **Why it matters:** It decides how often the programme misses its expected effect, and therefore how often the cost-effectiveness result changes.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96), or from the bootstrap distribution in the efficacy file.
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** averted cases, averted costs, averted qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_mab_death — Monoclonal antibody efficacy, death

- **Unit:** proportion
- **Definition:** The efficacy of the product against RSV death. The reference implementation sets this equal to the intensive-care efficacy.
- **Why it matters:** Deaths are the outcome with the largest quality-of-life consequence, so this value matters even though it is not observed directly in most trials.
- **How to source it:** The trial estimate for the severe composite, or the intensive-care efficacy with the reasoning recorded.
- **Range:** 0 to 1
- **Evidence:** Trial estimate for the same country or a multi-country trial including it: `ok` with the citation. Estimate from another setting: `transfer`, with the population named.
- **Drives:** averted cases, averted costs, averted qalys
- **Known sources:**
  - input/VE_mab_CI_MA.csv — Efficacy against medically attended infection, by age, with bootstrap samples.
  - input/VE_mab_CI_Hosp_MA.csv — Efficacy against hospitalisation, by age, with bootstrap samples.
  - input/VE_mab_CI_Seve_MA.csv — Efficacy against the severe composite, by age, with bootstrap samples.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_mab_death_sd — Monoclonal antibody efficacy, death: standard deviation

- **Unit:** proportion
- **Definition:** The standard deviation of the sampled efficacy, derived from the confidence interval of the trial estimate.
- **Why it matters:** It decides how often the programme misses its expected effect, and therefore how often the cost-effectiveness result changes.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96), or from the bootstrap distribution in the efficacy file.
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** averted cases, averted costs, averted qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_mab_non_ma — Monoclonal antibody efficacy, medically unattended infection

- **Unit:** proportion
- **Definition:** The efficacy of the product against RSV infection that never reaches a clinician.
- **Why it matters:** These cases are frequent; an efficacy of zero here means the programme only prevents visible disease.
- **How to source it:** The trial estimate for the same outcome, or the all-cause efficacy with the reasoning recorded.
- **Range:** 0 to 1
- **Evidence:** Trial estimate for the same country or a multi-country trial including it: `ok` with the citation. Estimate from another setting: `transfer`, with the population named.
- **Drives:** averted cases, averted costs, averted qalys
- **Known sources:**
  - input/VE_mab_CI_MA.csv — Efficacy against medically attended infection, by age, with bootstrap samples.
  - input/VE_mab_CI_Hosp_MA.csv — Efficacy against hospitalisation, by age, with bootstrap samples.
  - input/VE_mab_CI_Seve_MA.csv — Efficacy against the severe composite, by age, with bootstrap samples.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### ve_mab_non_ma_sd — Monoclonal antibody efficacy, medically unattended infection: standard deviation

- **Unit:** proportion
- **Definition:** The standard deviation of the sampled efficacy, derived from the confidence interval of the trial estimate.
- **Why it matters:** It decides how often the programme misses its expected effect, and therefore how often the cost-effectiveness result changes.
- **How to source it:** Derive it from the published confidence interval as (upper − lower) / (2 × 1.96), or from the bootstrap distribution in the efficacy file.
- **Range:** 0 to 1
- **Evidence:** Derived from a published interval: `derived`, with the interval cited.
- **Drives:** averted cases, averted costs, averted qalys

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### duration_mab_years — Monoclonal antibody protection duration

- **Unit:** years
- **Definition:** How long the protection lasts after the dose is given. The engine applies the efficacy for that many months of age.
- **Why it matters:** A duration that is too long credits the programme with protection it does not give, and a duration that is too short understates it.
- **How to source it:** The trial follow-up that supports the duration, or the regulatory label. State whether the protection wanes over time; if it does, supply an age-specific efficacy profile instead.
- **Range:** 0 to 5
- **Evidence:** Trial follow-up: `ok` with the citation. Regulatory label: `ok`.
- **Drives:** averted cases, averted costs

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### coverage_mab — Monoclonal antibody coverage

- **Unit:** proportion
- **Definition:** The share of the eligible cohort that receives the product. Every scenario can override it; this value is the default.
- **Why it matters:** Coverage scales the whole benefit linearly, so it is the assumption an analyst is most often asked to defend.
- **How to source it:** The coverage assumed in the study protocol, the expected uptake in the first year, or the national immunisation coverage for a comparable programme.
- **Range:** 0 to 1
- **Evidence:** Planned programme coverage from the study protocol: `ok`. Observed coverage from another country: `transfer`.
- **Drives:** averted cases, averted costs, doses

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### price_mab_min — Monoclonal antibody price, low

- **Unit:** local currency per dose
- **Definition:** The lower bound of the dose price. When the two price bounds are equal, the price is deterministic.
- **Why it matters:** The price drives the intervention cost, which is the dominant term in the incremental cost, and it drives the price threshold P*.
- **How to source it:** The confidential price from the manufacturer, the price used in the study protocol, or a published reference price. Always record the price year and whether the price is confidential.
- **Range:** 0 to 100000
- **Evidence:** Study price: `ok` with the citation. List price: `ok`. Reference price from another country: `transfer`.
- **Drives:** intervention cost, icer, pstar

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### price_mab_max — Monoclonal antibody price, high

- **Unit:** local currency per dose
- **Definition:** The upper bound of the dose price, sampled uniformly between the low and the high bound.
- **Why it matters:** A price range widens the cost-effectiveness plane and shows how much the conclusion depends on the price.
- **How to source it:** The same source as the low bound; a scenario analysis range is a legitimate choice if it is documented as such.
- **Range:** 0 to 100000
- **Evidence:** Same status as the low bound.
- **Drives:** intervention cost, icer, pstar

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### admin_cost_mab — Monoclonal antibody administration cost

- **Unit:** local currency per dose
- **Definition:** The cost of delivering one dose: the health-care professional time and the consumables, per dose.
- **Why it matters:** It is added to the price of every dose, so it matters most for expensive products with small cohorts.
- **How to source it:** The programme budget or the costing study of the delivery pathway.
- **Range:** 0 to 100000
- **Evidence:** Programme budget: `ok`. Unit-costing study: `ok`.
- **Drives:** intervention cost

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### implementation_cost_mab — Monoclonal antibody implementation cost

- **Unit:** local currency
- **Definition:** The fixed cost of setting the programme up (training, information, cold chain), divided over the doses of the cohort.
- **Why it matters:** It is a fixed cost spread over one cohort, so it matters most for small cohorts and for expensive products.
- **How to source it:** The programme budget for the first year.
- **Range:** 0 to 1000000000
- **Evidence:** Programme budget: `ok`.
- **Drives:** intervention cost

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### wastage_mab — Monoclonal antibody wastage

- **Unit:** proportion
- **Definition:** The share of doses that are lost: multi-dose vials, missed appointments, wastage at the end of the campaign.
- **Why it matters:** Wastage raises the number of doses that have to be bought, and therefore the intervention cost.
- **How to source it:** The programme's own wastage rate, or the wastage rate of a comparable immunisation programme in the same health system.
- **Range:** 0 to 1
- **Evidence:** Programme data: `ok`. Comparable programme: `transfer`.
- **Drives:** intervention cost, doses

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

## 7. Analysis conventions

Perspective, discounting and the willingness-to-pay thresholds used for the decision analysis.

### discount_rate_cost — Cost discount rate

- **Unit:** per year
- **Definition:** The annual rate at which future costs are discounted.
- **Why it matters:** Discounting lowers the value of a prevention programme whose costs are now and whose benefits are later, or the other way round when the intervention is cheap and durable.
- **How to source it:** The national guideline on health-economic evaluation; the reference implementation uses 3 %.
- **Range:** 0 to 0.15
- **Evidence:** National guideline: `ok` with the citation. A convention: `convention`.
- **Drives:** costs, icer

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### discount_rate_effect — Effect discount rate

- **Unit:** per year
- **Definition:** The annual rate at which future health effects are discounted.
- **Why it matters:** It changes the weight of the deaths averted in the first months of life against the cases averted later.
- **How to source it:** The national guideline; the reference implementation uses 1.5 %.
- **Range:** 0 to 0.15
- **Evidence:** National guideline: `ok` with the citation. A convention: `convention`.
- **Drives:** qalys, icer

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### gdp_per_capita — GDP per capita

- **Unit:** local currency per capita
- **Definition:** The gross domestic product per capita of the country in the price year. It is only used to derive the willingness-to-pay thresholds.
- **Why it matters:** The thresholds decide at which price a strategy is considered good value for money, so the GDP choice shapes the headline conclusion.
- **How to source it:** National accounts (Eurostat, World Bank) for the price year, in the currency of the analysis.
- **Range:** 0 to 1000000
- **Evidence:** National accounts: `ok` with the citation. International estimate: `transfer`.
- **Drives:** inmb, pstar

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### gdp_fx_usd — Exchange rate, local currency per USD

- **Unit:** local currency per USD
- **Definition:** The exchange rate used to convert a dollar value into the currency of the analysis.
- **Why it matters:** Only needed when a source is reported in dollars; the engine itself works in the local currency.
- **How to source it:** The central bank or the international source that published the dollar figure.
- **Range:** 0 to 100000
- **Evidence:** Central bank: `ok`.

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### wtp_multiplier_0_5 — Willingness-to-pay threshold, 0.5 × GDP per capita

- **Unit:** multiple of GDP per capita
- **Definition:** The lower decision threshold: half of GDP per capita per QALY.
- **Why it matters:** It is a decision convention, not an estimate; the results are reported against all three thresholds so that the reader can judge the choice.
- **How to source it:** The study protocol or the national guideline that sets the threshold.
- **Range:** 0 to 10
- **Evidence:** Decision convention: `convention`, with the protocol cited.
- **Drives:** inmb, pstar

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### wtp_multiplier_1 — Willingness-to-pay threshold, 1 × GDP per capita

- **Unit:** multiple of GDP per capita
- **Definition:** The central decision threshold: one times GDP per capita per QALY.
- **Why it matters:** It is the threshold most often used for the headline conclusion.
- **How to source it:** The study protocol or the national guideline that sets the threshold.
- **Range:** 0 to 10
- **Evidence:** Decision convention: `convention`, with the protocol cited.
- **Drives:** inmb, pstar

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### wtp_multiplier_3 — Willingness-to-pay threshold, 3 × GDP per capita

- **Unit:** multiple of GDP per capita
- **Definition:** The upper decision threshold: three times GDP per capita per QALY.
- **Why it matters:** It shows which strategies become good value for money under a more generous threshold.
- **How to source it:** The study protocol or the national guideline that sets the threshold.
- **Range:** 0 to 10
- **Evidence:** Decision convention: `convention`, with the protocol cited.
- **Drives:** inmb, pstar

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### analysis_perspective — Analysis perspective

- **Unit:** 
- **Definition:** Whose costs and benefits are counted: the health-care payer, the health system, or society.
- **Why it matters:** A societal perspective adds productivity losses, which the current engine does not model; say which perspective the results are in.
- **How to source it:** The study protocol and the national guideline.
- **Evidence:** Decision convention: `convention`, with the protocol cited.
- **Drives:** costs

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

### currency — Currency

- **Unit:** 
- **Definition:** The ISO 4217 code of the currency in which every monetary value of the pack is expressed.
- **Why it matters:** All costs have to be in the same currency; the engine does not convert them.
- **How to source it:** The study protocol and the price year.
- **Evidence:** Fact: `ok`.
- **Drives:** costs, icer

| Value | Evidence status | Source | Reviewer note |
|---|---|---|---|
|  |  |  |  |

