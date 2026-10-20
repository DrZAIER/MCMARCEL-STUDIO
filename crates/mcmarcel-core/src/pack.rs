use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::types::{
    CountryMeta, CountryPack, EvidenceStatus, Parameter, Scenario, SCHEMA_VERSION,
};
use crate::Error;

pub fn demo_pack() -> CountryPack {
    demo_pack_named("Tunisia (synthetic demo)", "TUN", "tun-demo")
}

pub fn demo_pack_named(name: &str, iso3: &str, id: &str) -> CountryPack {
    let mut p = BTreeMap::new();
    let synthetic = EvidenceStatus::Synthetic;
    let transfer = EvidenceStatus::Transfer;
    let convention = EvidenceStatus::Convention;

    let mut add = |param: Parameter| {
        p.insert(param.key.clone(), param);
    };

    add(Parameter::new("births", "Annual births", 136_509.0, "births/year", synthetic)
        .with_source("Synthetic demo value; replace with national vital registration or UN WPP."));
    add(Parameter::new("stillbirth_rate", "Stillbirth rate", 0.0, "proportion", convention));
    add(Parameter::new("pre_term_rate", "Pre-term birth rate", 0.0, "proportion", convention));
    add(Parameter::new("incidence_hospital_0_5m", "Hospital incidence, 0–5 months", 0.0279, "cases per infant", synthetic)
        .with_source("Illustrative shape only; not a Tunisia estimate."));
    add(Parameter::new("incidence_hospital_6_11m", "Hospital incidence, 6–11 months", 0.0121, "cases per infant", synthetic));
    add(Parameter::new("ratio_outpatient_hospital", "Outpatient to hospital ratio", 5.354, "ratio", synthetic));
    add(Parameter::new("proportion_non_medical_attended", "Non-medically-attended proportion", 0.40, "proportion", synthetic));
    add(Parameter::new("icu_share_of_hospital", "ICU share of hospital admissions", 0.10, "proportion", synthetic));
    add(Parameter::new("cfr_hospital", "Hospital case-fatality risk", 0.0005, "proportion", synthetic));
    add(Parameter::new("cfr_icu", "ICU case-fatality risk", 0.0043, "proportion", synthetic));
    add(Parameter::new("age_decay_hospital", "Hospital age decay", 0.92, "per month", convention));
    add(Parameter::new("age_decay_outpatient", "Outpatient age decay", 0.90, "per month", convention));
    add(Parameter::new("age_decay_cfr", "CFR age decay", 0.97, "per month", convention));
    add(Parameter::new("seasonality_amplitude", "Seasonality amplitude", 0.0, "0–0.8", convention));
    add(Parameter::new("life_expectancy_years", "Life expectancy at birth", 76.7, "years", transfer));
    add(Parameter::new("qol_mean", "Mean population utility", 0.90, "utility", synthetic));
    add(Parameter::new("discount_rate_cost", "Cost discount rate", 0.03, "annual", convention));
    add(Parameter::new("discount_rate_effect", "Effect discount rate", 0.03, "annual", convention));

    add(Parameter::new("cost_outpatient", "Outpatient cost per case", 90.0, "local currency", synthetic));
    add(Parameter::new("cost_hospital", "Hospital cost per admission", 2_500.0, "local currency", synthetic));
    add(Parameter::new("cost_icu", "ICU cost per admission", 9_000.0, "local currency", synthetic));
    add(Parameter::new("cost_non_ma", "Non-MA cost per case", 1.0, "local currency", synthetic));
    add(Parameter::new("cost_wheezing", "Wheezing sequelae cost", 0.0, "local currency", synthetic));
    add(Parameter::new("cost_outpatient_sd", "Outpatient cost SD", 18.0, "local currency", synthetic));
    add(Parameter::new("cost_hospital_sd", "Hospital cost SD", 500.0, "local currency", synthetic));
    add(Parameter::new("cost_icu_sd", "ICU cost SD", 1_800.0, "local currency", synthetic));
    add(Parameter::new("cost_non_ma_sd", "Non-MA cost SD", 0.2, "local currency", synthetic));
    add(Parameter::new("cost_wheezing_sd", "Wheezing cost SD", 0.0, "local currency", synthetic));

    add(Parameter::new("qaly_loss_outpatient", "QALY loss, outpatient", 0.0059, "QALY/case", synthetic));
    add(Parameter::new("qaly_loss_hospital", "QALY loss, hospital", 0.0098, "QALY/case", synthetic));
    add(Parameter::new("qaly_loss_non_ma", "QALY loss, non-MA", 0.0034, "QALY/case", synthetic));
    add(Parameter::new("qaly_loss_wheezing", "QALY loss, wheezing", 0.0183, "QALY/case", synthetic));
    add(Parameter::new("qaly_loss_outpatient_sd", "QALY loss SD, outpatient", 0.0012, "QALY/case", synthetic));
    add(Parameter::new("qaly_loss_hospital_sd", "QALY loss SD, hospital", 0.0020, "QALY/case", synthetic));
    add(Parameter::new("qaly_loss_non_ma_sd", "QALY loss SD, non-MA", 0.0007, "QALY/case", synthetic));
    add(Parameter::new("qaly_loss_wheezing_sd", "QALY loss SD, wheezing", 0.0037, "QALY/case", synthetic));

    add(Parameter::new("ve_maternal_hospital", "Maternal VE, hospital", 0.568, "proportion", transfer));
    add(Parameter::new("ve_maternal_hospital_sd", "Maternal VE SD, hospital", 0.05, "proportion", transfer));
    add(Parameter::new("ve_maternal_outpatient", "Maternal VE, outpatient", 0.513, "proportion", transfer));
    add(Parameter::new("ve_maternal_outpatient_sd", "Maternal VE SD, outpatient", 0.05, "proportion", transfer));
    add(Parameter::new("ve_maternal_icu", "Maternal VE, ICU", 0.568, "proportion", transfer));
    add(Parameter::new("ve_maternal_icu_sd", "Maternal VE SD, ICU", 0.05, "proportion", transfer));
    add(Parameter::new("ve_maternal_death", "Maternal VE, death", 0.568, "proportion", transfer));
    add(Parameter::new("ve_maternal_death_sd", "Maternal VE SD, death", 0.05, "proportion", transfer));
    add(Parameter::new("ve_maternal_non_ma", "Maternal VE, non-MA", 0.513, "proportion", transfer));
    add(Parameter::new("ve_maternal_non_ma_sd", "Maternal VE SD, non-MA", 0.05, "proportion", transfer));
    add(Parameter::new("duration_maternal_years", "Maternal protection duration", 0.5, "years", transfer));

    add(Parameter::new("ve_mab_hospital", "mAb VE, hospital", 0.832, "proportion", transfer));
    add(Parameter::new("ve_mab_hospital_sd", "mAb VE SD, hospital", 0.03, "proportion", transfer));
    add(Parameter::new("ve_mab_outpatient", "mAb VE, outpatient", 0.745, "proportion", transfer));
    add(Parameter::new("ve_mab_outpatient_sd", "mAb VE SD, outpatient", 0.03, "proportion", transfer));
    add(Parameter::new("ve_mab_icu", "mAb VE, ICU", 0.832, "proportion", transfer));
    add(Parameter::new("ve_mab_icu_sd", "mAb VE SD, ICU", 0.03, "proportion", transfer));
    add(Parameter::new("ve_mab_death", "mAb VE, death", 0.832, "proportion", transfer));
    add(Parameter::new("ve_mab_death_sd", "mAb VE SD, death", 0.03, "proportion", transfer));
    add(Parameter::new("ve_mab_non_ma", "mAb VE, non-MA", 0.745, "proportion", transfer));
    add(Parameter::new("ve_mab_non_ma_sd", "mAb VE SD, non-MA", 0.03, "proportion", transfer));
    add(Parameter::new("duration_mab_years", "mAb protection duration", 0.5, "years", transfer));

    add(Parameter::new("coverage_maternal", "Maternal vaccine coverage", 0.97, "proportion", convention));
    add(Parameter::new("coverage_mab", "mAb coverage", 0.99, "proportion", convention));
    add(Parameter::new("price_maternal_min", "Maternal vaccine price, low", 100.0, "local currency/dose", synthetic));
    add(Parameter::new("price_maternal_max", "Maternal vaccine price, high", 150.0, "local currency/dose", synthetic));
    add(Parameter::new("price_mab_min", "mAb price, low", 200.0, "local currency/dose", synthetic));
    add(Parameter::new("price_mab_max", "mAb price, high", 300.0, "local currency/dose", synthetic));
    add(Parameter::new("admin_cost_maternal", "Maternal administration cost", 29.0877, "local currency/dose", synthetic));
    add(Parameter::new("admin_cost_mab", "mAb administration cost", 29.0877, "local currency/dose", synthetic));
    add(Parameter::new("wastage_maternal", "Maternal vaccine wastage", 0.05, "proportion", convention));
    add(Parameter::new("wastage_mab", "mAb wastage", 0.05, "proportion", convention));
    add(Parameter::new("implementation_cost_maternal", "Maternal implementation cost", 0.0, "local currency/dose", synthetic));
    add(Parameter::new("implementation_cost_mab", "mAb implementation cost", 0.0, "local currency/dose", synthetic));

    add(Parameter::new("gdp_per_capita", "GDP per capita (local currency)", 4_187.64, "local currency", transfer));
    add(Parameter::new("gdp_fx_usd", "Exchange rate, local currency per USD", 3.10745, "local currency/USD", transfer));
    add(Parameter::new("wtp_multiplier_0_5", "WTP threshold, 0.5× GDP", 0.5, "multiple", convention));
    add(Parameter::new("wtp_multiplier_1", "WTP threshold, 1× GDP", 1.0, "multiple", convention));
    add(Parameter::new("wtp_multiplier_3", "WTP threshold, 3× GDP", 3.0, "multiple", convention));
    add(Parameter::text("analysis_perspective", "Analysis perspective", "healthcare", convention));
    add(Parameter::text("currency", "Currency", "TND", convention));

    let country = CountryMeta {
        id: id.into(),
        name: name.into(),
        iso3: iso3.into(),
        currency: "TND".into(),
        price_year: 2024,
        locale: "en".into(),
        perspective: "healthcare".into(),
        region: "Africa".into(),
        notes: "Synthetic demonstration pack. Replace every synthetic value and asset with a sourced, reviewed country value before production use.".into(),
    };

    let mut maternal = Scenario::reference();
    maternal.id = "maternal".into();
    maternal.name = "Maternal vaccine".into();
    maternal.is_reference = false;
    maternal.coverage_maternal = 0.97;
    maternal.coverage_mab = 0.0;
    maternal.notes = "Year-round maternal vaccination, 97% coverage.".into();

    let mut mab = Scenario::reference();
    mab.id = "mab".into();
    mab.name = "Monoclonal antibody".into();
    mab.is_reference = false;
    mab.coverage_maternal = 0.0;
    mab.coverage_mab = 0.99;
    mab.notes = "Year-round mAb, 99% coverage.".into();

    CountryPack {
        schema_version: SCHEMA_VERSION,
        country,
        parameters: p,
        assets: Default::default(),
        scenarios: vec![Scenario::reference(), maternal, mab],
        notes: "McMarcel Studio demo pack. Fixture runs are stamped SYNTHETIC. Production mode is intentionally blocked until all evidence is sourced and reviewed.".into(),
    }
}

impl CountryPack {
    pub fn from_json_str(input: &str) -> Result<Self, Error> {
        let pack: Self = serde_json::from_str(input)?;
        Ok(pack)
    }

    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let text = fs::read_to_string(path)?;
        Self::from_json_str(&text)
    }

    pub fn to_json_string(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn to_json_pretty(&self) -> Result<String, Error> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn save_json_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.to_json_pretty()?)?;
        Ok(())
    }

    pub fn hash(&self) -> String {
        let canonical = serde_json::to_vec(self).unwrap_or_default();
        blake3::hash(&canonical).to_hex().to_string()
    }

    pub fn with_identity(mut self, id: impl Into<String>, name: impl Into<String>, iso3: impl Into<String>) -> Self {
        self.country.id = id.into();
        self.country.name = name.into();
        self.country.iso3 = iso3.into();
        self
    }

    pub fn set_parameter_value(&mut self, key: &str, value: Value) -> Result<(), Error> {
        let param = self
            .parameters
            .get_mut(key)
            .ok_or_else(|| Error::MissingParameter(key.to_string()))?;
        param.value = Some(value);
        Ok(())
    }

    pub fn set_number(&mut self, key: &str, value: f64) -> Result<(), Error> {
        self.set_parameter_value(key, json!(value))
    }

    pub fn set_text(&mut self, key: &str, value: &str) -> Result<(), Error> {
        self.set_parameter_value(key, Value::String(value.to_string()))
    }

    pub fn wtp_thresholds(&self) -> Vec<f64> {
        let gdp = self.f64_or("gdp_per_capita", 0.0);
        if gdp <= 0.0 {
            return vec![1_000.0, 2_000.0, 6_000.0];
        }
        ["wtp_multiplier_0_5", "wtp_multiplier_1", "wtp_multiplier_3"]
            .iter()
            .map(|key| {
                let multiple = self.f64_or(key, match *key {
                    "wtp_multiplier_0_5" => 0.5,
                    "wtp_multiplier_1" => 1.0,
                    _ => 3.0,
                });
                (gdp * multiple).max(0.0)
            })
            .collect()
    }

    pub fn missing_or_synthetic_parameters(&self) -> Vec<&Parameter> {
        self.parameters
            .values()
            .filter(|p| p.status.blocks_production())
            .collect()
    }

    pub fn scenario(&self, id: &str) -> Option<&Scenario> {
        self.scenarios.iter().find(|s| s.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_pack_round_trips() {
        let pack = demo_pack();
        let json = pack.to_json_string().unwrap();
        let restored = CountryPack::from_json_str(&json).unwrap();
        assert_eq!(restored.country.iso3, "TUN");
        assert_eq!(restored.scenarios.len(), 3);
    }

    #[test]
    fn demo_pack_has_positive_thresholds() {
        let pack = demo_pack();
        let thresholds = pack.wtp_thresholds();
        assert_eq!(thresholds.len(), 3);
        assert!(thresholds[2] > thresholds[0]);
    }
}
