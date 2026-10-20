//! The Gavi country catalogue and the packs generated from it.
//!
//! The packs are the *derivable* part of a country study: the life table, the
//! life expectancy, the births and the GDP per capita. Everything that needs
//! country evidence stays MISSING, so the production gate keeps the packs
//! closed until an analyst fills the questionnaire.

use std::path::PathBuf;

use mcmarcel_core::types::{CountryPack, RunMode};
use mcmarcel_core::validate::validate_pack;

fn rs_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn catalogue() -> Vec<(String, String, String)> {
    let path = rs_root().join("countries/gavi-2020.csv");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let mut rows = Vec::new();
    for line in raw.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        // The file is quoted CSV: split on the separator outside the quotes.
        let mut fields = Vec::new();
        let mut current = String::new();
        let mut quoted = false;
        for character in line.chars() {
            match character {
                '"' => quoted = !quoted,
                ',' if !quoted => {
                    fields.push(current.clone());
                    current.clear();
                }
                other => current.push(other),
            }
        }
        fields.push(current);
        let field = |index: usize| {
            fields
                .get(index)
                .map(|value| value.trim().to_string())
                .unwrap_or_default()
        };
        rows.push((field(0), field(1), field(3)));
    }
    rows
}

#[test]
fn the_catalogue_lists_the_gavi_countries() {
    let rows = catalogue();
    assert_eq!(rows.len(), 58, "the 2020 Gavi eligibility list has 58 countries");
    let iso3: Vec<&str> = rows.iter().map(|row| row.0.as_str()).collect();
    for expected in ["AFG", "BEN", "GHA", "IND", "NGA", "TZA", "ZMB"] {
        assert!(iso3.contains(&expected), "{expected} is missing from the catalogue");
    }
    // Every country has a currency and a name.
    for (iso3, name, currency) in &rows {
        assert_eq!(iso3.len(), 3, "{name}: the ISO3 code is malformed");
        assert!(!name.is_empty());
        assert!(!currency.is_empty(), "{name}: no currency resolved");
    }
}

#[test]
fn every_catalogue_country_has_a_pack_that_is_not_production_ready() {
    let root = rs_root().join("packs/gavi");
    for (iso3, name, _) in catalogue() {
        let path = root.join(format!("{}.json", iso3.to_lowercase()));
        assert!(path.exists(), "{name}: no pack at {}", path.display());
        let pack = CountryPack::from_json_file(&path).expect("pack");
        assert_eq!(pack.country.iso3, iso3);
        assert_eq!(pack.country.id, format!("gavi-{iso3}").to_lowercase());

        // The derivable inputs are filled in.
        let births = pack.parameters.get("births").expect("births");
        assert!(births.as_f64().unwrap_or(0.0) > 0.0, "{name}: no births");
        assert_eq!(births.status, mcmarcel_core::types::EvidenceStatus::Derived);
        let life = pack.parameters.get("life_expectancy_years").expect("life expectancy");
        assert!(life.as_f64().unwrap_or(0.0) > 0.0, "{name}: no life expectancy");
        let gdp = pack.parameters.get("gdp_per_capita").expect("gdp");
        // The World Bank has no figure for a few of these countries; the pack then
        // records the gap instead of inventing a number.
        match gdp.as_f64() {
            Some(value) => {
                assert!(value > 0.0, "{name}: the GDP per capita is not positive");
                assert_eq!(gdp.status, mcmarcel_core::types::EvidenceStatus::Ok);
            }
            None => assert_eq!(gdp.status, mcmarcel_core::types::EvidenceStatus::Missing),
        }

        // The life table is there and has the expected shape.
        let life_table = pack.assets.life_table.as_ref().expect("life table");
        assert_eq!(life_table.rows, 60);
        assert_eq!(life_table.cols, 5);
        assert!(life_table.data.iter().all(|value| value.is_finite()));

        // Nothing is fabricated: the pack stays blocked until it is filled in.
        let report = validate_pack(&pack, RunMode::Production);
        assert!(!report.production_ready, "{name}: the pack must not be production-ready");
        assert!(report.fixture_stamp.is_some(), "{name}: the pack must be stamped");
        let missing: Vec<&str> = pack
            .parameters
            .values()
            .filter(|parameter| parameter.as_f64().is_none())
            .map(|parameter| parameter.key.as_str())
            .collect();
        assert!(
            missing.contains(&"cfr_hospital") && missing.contains(&"cost_hospital"),
            "{name}: the country evidence should still be missing"
        );
    }
}
