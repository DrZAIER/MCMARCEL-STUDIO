//! Regression test against the upstream Belgian reference case.
//!
//! The Belgian base case is the reference of the McMarcel framework: the R
//! implementation reproduces it byte-identically (see
//! `tests/test_regression_belgium.R`), and the committed upstream fixture holds
//! the expected numbers.
//!
//! This test runs the Rust engine on the Belgian reference pack and compares the
//! decision-relevant outputs with the upstream fixture. The tolerances are the
//! ones recorded in `parity/expected/belgium_reference.json`; they cover the
//! difference between the two implementations (independent random streams, the
//! corrected fractional-year discounting of life expectancy) and the Monte Carlo
//! error of the 20 upstream simulations.
//!
//! Regenerate the fixture with:
//!   Rscript parity/extract_belgium_reference.R

use std::path::PathBuf;

use mcmarcel_core::engine::run_pack;
use mcmarcel_core::types::{CountryPack, RunMode, RunSpec};
use serde_json::Value;

fn rs_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_json(path: PathBuf) -> Value {
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&raw).expect("valid JSON")
}

fn relative(actual: f64, expected: f64, tolerance: f64, label: &str) {
    if expected.abs() < 1e-9 {
        assert!(actual.abs() < 1e-6, "{label}: rust {actual} vs R {expected}");
        return;
    }
    let difference = (actual - expected).abs() / expected.abs();
    assert!(
        difference <= tolerance,
        "{label}: rust {actual} differs from the R reference {expected} by {:.1} % (tolerance {:.0} %)",
        difference * 100.0,
        tolerance * 100.0
    );
}

#[test]
fn belgian_reference_case_reproduces_the_upstream_fixture() {
    let pack_path = rs_root().join("packs/belgium-belx.json");
    if !pack_path.exists() {
        eprintln!("skipping: {} has not been built", pack_path.display());
        return;
    }
    let pack = CountryPack::from_json_file(&pack_path).expect("Belgian reference pack");
    let reference = load_json(rs_root().join("parity/expected/belgium_reference.json"));
    let tolerance = &reference["tolerance"];

    // 20 draws, like the 20 upstream simulations.
    let spec = RunSpec { draws: 20, seed: 20_260_120, mode: RunMode::Fixture };
    let result = run_pack(&pack, &spec).expect("Belgian reference run");

    assert!(
        result.fixture_stamp.is_some(),
        "the reference pack carries perturbed public-release inputs and must be stamped"
    );

    let scenarios = reference["scenarios"].as_array().expect("reference scenarios");
    assert_eq!(scenarios.len(), 10, "the upstream fixture has ten intervention scenarios");

    for (index, expected) in scenarios.iter().enumerate() {
        let scenario_id = expected["scenario_id"].as_u64().expect("scenario id");
        let label = format!("scenario {scenario_id} ({})", expected["intervention"].as_str().unwrap_or("?"));
        // The pack keeps the upstream scenario order after the reference arm.
        let actual = result
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario_id.starts_with(&format!("bel-{scenario_id:02}-")))
            .unwrap_or_else(|| panic!("{label}: missing in the Rust run"));

        relative(
            actual.mean_averted_cases,
            expected["total_cases_disc_averted"].as_f64().unwrap(),
            tolerance["cases"].as_f64().unwrap(),
            &format!("{label}: cases averted"),
        );
        relative(
            actual.mean_averted_hospital,
            expected["hosp_cases_disc_averted"].as_f64().unwrap(),
            tolerance["hospital"].as_f64().unwrap(),
            &format!("{label}: hospital admissions averted"),
        );
        relative(
            actual.mean_averted_deaths,
            expected["rsv_deaths_disc_averted"].as_f64().unwrap(),
            tolerance["deaths"].as_f64().unwrap(),
            &format!("{label}: deaths averted"),
        );
        relative(
            actual.mean_qaly_averted,
            expected["total_QALY_disc_averted"].as_f64().unwrap(),
            tolerance["qaly"].as_f64().unwrap(),
            &format!("{label}: QALYs averted"),
        );
        relative(
            actual.mean_incremental_cost,
            expected["incremental_cost_disc_program"].as_f64().unwrap(),
            tolerance["incremental_cost"].as_f64().unwrap(),
            &format!("{label}: incremental cost"),
        );

        // The intervention cost is deterministic: the number of doses times the
        // price. It must match the upstream fixture exactly.
        let upstream_cost = expected["intervention_cost_program"].as_f64().unwrap();
        let difference = (actual.mean_intervention_cost - upstream_cost).abs() / upstream_cost.abs();
        assert!(
            difference < 0.001,
            "{label}: intervention cost {} differs from the R reference {upstream_cost} by {:.2} %",
            actual.mean_intervention_cost,
            difference * 100.0
        );

        let _ = index;
    }
}

#[test]
fn belgian_reference_pack_is_not_production_ready() {
    let pack_path = rs_root().join("packs/belgium-belx.json");
    if !pack_path.exists() {
        return;
    }
    let pack = CountryPack::from_json_file(&pack_path).expect("Belgian reference pack");
    let report = mcmarcel_core::validate::validate_pack(&pack, RunMode::Production);
    assert!(
        !report.production_ready,
        "the perturbed public-release inputs must block production mode"
    );
    assert!(report.fixture_stamp.is_some());
}
