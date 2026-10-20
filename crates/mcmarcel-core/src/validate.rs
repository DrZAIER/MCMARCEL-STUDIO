use crate::model::build_inputs;
use crate::types::{
    CountryPack, EvidenceStatus, RunMode, Severity, ValidationIssue, ValidationReport, AGE_MONTHS,
    CALENDAR_MONTHS,
};
use crate::Error;

const REQUIRED_PARAMETERS: &[&str] = &[
    "births",
    "incidence_hospital_0_5m",
    "incidence_hospital_6_11m",
    "ratio_outpatient_hospital",
    "proportion_non_medical_attended",
    "icu_share_of_hospital",
    "cfr_hospital",
    "cfr_icu",
    "cost_outpatient",
    "cost_hospital",
    "cost_icu",
    "cost_non_ma",
    "qaly_loss_outpatient",
    "qaly_loss_hospital",
    "qaly_loss_non_ma",
    "ve_maternal_hospital",
    "ve_maternal_outpatient",
    "ve_maternal_icu",
    "ve_maternal_death",
    "ve_mab_hospital",
    "ve_mab_outpatient",
    "ve_mab_icu",
    "ve_mab_death",
    "price_maternal_min",
    "price_maternal_max",
    "price_mab_min",
    "price_mab_max",
    "admin_cost_maternal",
    "admin_cost_mab",
    "wastage_maternal",
    "wastage_mab",
    "duration_maternal_years",
    "duration_mab_years",
    "discount_rate_cost",
    "discount_rate_effect",
    "life_expectancy_years",
    "qol_mean",
];

pub fn validate_pack(pack: &CountryPack, mode: RunMode) -> ValidationReport {
    let mut issues = Vec::new();
    let mut production_ready = true;

    if pack.schema_version != crate::types::SCHEMA_VERSION {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            code: "schema.version".into(),
            message: format!("Unsupported pack schema version {}.", pack.schema_version),
            parameter: None,
            blocks_production: true,
        });
        production_ready = false;
    }

    for key in REQUIRED_PARAMETERS {
        match pack.parameters.get(*key) {
            None => {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    code: "parameter.missing".into(),
                    message: format!("Required parameter '{key}' is absent."),
                    parameter: Some((*key).into()),
                    blocks_production: true,
                });
                production_ready = false;
            }
            Some(param) => {
                if param.as_f64().is_none() {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        code: "parameter.not_numeric".into(),
                        message: format!("Parameter '{}' must be numeric.", param.key),
                        parameter: Some(param.key.clone()),
                        blocks_production: true,
                    });
                    production_ready = false;
                }
                if param.status.blocks_production() {
                    issues.push(ValidationIssue {
                        severity: if mode == RunMode::Production { Severity::Error } else { Severity::Warning },
                        code: format!("evidence.{}", param.status.as_str()),
                        message: format!(
                            "{} is marked {}: {}",
                            param.label,
                            param.status.as_str().to_uppercase(),
                            param.notes.clone().unwrap_or_else(|| "replace with sourced evidence before production.".into())
                        ),
                        parameter: Some(param.key.clone()),
                        blocks_production: true,
                    });
                    production_ready = false;
                }
            }
        }
    }

    for param in pack.parameters.values() {
        if let Some(value) = param.as_f64() {
            if !value.is_finite() {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    code: "parameter.non_finite".into(),
                    message: format!("{} is not finite.", param.label),
                    parameter: Some(param.key.clone()),
                    blocks_production: true,
                });
                production_ready = false;
            }
        }
    }

    if pack.f64_or("births", 0.0) <= 0.0 {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            code: "births.non_positive".into(),
            message: "Annual births must be greater than zero.".into(),
            parameter: Some("births".into()),
            blocks_production: true,
        });
        production_ready = false;
    }

    for key in ["wastage_maternal", "wastage_mab"] {
        let value = pack.f64_or(key, 0.0);
        if !(0.0..=1.0).contains(&value) {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                code: "wastage.range".into(),
                message: format!("{key} must be between 0 and 1."),
                parameter: Some(key.into()),
                blocks_production: true,
            });
            production_ready = false;
        }
    }

    if pack.scenarios.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            code: "scenario.empty".into(),
            message: "At least one scenario is required.".into(),
            parameter: None,
            blocks_production: true,
        });
        production_ready = false;
    }
    if !pack.scenarios.iter().any(|s| s.is_reference) {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            code: "scenario.reference".into(),
            message: "A reference scenario is required for incremental analysis.".into(),
            parameter: None,
            blocks_production: true,
        });
        production_ready = false;
    }
    let mut seen = std::collections::BTreeSet::new();
    for scenario in &pack.scenarios {
        if !seen.insert(scenario.id.clone()) {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                code: "scenario.duplicate".into(),
                message: format!("Scenario id '{}' is duplicated.", scenario.id),
                parameter: None,
                blocks_production: true,
            });
            production_ready = false;
        }
        if !(0.0..=1.0).contains(&scenario.coverage_maternal) || !(0.0..=1.0).contains(&scenario.coverage_mab) {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                code: "scenario.coverage".into(),
                message: format!("Scenario '{}' has coverage outside 0..1.", scenario.name),
                parameter: None,
                blocks_production: true,
            });
            production_ready = false;
        }
        if !(1..=12).contains(&scenario.intervention_start) || !(1..=12).contains(&scenario.intervention_end) {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                code: "scenario.months".into(),
                message: format!("Scenario '{}' has an intervention month outside 1..12.", scenario.name),
                parameter: None,
                blocks_production: true,
            });
            production_ready = false;
        }
        if scenario.coverage_catch_up < 0.0 || scenario.coverage_catch_up > 1.0 {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                code: "scenario.catch_up".into(),
                message: format!("Scenario '{}' catch-up coverage must be 0..1.", scenario.name),
                parameter: None,
                blocks_production: true,
            });
            production_ready = false;
        }
    }

    if let Some(hosp) = &pack.assets.hospital_age_calendar {
        if hosp.rows != AGE_MONTHS || hosp.cols != CALENDAR_MONTHS {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                code: "asset.hospital.shape".into(),
                message: format!("Hospital asset must be {AGE_MONTHS}x{CALENDAR_MONTHS}."),
                parameter: None,
                blocks_production: true,
            });
            production_ready = false;
        }
    }
    if let Some(outpatient) = &pack.assets.outpatient_rate_age_sample {
        if outpatient.rows != AGE_MONTHS || outpatient.cols == 0 {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                code: "asset.outpatient.shape".into(),
                message: format!("Outpatient asset must be {AGE_MONTHS} x samples."),
                parameter: None,
                blocks_production: true,
            });
            production_ready = false;
        }
    }

    // The recurrent-wheezing module of the R implementation is not part of the
    // v1 engine. A non-zero wheezing cost or QALY loss would not reach the
    // results, so the pack is told about it explicitly instead of the value
    // being silently ignored.
    let wheezing_cost = pack.f64_or("cost_wheezing", 0.0);
    let wheezing_qaly = pack.f64_or("qaly_loss_wheezing", 0.0);
    if wheezing_cost.abs() > 0.0 || wheezing_qaly.abs() > 0.0 {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            code: "model.wheezing.not_implemented".into(),
            message: "The recurrent-wheezing module is not implemented in this engine version, so the wheezing cost and QALY loss are not applied to the results. Set both parameters to 0 to record that omission, and interpret the results with that limitation.".into(),
            parameter: None,
            blocks_production: false,
        });
    }

    if let Err(err) = build_inputs(pack) {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            code: "model.inputs".into(),
            message: format!("Model inputs could not be assembled: {err}"),
            parameter: None,
            blocks_production: true,
        });
        production_ready = false;
    }

    let has_synthetic = pack.parameters.values().any(|p| {
        matches!(p.status, EvidenceStatus::Synthetic | EvidenceStatus::Missing)
    });
    if has_synthetic || mode == RunMode::Fixture {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            code: "fixture.stamp".into(),
            message: "Fixture run: results must be stamped SYNTHETIC and must not be used for policy or reimbursement decisions.".into(),
            parameter: None,
            blocks_production: false,
        });
    }

    ValidationReport {
        production_ready,
        issues,
        fixture_stamp: if has_synthetic || mode == RunMode::Fixture {
            Some("SYNTHETIC FIXTURE — NOT FOR POLICY USE".into())
        } else {
            None
        },
        pack_hash: pack.hash(),
    }
}

pub fn assert_production_ready(pack: &CountryPack) -> Result<(), Error> {
    let report = validate_pack(pack, RunMode::Production);
    if report.production_ready {
        Ok(())
    } else {
        let blockers: Vec<String> = report
            .issues
            .iter()
            .filter(|issue| issue.blocks_production)
            .map(|issue| format!("{}: {}", issue.code, issue.message))
            .collect();
        Err(Error::ProductionGate(blockers.join("; ")))
    }
}
