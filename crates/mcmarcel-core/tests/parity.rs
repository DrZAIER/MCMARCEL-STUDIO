//! Parity against the R implementation.
//!
//! The R project in the parent directory is the validation oracle for this
//! rewrite. The random streams deliberately differ (R uses its own Mersenne
//! Twister, the engine uses a seeded ChaCha8 stream), so parity is established on
//! the deterministic shared machinery:
//!
//! * the time-age to cohort-age conversion;
//! * the protection matrix by birth cohort, including catch-up;
//! * the shared unit-price formula for both intervention products (defect C7);
//! * the gamma parameterisation of the cost and utility sampling;
//! * the age-based discounting vector;
//! * the net monetary benefit of a strategy.
//!
//! The fixture is regenerated with `Rscript parity/oracle_case_001.R`, which calls
//! the R implementation's own functions. The expected file records the R version
//! that produced it.

use std::path::PathBuf;

use mcmarcel_core::analysis::net_benefit;
use mcmarcel_core::engine::{discount_factor, protection_by_cohort, time_age_to_cohort_age, unit_price};
use mcmarcel_core::rng::gamma_parameters;
use mcmarcel_core::types::Matrix;
use serde_json::Value;

const TOLERANCE: f64 = 1e-9;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../parity/expected/case_001.json")
}

fn load_fixture() -> Value {
    let path = fixture_path();
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&raw).expect("parity fixture must be valid JSON")
}

fn numbers(value: &Value) -> Vec<Vec<f64>> {
    value
        .as_array()
        .expect("matrix must be an array of rows")
        .iter()
        .map(|row| {
            row.as_array()
                .expect("row must be an array")
                .iter()
                .map(|cell| cell.as_f64().expect("cell must be a number"))
                .collect()
        })
        .collect()
}

fn flat_numbers(value: &Value) -> Vec<f64> {
    value
        .as_array()
        .expect("vector must be an array")
        .iter()
        .map(|cell| cell.as_f64().expect("cell must be a number"))
        .collect()
}

fn assert_close(actual: f64, expected: f64, label: &str) {
    let difference = (actual - expected).abs();
    assert!(
        difference <= TOLERANCE * expected.abs().max(1.0),
        "{label}: rust {actual} differs from R {expected} (difference {difference})"
    );
}

fn matrix_from(rows: &[Vec<f64>]) -> Matrix {
    let cols = rows.first().map(|row| row.len()).unwrap_or(0);
    let data: Vec<f64> = rows.iter().flatten().copied().collect();
    Matrix::new(rows.len(), cols, data).expect("matrix shape")
}

#[test]
fn fixture_is_generated_by_the_r_oracle() {
    let fixture = load_fixture();
    assert_eq!(fixture["fixture"], "case_001");
    assert_eq!(fixture["generated_by"], "parity/oracle_case_001.R");
    let r_version = fixture["r_version"].as_str().expect("r_version");
    assert!(r_version.starts_with("R version"), "unexpected R version: {r_version}");
}

#[test]
fn cohort_conversion_matches_the_r_implementation() {
    let fixture = load_fixture();
    let time_age = matrix_from(&numbers(&fixture["time_age_matrix"]));
    let cohort = time_age_to_cohort_age(&time_age).expect("cohort conversion");
    let expected = numbers(&fixture["cohort_matrix"]);
    assert_eq!(cohort.rows, expected.len());
    assert_eq!(cohort.cols, expected[0].len());
    for row in 0..cohort.rows {
        for col in 0..cohort.cols {
            assert_close(
                cohort.get(row, col),
                expected[row][col],
                &format!("cohort[{row},{col}]"),
            );
        }
    }
}

#[test]
fn protection_by_cohort_matches_the_r_implementation() {
    let fixture = load_fixture();
    let efficacy = flat_numbers(&fixture["efficacy_vector"]);

    let year_round = protection_by_cohort(&efficacy, 1, 12, 0.95, 0.0);
    let expected_year_round = numbers(&fixture["protection_year_round"]);
    for row in 0..year_round.rows {
        for col in 0..year_round.cols {
            assert_close(
                year_round.get(row, col),
                expected_year_round[row][col],
                &format!("year-round protection[{row},{col}]"),
            );
        }
    }

    let seasonal = protection_by_cohort(&efficacy, 9, 3, 0.95, 0.4);
    let expected_seasonal = numbers(&fixture["protection_seasonal"]);
    for row in 0..seasonal.rows {
        for col in 0..seasonal.cols {
            assert_close(
                seasonal.get(row, col),
                expected_seasonal[row][col],
                &format!("seasonal protection[{row},{col}]"),
            );
        }
    }
}

#[test]
fn unit_price_matches_the_r_implementation() {
    let fixture = load_fixture();
    assert_close(
        unit_price(12.5, 95.0, 0.05, 250_000.0, 120_000.0),
        fixture["unit_price_maternal"].as_f64().unwrap(),
        "maternal unit price",
    );
    assert_close(
        unit_price(380.0, 2100.0, 0.02, 500_000.0, 95_000.0),
        fixture["unit_price_mab"].as_f64().unwrap(),
        "mAb unit price",
    );
}

#[test]
fn gamma_parameterisation_matches_the_r_implementation() {
    let fixture = load_fixture();
    let mean = fixture["gamma_mean"].as_f64().unwrap();
    let sd = fixture["gamma_sd"].as_f64().unwrap();
    let (shape, rate) = gamma_parameters(mean, sd);
    assert_close(shape, fixture["gamma_shape"].as_f64().unwrap(), "gamma shape");
    assert_close(rate, fixture["gamma_rate"].as_f64().unwrap(), "gamma rate");
}

#[test]
fn discounting_vector_matches_the_r_implementation() {
    let fixture = load_fixture();
    let expected_cost = flat_numbers(&fixture["discount_cost"]);
    let expected_effect = flat_numbers(&fixture["discount_effect"]);
    assert_eq!(expected_cost.len(), 60);
    for age in 0..expected_cost.len() {
        assert_close(
            discount_factor(age, 0.03),
            expected_cost[age],
            &format!("cost discount factor at age {age}"),
        );
        assert_close(
            discount_factor(age, 0.03),
            expected_effect[age],
            &format!("effect discount factor at age {age}"),
        );
    }
}

#[test]
fn net_benefit_matches_the_r_implementation() {
    let fixture = load_fixture();
    let expected = flat_numbers(&fixture["net_benefit"]);
    let qaly = [0.0, 25.5, 60.25, 120.0];
    let cost = [0.0, 1_200_000.0, 4_800_000.0, 13_500_000.0];
    for index in 0..expected.len() {
        assert_close(
            net_benefit(qaly[index], cost[index], 4188.0),
            expected[index],
            &format!("net benefit {index}"),
        );
    }
}

/// The R implementation extrapolates unit costs over age bands
/// (`approx_age_by_sim`); the rewrite carries one cost per category and does not
/// reproduce that interpolation. The fixture keeps the R values so the deviation
/// stays visible, and this test records it explicitly.
#[test]
fn age_banded_cost_interpolation_is_a_documented_deviation() {
    let fixture = load_fixture();
    let interpolated = numbers(&fixture["age_interpolated"]);
    assert_eq!(interpolated.len(), 60);
    assert_close(interpolated[0][0], 120.0, "R cost at age 0");
    assert_close(interpolated[10][0], 120.0, "R cost at age 10");
    assert_close(interpolated[11][0], 240.0, "R cost at age 11");
    assert_close(interpolated[59][0], 240.0, "R cost at age 59");
}
