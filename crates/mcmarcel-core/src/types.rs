use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::Error;

pub const AGE_MONTHS: usize = 60;
pub const CALENDAR_MONTHS: usize = 12;
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceStatus {
    Ok,
    Derived,
    Transfer,
    Convention,
    Missing,
    Synthetic,
}

impl EvidenceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Derived => "derived",
            Self::Transfer => "transfer",
            Self::Convention => "convention",
            Self::Missing => "missing",
            Self::Synthetic => "synthetic",
        }
    }

    pub fn blocks_production(self) -> bool {
        matches!(self, Self::Missing | Self::Synthetic)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub low: Option<f64>,
    #[serde(default)]
    pub high: Option<f64>,
    pub status: EvidenceStatus,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub source_ref: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub blocks_outputs: Vec<String>,
}

impl Parameter {
    pub fn new(key: impl Into<String>, label: impl Into<String>, value: f64, unit: impl Into<String>, status: EvidenceStatus) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            unit: unit.into(),
            value: Some(Value::from(value)),
            low: None,
            high: None,
            status,
            source: None,
            source_ref: None,
            url: None,
            notes: None,
            blocks_outputs: Vec::new(),
        }
    }

    pub fn text(key: impl Into<String>, label: impl Into<String>, value: impl Into<String>, status: EvidenceStatus) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            unit: String::new(),
            value: Some(Value::String(value.into())),
            low: None,
            high: None,
            status,
            source: None,
            source_ref: None,
            url: None,
            notes: None,
            blocks_outputs: Vec::new(),
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_range(mut self, low: f64, high: f64) -> Self {
        self.low = Some(low);
        self.high = Some(high);
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self.value.as_ref()? {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => s.trim().parse().ok(),
            Value::Bool(_) | Value::Null | Value::Array(_) | Value::Object(_) => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self.value.as_ref()? {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self.value.as_ref()? {
            Value::Bool(b) => Some(*b),
            Value::String(s) => match s.to_ascii_lowercase().as_str() {
                "true" | "yes" | "1" => Some(true),
                "false" | "no" | "0" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryMeta {
    pub id: String,
    pub name: String,
    pub iso3: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub price_year: i32,
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub perspective: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize, data: Vec<f64>) -> Result<Self, Error> {
        if rows == 0 || cols == 0 {
            return Err(Error::Validation("matrix dimensions must be positive".into()));
        }
        if data.len() != rows.checked_mul(cols).ok_or_else(|| Error::Validation("matrix dimensions overflow".into()))? {
            return Err(Error::Validation(format!(
                "matrix shape {rows}x{cols} requires {} values, got {}",
                rows * cols,
                data.len()
            )));
        }
        Ok(Self { rows, cols, data })
    }

    pub fn filled(rows: usize, cols: usize, value: f64) -> Result<Self, Error> {
        Self::new(rows, cols, vec![value; rows * cols])
    }

    pub fn from_rows(rows: &[Vec<f64>]) -> Result<Self, Error> {
        let nrows = rows.len();
        let ncols = rows.first().map(|r| r.len()).unwrap_or(0);
        if nrows == 0 || ncols == 0 {
            return Err(Error::Validation("matrix must have at least one row and column".into()));
        }
        if rows.iter().any(|r| r.len() != ncols) {
            return Err(Error::Validation("ragged matrix rows".into()));
        }
        let mut data = Vec::with_capacity(nrows * ncols);
        for row in rows {
            data.extend_from_slice(row);
        }
        Self::new(nrows, ncols, data)
    }

    #[inline]
    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.cols + col]
    }

    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row * self.cols + col] = value;
    }

    pub fn row_sums(&self) -> Vec<f64> {
        (0..self.rows)
            .map(|r| (0..self.cols).map(|c| self.get(r, c)).sum())
            .collect()
    }

    pub fn col_sums(&self) -> Vec<f64> {
        (0..self.cols)
            .map(|c| (0..self.rows).map(|r| self.get(r, c)).sum())
            .collect()
    }

    pub fn total(&self) -> f64 {
        self.data.iter().sum()
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|v| v * factor).collect(),
        }
    }

    pub fn validate_shape(&self, rows: usize, cols: usize) -> Result<(), Error> {
        if self.rows != rows || self.cols != cols {
            return Err(Error::Validation(format!(
                "expected matrix {rows}x{cols}, got {}x{}",
                self.rows, self.cols
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Assets {
    /// Hospital admissions, row = age month (0..59), column = calendar month (0..11).
    #[serde(default)]
    pub hospital_age_calendar: Option<Matrix>,
    /// ICU admissions with the same orientation as hospital.
    #[serde(default)]
    pub icu_age_calendar: Option<Matrix>,
    /// Annual outpatient cases per 1,000 infants by age; columns are epidemiological samples.
    #[serde(default)]
    pub outpatient_rate_age_sample: Option<Matrix>,
    /// Hospital case-fatality risks by age: column 0 = hospital, column 1 = ICU.
    #[serde(default)]
    pub hcfr_age: Option<Matrix>,
    /// Non-medically-attended proportion; rows are samples, columns are age months.
    #[serde(default)]
    pub non_ma_sample_age: Option<Matrix>,
    /// Optional life table with columns: age_years, le, le_disc, hrqale, hrqale_disc.
    #[serde(default)]
    pub life_table: Option<Matrix>,
    /// Optional age-banded unit costs. When present, the engine samples one unit
    /// cost per band and interpolates it over the age range, mirroring the
    /// `approx_age_by_sim` logic of the R implementation.
    #[serde(default)]
    pub cost_bands: Option<CostBands>,
    /// Optional age-specific efficacy profiles. The Belgian reference case uses
    /// them: efficacy against hospitalisation, for instance, falls over the
    /// first months of life in the trial-derived bootstrap files.
    #[serde(default)]
    pub efficacy_profile: Option<EfficacyProfile>,
}

/// Mean and standard deviation of one efficacy outcome, by age month.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EfficacyByAge {
    /// Mean efficacy for each age month (0..59).
    pub mean: Vec<f64>,
    /// Standard deviation of the sampled efficacy for each age month.
    #[serde(default)]
    pub sd: Vec<f64>,
}

/// Age-specific efficacy of one intervention product.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductEfficacy {
    #[serde(default)]
    pub hospital: EfficacyByAge,
    #[serde(default)]
    pub outpatient: EfficacyByAge,
    #[serde(default)]
    pub icu: EfficacyByAge,
    #[serde(default)]
    pub death: EfficacyByAge,
    #[serde(default)]
    pub non_ma: EfficacyByAge,
}

/// Age-specific efficacy of both intervention products.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EfficacyProfile {
    #[serde(default)]
    pub maternal: Option<ProductEfficacy>,
    #[serde(default)]
    pub mab: Option<ProductEfficacy>,
}

/// One age band of a unit cost.
///
/// The band stops at `max_age_months` (inclusive); the values live in the
/// parameter map so that they keep their evidence status and source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBand {
    /// Last age month (inclusive) the band applies to.
    pub max_age_months: u32,
    /// Parameter key holding the mean unit cost of the band.
    pub mean_key: String,
    /// Parameter key holding the standard deviation of the sampled unit cost.
    #[serde(default)]
    pub sd_key: Option<String>,
}

/// Age-banded unit costs for the four cost categories.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostBands {
    #[serde(default)]
    pub outpatient: Vec<CostBand>,
    #[serde(default)]
    pub hospital: Vec<CostBand>,
    #[serde(default)]
    pub icu: Vec<CostBand>,
    #[serde(default)]
    pub non_ma: Vec<CostBand>,
}

impl CostBands {
    /// Bands of one category, if the pack defines them.
    pub fn category(&self, category: &str) -> Option<&[CostBand]> {
        match category {
            "outpatient" => Some(&self.outpatient),
            "hospital" => Some(&self.hospital),
            "icu" => Some(&self.icu),
            "non_ma" => Some(&self.non_ma),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub is_reference: bool,
    #[serde(default)]
    pub coverage_maternal: f64,
    #[serde(default)]
    pub coverage_mab: f64,
    /// 1 = January, 12 = December. A year-round programme uses 1..12.
    #[serde(default = "default_start")]
    pub intervention_start: u8,
    #[serde(default = "default_end")]
    pub intervention_end: u8,
    /// 0 = no catch-up; values in (0, 1] scale catch-up coverage.
    #[serde(default)]
    pub coverage_catch_up: f64,
    #[serde(default)]
    pub price_maternal: Option<f64>,
    #[serde(default)]
    pub price_mab: Option<f64>,
    #[serde(default)]
    pub notes: String,
}

fn default_start() -> u8 { 1 }
fn default_end() -> u8 { 12 }

impl Scenario {
    pub fn reference() -> Self {
        Self {
            id: "none".into(),
            name: "No intervention".into(),
            is_reference: true,
            coverage_maternal: 0.0,
            coverage_mab: 0.0,
            intervention_start: 1,
            intervention_end: 12,
            coverage_catch_up: 0.0,
            price_maternal: None,
            price_mab: None,
            notes: "Comparator".into(),
        }
    }

    pub fn intervention_months(&self) -> u8 {
        if self.intervention_start == self.intervention_end {
            1
        } else if self.intervention_start < self.intervention_end {
            self.intervention_end - self.intervention_start + 1
        } else {
            12 - self.intervention_start + 1 + self.intervention_end
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryPack {
    pub schema_version: u32,
    pub country: CountryMeta,
    pub parameters: BTreeMap<String, Parameter>,
    #[serde(default)]
    pub assets: Assets,
    pub scenarios: Vec<Scenario>,
    #[serde(default)]
    pub notes: String,
}

impl CountryPack {
    pub fn parameter(&self, key: &str) -> Option<&Parameter> {
        self.parameters.get(key)
    }

    pub fn f64_or(&self, key: &str, default: f64) -> f64 {
        self.parameters.get(key).and_then(Parameter::as_f64).unwrap_or(default)
    }

    pub fn f64(&self, key: &str) -> Result<f64, Error> {
        self.parameters
            .get(key)
            .and_then(Parameter::as_f64)
            .ok_or_else(|| Error::MissingParameter(key.to_string()))
    }

    pub fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.parameters.get(key).and_then(Parameter::as_str).unwrap_or(default)
    }

    pub fn bool_or(&self, key: &str, default: bool) -> bool {
        self.parameters.get(key).and_then(Parameter::as_bool).unwrap_or(default)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunMode {
    Production,
    Fixture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSpec {
    pub draws: u32,
    pub seed: u64,
    pub mode: RunMode,
}

impl Default for RunSpec {
    fn default() -> Self {
        Self { draws: 500, seed: 20250101, mode: RunMode::Fixture }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawMetrics {
    pub draw: u32,
    pub scenario_id: String,
    pub scenario_name: String,
    pub reference_cases: f64,
    pub program_cases: f64,
    pub averted_cases: f64,
    pub reference_hospital: f64,
    pub program_hospital: f64,
    pub averted_hospital: f64,
    pub reference_deaths: f64,
    pub program_deaths: f64,
    pub averted_deaths: f64,
    pub reference_cost: f64,
    pub program_cost: f64,
    pub medical_cost_averted: f64,
    pub intervention_cost: f64,
    pub incremental_cost: f64,
    pub reference_qaly: f64,
    pub program_qaly: f64,
    pub qaly_averted: f64,
    pub fvp: f64,
    pub doses: f64,
    pub mean_price: f64,
    pub wastage: f64,
    pub intervention_months: f64,
    pub coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawPoint {
    pub draw: u32,
    pub scenario_id: String,
    pub reference_cost: f64,
    pub reference_effect: f64,
    pub program_cost: f64,
    pub program_effect: f64,
    pub incremental_cost: f64,
    pub incremental_effect: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScenarioSummary {
    pub scenario_id: String,
    pub scenario_name: String,
    pub is_reference: bool,
    pub coverage_maternal: f64,
    pub coverage_mab: f64,
    pub mean_reference_cases: f64,
    pub mean_program_cases: f64,
    pub mean_averted_cases: f64,
    pub mean_averted_hospital: f64,
    pub mean_averted_deaths: f64,
    pub mean_reference_cost: f64,
    pub mean_program_cost: f64,
    pub mean_medical_cost_averted: f64,
    pub mean_intervention_cost: f64,
    pub mean_incremental_cost: f64,
    pub mean_reference_qaly: f64,
    pub mean_program_qaly: f64,
    pub mean_qaly_averted: f64,
    pub mean_fvp: f64,
    pub mean_doses: f64,
    pub mean_price: f64,
    pub mean_wastage: f64,
    pub intervention_months: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcerRow {
    pub strategy_id: String,
    pub strategy: String,
    pub cost: f64,
    pub effect: f64,
    pub incremental_cost: Option<f64>,
    pub incremental_effect: Option<f64>,
    pub icer: Option<f64>,
    pub status: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InmbRow {
    pub threshold: f64,
    pub strategy_id: String,
    pub strategy: String,
    pub inmb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PstarRow {
    pub strategy_id: String,
    pub strategy: String,
    pub wtp: f64,
    pub pstar: Option<f64>,
    pub current_price: f64,
    pub price_floor: f64,
    pub slope: f64,
    pub doses: f64,
    pub fvp: f64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub thresholds: Vec<f64>,
    pub icer_table: Vec<IcerRow>,
    pub inmb_table: Vec<InmbRow>,
    pub pstar_table: Vec<PstarRow>,
    pub frontier: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub parameter: Option<String>,
    #[serde(default)]
    pub blocks_production: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationReport {
    pub production_ready: bool,
    pub issues: Vec<ValidationIssue>,
    pub fixture_stamp: Option<String>,
    pub pack_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub pack_id: String,
    pub pack_name: String,
    pub pack_hash: String,
    pub mode: RunMode,
    pub draws: u32,
    pub seed: u64,
    pub created_at_ms: u128,
    pub fixture_stamp: Option<String>,
    pub scenarios: Vec<ScenarioSummary>,
    pub points: Vec<DrawPoint>,
    pub analysis: AnalysisResult,
    pub validation: ValidationReport,
    pub warnings: Vec<String>,
}
