use std::time::{SystemTime, UNIX_EPOCH};

use rayon::prelude::*;

use crate::analysis;
use crate::model::{build_inputs, EfficacySet, ModelInputs, UncertainValue};
use crate::rng::DrawRng;
use crate::types::{
    CountryPack, DrawMetrics, DrawPoint, Matrix, RunMode, RunResult, RunSpec, Scenario,
    ScenarioSummary, AGE_MONTHS, CALENDAR_MONTHS,
};
use crate::validate::validate_pack;
use crate::{Error, Result};

pub fn run_pack(pack: &CountryPack, spec: &RunSpec) -> Result<RunResult> {
    if spec.draws == 0 {
        return Err(Error::Validation("draws must be at least 1".into()));
    }
    if spec.draws > 100_000 {
        return Err(Error::Validation("draws above 100,000 are not supported in the local engine".into()));
    }

    let validation = validate_pack(pack, spec.mode);
    if spec.mode == RunMode::Production && !validation.production_ready {
        let blockers: Vec<String> = validation
            .issues
            .iter()
            .filter(|issue| issue.blocks_production)
            .map(|issue| format!("{}: {}", issue.code, issue.message))
            .collect();
        return Err(Error::ProductionGate(blockers.join("; ")));
    }

    let inputs = build_inputs(pack)?;
    let outputs: Vec<DrawOutput> = (0..spec.draws)
        .into_par_iter()
        .map(|draw| run_draw(pack, &inputs, spec, draw))
        .collect::<Result<Vec<_>>>()?;

    let mut metrics = Vec::with_capacity(outputs.len() * pack.scenarios.len());
    let mut points = Vec::with_capacity(outputs.len() * pack.scenarios.len());
    for output in outputs {
        metrics.extend(output.metrics);
        points.extend(output.points);
    }

    let scenarios = summarize(pack, &metrics);
    let analysis = analysis::analyze(pack, &scenarios);
    let warnings = validation
        .issues
        .iter()
        .map(|issue| format!("[{}] {}", issue.code, issue.message))
        .collect();

    Ok(RunResult {
        pack_id: pack.country.id.clone(),
        pack_name: pack.country.name.clone(),
        pack_hash: validation.pack_hash.clone(),
        mode: spec.mode,
        draws: spec.draws,
        seed: spec.seed,
        created_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        fixture_stamp: validation.fixture_stamp.clone(),
        scenarios,
        points,
        analysis,
        validation,
        warnings,
    })
}

struct DrawOutput {
    metrics: Vec<DrawMetrics>,
    points: Vec<DrawPoint>,
}

#[derive(Debug, Clone)]
struct CostSamples {
    outpatient: f64,
    hospital: f64,
    icu: f64,
    non_ma: f64,
}

#[derive(Debug, Clone)]
struct QalySamples {
    outpatient: f64,
    hospital: f64,
    non_ma: f64,
}

#[derive(Debug, Clone)]
struct SampledEfficacy {
    hospital: f64,
    outpatient: f64,
    icu: f64,
    death: f64,
    non_ma: f64,
}

#[derive(Debug, Clone)]
struct Baseline {
    hospital: Matrix,
    icu: Matrix,
    outpatient: Matrix,
    non_ma: Matrix,
    deaths: Matrix,
    age_hospital: Vec<f64>,
    age_icu: Vec<f64>,
    age_outpatient: Vec<f64>,
    age_non_ma: Vec<f64>,
    age_deaths: Vec<f64>,
}

#[derive(Debug, Clone, Default)]
struct Averted {
    age_hospital: Vec<f64>,
    age_icu: Vec<f64>,
    age_outpatient: Vec<f64>,
    age_non_ma: Vec<f64>,
    age_deaths: Vec<f64>,
}

fn run_draw(pack: &CountryPack, inputs: &ModelInputs, spec: &RunSpec, draw: u32) -> Result<DrawOutput> {
    let mut rng = DrawRng::new(spec.seed, draw);

    // Draw efficacy first so both scenarios in a draw share the same protection.
    let maternal_efficacy = sample_efficacy(&inputs.maternal.efficacy, &mut rng);
    let mab_efficacy = sample_efficacy(&inputs.mab.efficacy, &mut rng);

    let baseline = build_baseline(inputs, &mut rng, draw)?;

    let costs = CostSamples {
        outpatient: rng.gamma(inputs.costs.outpatient.mean, inputs.costs.outpatient.sd),
        hospital: rng.gamma(inputs.costs.hospital.mean, inputs.costs.hospital.sd),
        icu: rng.gamma(inputs.costs.icu.mean, inputs.costs.icu.sd),
        non_ma: rng.gamma(inputs.costs.non_ma.mean, inputs.costs.non_ma.sd),
    };
    let qaly = QalySamples {
        outpatient: rng.gamma(inputs.qaly.outpatient.mean, inputs.qaly.outpatient.sd),
        hospital: rng.gamma(inputs.qaly.hospital.mean, inputs.qaly.hospital.sd),
        non_ma: rng.gamma(inputs.qaly.non_ma.mean, inputs.qaly.non_ma.sd),
    };

    let reference_medical_cost = medical_cost(&baseline.age_hospital, &baseline.age_icu, &baseline.age_outpatient, &baseline.age_non_ma, &costs, inputs);
    let reference_qaly = qaly_burden(&baseline.age_hospital, &baseline.age_icu, &baseline.age_outpatient, &baseline.age_non_ma, &baseline.age_deaths, &qaly, inputs);
    let reference_cases = sum5(&baseline.age_hospital, &baseline.age_icu, &baseline.age_outpatient, &baseline.age_non_ma);
    let reference_hospital = sum_vec(&baseline.age_hospital);
    let reference_deaths = sum_vec(&baseline.age_deaths);

    let mut metrics = Vec::with_capacity(pack.scenarios.len());
    let mut points = Vec::with_capacity(pack.scenarios.len());

    for scenario in &pack.scenarios {
        let maternal_protection = protection_matrices(
            &maternal_efficacy,
            inputs.maternal.duration_months,
            scenario,
            scenario.coverage_maternal * (1.0 - inputs.pre_term_rate),
            inputs,
        );
        let mab_protection = protection_matrices(
            &mab_efficacy,
            inputs.mab.duration_months,
            scenario,
            scenario.coverage_mab,
            inputs,
        );

        let averted = averted_burden(&baseline, &maternal_protection, &mab_protection);
        let averted_medical_cost = medical_cost(&averted.age_hospital, &averted.age_icu, &averted.age_outpatient, &averted.age_non_ma, &costs, inputs);
        let averted_qaly = qaly_burden(&averted.age_hospital, &averted.age_icu, &averted.age_outpatient, &averted.age_non_ma, &averted.age_deaths, &qaly, inputs);

        let maternal_price = scenario
            .price_maternal
            .unwrap_or_else(|| rng.uniform(inputs.maternal.price.low, inputs.maternal.price.high));
        let mab_price = scenario
            .price_mab
            .unwrap_or_else(|| rng.uniform(inputs.mab.price.low, inputs.mab.price.high));

        let maternal_doses = dose_count(
            scenario,
            scenario.coverage_maternal * (1.0 - inputs.pre_term_rate),
            inputs.births * (1.0 + inputs.stillbirth_rate),
            inputs,
        );
        let mab_doses = dose_count(scenario, scenario.coverage_mab, inputs.target_population, inputs);

        let maternal_unit = unit_price(
            inputs.maternal.administration_cost,
            maternal_price,
            inputs.maternal.wastage,
            inputs.maternal.implementation_cost,
            maternal_doses,
        );
        let mab_unit = unit_price(
            inputs.mab.administration_cost,
            mab_price,
            inputs.mab.wastage,
            inputs.mab.implementation_cost,
            mab_doses,
        );
        let maternal_cost = maternal_doses * maternal_unit;
        let mab_cost = mab_doses * mab_unit;
        let intervention_cost = maternal_cost + mab_cost;

        let total_doses = maternal_doses + mab_doses;
        let mean_price = if total_doses > 0.0 {
            (maternal_doses * maternal_price + mab_doses * mab_price) / total_doses
        } else {
            0.0
        };
        let mean_wastage = if total_doses > 0.0 {
            (maternal_doses * inputs.maternal.wastage + mab_doses * inputs.mab.wastage) / total_doses
        } else {
            0.0
        };

        let averted_cases = sum5(&averted.age_hospital, &averted.age_icu, &averted.age_outpatient, &averted.age_non_ma);
        let averted_hospital = sum_vec(&averted.age_hospital);
        let averted_deaths = sum_vec(&averted.age_deaths);
        let incremental_cost = intervention_cost - averted_medical_cost;
        let program_cases = reference_cases - averted_cases;
        let program_hospital = reference_hospital - averted_hospital;
        let program_deaths = reference_deaths - averted_deaths;
        let program_medical_cost = reference_medical_cost - averted_medical_cost;
        let program_qaly = reference_qaly - averted_qaly;
        let program_effect = reference_qaly - averted_qaly;
        let fvp_maternal = if scenario.coverage_maternal > 0.0 {
            inputs.births * scenario.coverage_maternal * proportion_target_population(scenario)
        } else {
            0.0
        };
        let fvp_mab = if scenario.coverage_mab > 0.0 {
            inputs.target_population * scenario.coverage_mab * proportion_target_population(scenario)
        } else {
            0.0
        };

        metrics.push(DrawMetrics {
            draw,
            scenario_id: scenario.id.clone(),
            scenario_name: scenario.name.clone(),
            reference_cases,
            program_cases,
            averted_cases,
            reference_hospital,
            program_hospital,
            averted_hospital,
            reference_deaths,
            program_deaths,
            averted_deaths,
            reference_cost: reference_medical_cost,
            program_cost: program_medical_cost,
            medical_cost_averted: averted_medical_cost,
            intervention_cost,
            incremental_cost,
            reference_qaly,
            program_qaly,
            qaly_averted: averted_qaly,
            fvp: fvp_maternal + fvp_mab,
            doses: total_doses,
            mean_price,
            wastage: mean_wastage,
            intervention_months: scenario.intervention_months() as f64,
            coverage: scenario.coverage_maternal.max(scenario.coverage_mab),
        });

        points.push(DrawPoint {
            draw,
            scenario_id: scenario.id.clone(),
            reference_cost: reference_medical_cost,
            reference_effect: reference_qaly,
            program_cost: program_medical_cost + intervention_cost,
            program_effect,
            incremental_cost,
            incremental_effect: averted_qaly,
        });
    }

    Ok(DrawOutput { metrics, points })
}

fn build_baseline(inputs: &ModelInputs, rng: &mut DrawRng, draw: u32) -> Result<Baseline> {
    let outpatient_col = if (draw as usize) < inputs.outpatient_rate.cols {
        draw as usize
    } else {
        rng.index(inputs.outpatient_rate.cols)
    };
    let non_ma_row = if (draw as usize) < inputs.non_ma.rows {
        draw as usize
    } else {
        rng.index(inputs.non_ma.rows)
    };

    let hospital_cohort = time_age_to_cohort_age(&inputs.hospital)?;
    let icu_cohort = time_age_to_cohort_age(&inputs.icu)?;

    let mut outpatient_time = Matrix::filled(AGE_MONTHS, CALENDAR_MONTHS, 0.0)?;
    let mut non_ma_time = Matrix::filled(AGE_MONTHS, CALENDAR_MONTHS, 0.0)?;
    for age in 0..AGE_MONTHS {
        let annual_outpatient = inputs.outpatient_rate.get(age, outpatient_col) / 1000.0 * inputs.target_population;
        let row_total: f64 = (0..CALENDAR_MONTHS).map(|m| inputs.hospital.get(age, m)).sum();
        for month in 0..CALENDAR_MONTHS {
            let hospital_weight = if row_total > 0.0 {
                inputs.hospital.get(age, month) / row_total
            } else {
                1.0 / CALENDAR_MONTHS as f64
            };
            let out = annual_outpatient * hospital_weight;
            outpatient_time.set(age, month, out);
            let ma = inputs.hospital.get(age, month) + out;
            non_ma_time.set(age, month, ma * inputs.non_ma.get(non_ma_row, age).clamp(0.0, 1.0));
        }
    }

    let outpatient_cohort = time_age_to_cohort_age(&outpatient_time)?;
    let non_ma_cohort = time_age_to_cohort_age(&non_ma_time)?;

    let mut deaths_time = Matrix::filled(AGE_MONTHS, CALENDAR_MONTHS, 0.0)?;
    for age in 0..AGE_MONTHS {
        for month in 0..CALENDAR_MONTHS {
            let deaths = inputs.hospital.get(age, month) * inputs.hcfr.get(age, 0)
                + inputs.icu.get(age, month) * inputs.hcfr.get(age, 1);
            deaths_time.set(age, month, deaths.max(0.0));
        }
    }
    let deaths_cohort = time_age_to_cohort_age(&deaths_time)?;

    let age_hospital = hospital_cohort.row_sums();
    let age_icu = icu_cohort.row_sums();
    let age_outpatient = outpatient_cohort.row_sums();
    let age_non_ma = non_ma_cohort.row_sums();
    let age_deaths = deaths_cohort.row_sums();

    Ok(Baseline {
        hospital: hospital_cohort,
        icu: icu_cohort,
        outpatient: outpatient_cohort,
        non_ma: non_ma_cohort,
        deaths: deaths_cohort,
        age_hospital,
        age_icu,
        age_outpatient,
        age_non_ma,
        age_deaths,
    })
}

struct ProtectionMatrices {
    hospital: Matrix,
    outpatient: Matrix,
    non_ma: Matrix,
    icu: Matrix,
    death: Matrix,
}

fn protection_matrices(
    efficacy: &SampledEfficacy,
    duration_months: usize,
    scenario: &Scenario,
    coverage: f64,
    inputs: &ModelInputs,
) -> ProtectionMatrices {
    let _ = inputs;
    ProtectionMatrices {
        hospital: protection_by_cohort(&efficacy_vector(efficacy.hospital, duration_months), scenario.intervention_start, scenario.intervention_end, coverage, scenario.coverage_catch_up),
        outpatient: protection_by_cohort(&efficacy_vector(efficacy.outpatient, duration_months), scenario.intervention_start, scenario.intervention_end, coverage, scenario.coverage_catch_up),
        non_ma: protection_by_cohort(&efficacy_vector(efficacy.non_ma, duration_months), scenario.intervention_start, scenario.intervention_end, coverage, scenario.coverage_catch_up),
        icu: protection_by_cohort(&efficacy_vector(efficacy.icu, duration_months), scenario.intervention_start, scenario.intervention_end, coverage, scenario.coverage_catch_up),
        death: protection_by_cohort(&efficacy_vector(efficacy.death, duration_months), scenario.intervention_start, scenario.intervention_end, coverage, scenario.coverage_catch_up),
    }
}

fn averted_burden(baseline: &Baseline, maternal: &ProtectionMatrices, mab: &ProtectionMatrices) -> Averted {
    Averted {
        age_hospital: averted_by_age(&baseline.hospital, &maternal.hospital, &mab.hospital),
        age_icu: averted_by_age(&baseline.icu, &maternal.icu, &mab.icu),
        age_outpatient: averted_by_age(&baseline.outpatient, &maternal.outpatient, &mab.outpatient),
        age_non_ma: averted_by_age(&baseline.non_ma, &maternal.non_ma, &mab.non_ma),
        age_deaths: averted_by_age(&baseline.deaths, &maternal.death, &mab.death),
    }
}

fn averted_by_age(baseline: &Matrix, maternal: &Matrix, mab: &Matrix) -> Vec<f64> {
    (0..AGE_MONTHS)
        .map(|age| {
            (0..CALENDAR_MONTHS)
                .map(|cohort| {
                    let protection = (maternal.get(age, cohort) + mab.get(age, cohort)).clamp(0.0, 1.0);
                    baseline.get(age, cohort) * protection
                })
                .sum()
        })
        .collect()
}

pub fn time_age_to_cohort_age(time_age: &Matrix) -> Result<Matrix> {
    time_age.validate_shape(AGE_MONTHS, CALENDAR_MONTHS)?;
    let mut cohort = Matrix::filled(AGE_MONTHS, CALENDAR_MONTHS, 0.0)?;
    for block in 0..5 {
        for age_in_year in 0..CALENDAR_MONTHS {
            let age = block * CALENDAR_MONTHS + age_in_year;
            for birth_month in 0..CALENDAR_MONTHS {
                let calendar_month = (birth_month + age_in_year) % CALENDAR_MONTHS;
                cohort.set(age, birth_month, time_age.get(age, calendar_month));
            }
        }
    }
    Ok(cohort)
}

/// Protection matrix by birth cohort, mirroring `get_protection_by_cohort` in the
/// R implementation (`functions/RSV_get_incidence.R`).
///
/// * Cohorts born inside the programme window are protected from birth, scaled by
///   `coverage`.
/// * Cohorts born outside the window are protected from the first month at which
///   the programme would have reached them, scaled by `coverage_catch_up`.
pub fn protection_by_cohort(
    efficacy_by_age: &[f64],
    intervention_start: u8,
    intervention_end: u8,
    coverage: f64,
    coverage_catch_up: f64,
) -> Matrix {
    let mut protection = Matrix::filled(efficacy_by_age.len(), CALENDAR_MONTHS, 0.0).expect("valid protection matrix");
    let mut intervention_months = [false; 12];
    let start = intervention_start.clamp(1, 12) as usize;
    let end = intervention_end.clamp(1, 12) as usize;
    if start <= end {
        for month in start..=end {
            intervention_months[month - 1] = true;
        }
    } else {
        for month in start..=12 {
            intervention_months[month - 1] = true;
        }
        for month in 1..=end {
            intervention_months[month - 1] = true;
        }
    }

    for cohort in 0..CALENDAR_MONTHS {
        if intervention_months[cohort] {
            for age in 0..efficacy_by_age.len() {
                protection.set(age, cohort, efficacy_by_age[age] * coverage);
            }
        } else if coverage_catch_up > 0.0 {
            let shift = (start + 12 - (cohort + 1)) % 12;
            for age in shift..efficacy_by_age.len().min(AGE_MONTHS) {
                protection.set(age, cohort, efficacy_by_age[age - shift] * coverage_catch_up);
            }
        }
    }
    protection
}

fn dose_count(scenario: &Scenario, coverage: f64, population: f64, inputs: &ModelInputs) -> f64 {
    let _ = inputs;
    if coverage <= 0.0 {
        return 0.0;
    }
    let mut dose_vector = vec![0.0; AGE_MONTHS];
    dose_vector[0] = 1.0;
    let matrix = protection_by_cohort(&dose_vector, scenario.intervention_start, scenario.intervention_end, coverage, scenario.coverage_catch_up);
    matrix.total() * population / 12.0
}

fn proportion_target_population(scenario: &Scenario) -> f64 {
    if scenario.coverage_catch_up > 0.0 {
        1.0
    } else {
        scenario.intervention_months() as f64 / 12.0
    }
}

/// Unit price of one delivered dose, mirroring `compute_unit_price` in the R
/// implementation (defect C7): both intervention products use the same formula.
///
/// `admin + price × (1 + wastage) + implementation / doses`
pub fn unit_price(admin: f64, price: f64, wastage: f64, implementation: f64, doses: f64) -> f64 {
    let mut unit = admin + price * (1.0 + wastage);
    if doses > 0.0 {
        unit += implementation / doses;
    }
    unit
}

fn medical_cost(hospital: &[f64], icu: &[f64], outpatient: &[f64], non_ma: &[f64], costs: &CostSamples, inputs: &ModelInputs) -> f64 {
    let mut total = 0.0;
    for age in 0..AGE_MONTHS {
        let discount = discount_factor(age, inputs.discount_cost);
        total += (hospital[age] * costs.hospital
            + icu[age] * costs.icu
            + outpatient[age] * costs.outpatient
            + non_ma[age] * costs.non_ma)
            * discount;
    }
    total
}

fn qaly_burden(hospital: &[f64], icu: &[f64], outpatient: &[f64], non_ma: &[f64], deaths: &[f64], qaly: &QalySamples, inputs: &ModelInputs) -> f64 {
    let mut total = 0.0;
    for age in 0..AGE_MONTHS {
        let discount = discount_factor(age, inputs.discount_effect);
        let yld = (hospital[age] + icu[age]) * qaly.hospital
            + outpatient[age] * qaly.outpatient
            + non_ma[age] * qaly.non_ma;
        let yll = deaths[age] * inputs.life.hrqale_disc[age];
        total += (yld * discount) + (yll * discount);
    }
    total
}

/// Discount factor for an age month, mirroring the discounting vector built in
/// `get_burden` (`functions/RSV_get_burden.R`): the factor changes per year of
/// age, not per month.
pub fn discount_factor(age_month: usize, rate: f64) -> f64 {
    let years = (age_month / 12) as i32;
    1.0 / (1.0 + rate).powi(years)
}

fn sample_efficacy(set: &EfficacySet, rng: &mut DrawRng) -> SampledEfficacy {
    let hospital = rng.beta(set.hospital.mean, set.hospital.sd);
    let outpatient = if same(set.outpatient, set.hospital) {
        hospital
    } else {
        rng.beta(set.outpatient.mean, set.outpatient.sd)
    };
    let non_ma = if same(set.non_ma, set.outpatient) {
        outpatient
    } else {
        rng.beta(set.non_ma.mean, set.non_ma.sd)
    };
    let icu = if same(set.icu, set.hospital) {
        hospital
    } else {
        rng.beta(set.icu.mean, set.icu.sd)
    };
    let death = if same(set.death, set.icu) {
        icu
    } else {
        rng.beta(set.death.mean, set.death.sd)
    };
    SampledEfficacy { hospital, outpatient, icu, death, non_ma }
}

fn same(a: UncertainValue, b: UncertainValue) -> bool {
    (a.mean - b.mean).abs() < 1e-12 && (a.sd - b.sd).abs() < 1e-12
}

fn efficacy_vector(value: f64, duration_months: usize) -> Vec<f64> {
    let mut vector = vec![0.0; AGE_MONTHS];
    for item in vector.iter_mut().take(duration_months.min(AGE_MONTHS)) {
        *item = value;
    }
    vector
}

fn summarize(pack: &CountryPack, metrics: &[DrawMetrics]) -> Vec<ScenarioSummary> {
    pack.scenarios
        .iter()
        .map(|scenario| {
            let rows: Vec<&DrawMetrics> = metrics.iter().filter(|m| m.scenario_id == scenario.id).collect();
            let n = rows.len().max(1) as f64;
            ScenarioSummary {
                scenario_id: scenario.id.clone(),
                scenario_name: scenario.name.clone(),
                is_reference: scenario.is_reference,
                coverage_maternal: scenario.coverage_maternal,
                coverage_mab: scenario.coverage_mab,
                mean_reference_cases: mean(&rows, |m| m.reference_cases, n),
                mean_program_cases: mean(&rows, |m| m.program_cases, n),
                mean_averted_cases: mean(&rows, |m| m.averted_cases, n),
                mean_averted_hospital: mean(&rows, |m| m.averted_hospital, n),
                mean_averted_deaths: mean(&rows, |m| m.averted_deaths, n),
                mean_reference_cost: mean(&rows, |m| m.reference_cost, n),
                mean_program_cost: mean(&rows, |m| m.program_cost, n),
                mean_medical_cost_averted: mean(&rows, |m| m.medical_cost_averted, n),
                mean_intervention_cost: mean(&rows, |m| m.intervention_cost, n),
                mean_incremental_cost: mean(&rows, |m| m.incremental_cost, n),
                mean_reference_qaly: mean(&rows, |m| m.reference_qaly, n),
                mean_program_qaly: mean(&rows, |m| m.program_qaly, n),
                mean_qaly_averted: mean(&rows, |m| m.qaly_averted, n),
                mean_fvp: mean(&rows, |m| m.fvp, n),
                mean_doses: mean(&rows, |m| m.doses, n),
                mean_price: mean(&rows, |m| m.mean_price, n),
                mean_wastage: mean(&rows, |m| m.wastage, n),
                intervention_months: scenario.intervention_months() as f64,
            }
        })
        .collect()
}

fn mean(rows: &[&DrawMetrics], accessor: impl Fn(&DrawMetrics) -> f64, n: f64) -> f64 {
    rows.iter().map(|m| accessor(m)).sum::<f64>() / n
}

fn sum_vec(values: &[f64]) -> f64 {
    values.iter().sum()
}

fn sum5(a: &[f64], b: &[f64], c: &[f64], d: &[f64]) -> f64 {
    a.iter().chain(b).chain(c).chain(d).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::demo_pack;

    #[test]
    fn cohort_conversion_moves_calendar_months() {
        let mut matrix = Matrix::filled(AGE_MONTHS, CALENDAR_MONTHS, 0.0).unwrap();
        matrix.set(0, 2, 7.0);
        let cohort = time_age_to_cohort_age(&matrix).unwrap();
        assert_eq!(cohort.get(0, 2), 7.0);
        assert_eq!(cohort.get(1, 1), 0.0);
    }

    #[test]
    fn year_round_protection_covers_every_cohort() {
        let efficacy = vec![0.5; AGE_MONTHS];
        let protection = protection_by_cohort(&efficacy, 1, 12, 1.0, 0.0);
        for cohort in 0..12 {
            assert_eq!(protection.get(0, cohort), 0.5);
        }
    }

    #[test]
    fn demo_run_produces_finite_results() {
        let pack = demo_pack();
        let spec = RunSpec { draws: 8, seed: 42, mode: RunMode::Fixture };
        let result = run_pack(&pack, &spec).unwrap();
        assert_eq!(result.scenarios.len(), 3);
        for scenario in &result.scenarios {
            assert!(scenario.mean_qaly_averted.is_finite());
            assert!(scenario.mean_incremental_cost.is_finite());
        }
        let maternal = result.scenarios.iter().find(|s| s.scenario_id == "maternal").unwrap();
        assert!(maternal.mean_averted_cases > 0.0);
        assert!(maternal.mean_qaly_averted > 0.0);
    }
}
