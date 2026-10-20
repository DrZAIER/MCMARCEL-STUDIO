use serde::{Deserialize, Serialize};

use crate::rng::DrawRng;
use crate::types::{Assets, CountryPack, Matrix, AGE_MONTHS, CALENDAR_MONTHS};
use crate::Error;

pub const OUTPATIENT_SAMPLES: usize = 31;
pub const NON_MA_SAMPLES: usize = 31;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UncertainValue {
    pub mean: f64,
    pub sd: f64,
}

impl UncertainValue {
    pub fn fixed(mean: f64) -> Self {
        Self { mean, sd: 0.0 }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriceRange {
    pub low: f64,
    pub high: f64,
}

impl PriceRange {
    pub fn mean(self) -> f64 {
        (self.low + self.high) / 2.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficacySet {
    pub hospital: UncertainValue,
    pub outpatient: UncertainValue,
    pub icu: UncertainValue,
    pub death: UncertainValue,
    pub non_ma: UncertainValue,
    /// Optional age-specific profile; when present it replaces the scalar values.
    #[serde(default)]
    pub profile: Option<crate::types::ProductEfficacy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionProduct {
    pub name: String,
    pub price: PriceRange,
    pub administration_cost: f64,
    pub wastage: f64,
    pub implementation_cost: f64,
    pub duration_months: usize,
    pub efficacy: EfficacySet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostParams {
    pub outpatient: UncertainValue,
    pub hospital: UncertainValue,
    pub icu: UncertainValue,
    pub non_ma: UncertainValue,
    pub wheezing: UncertainValue,
}

/// One resolved cost band: the age range it covers and the sampled unit cost.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResolvedCostBand {
    pub max_age_months: usize,
    pub mean: f64,
    pub sd: f64,
}

/// Age-banded costs, resolved from the pack's parameter map.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BandedCosts {    pub outpatient: Vec<ResolvedCostBand>,
    pub hospital: Vec<ResolvedCostBand>,
    pub icu: Vec<ResolvedCostBand>,
    pub non_ma: Vec<ResolvedCostBand>,
}

impl BandedCosts {
    /// Bands of one category.
    pub fn category(&self, category: &str) -> &[ResolvedCostBand] {
        match category {
            "outpatient" => &self.outpatient,
            "hospital" => &self.hospital,
            "icu" => &self.icu,
            "non_ma" => &self.non_ma,
            _ => &[],
        }
    }
}

/// Resolve the age-banded costs of a pack, if it defines any.
pub fn build_banded_costs(pack: &CountryPack) -> Result<BandedCosts, Error> {
    let mut banded = BandedCosts::default();
    let Some(structure) = pack.assets.cost_bands.as_ref() else {
        return Ok(banded);
    };
    for (category, bands) in [
        ("outpatient", &structure.outpatient),
        ("hospital", &structure.hospital),
        ("icu", &structure.icu),
        ("non_ma", &structure.non_ma),
    ] {
        let mut resolved = Vec::with_capacity(bands.len());
        let mut previous: i64 = -1;
        for band in bands {
            let max_age = band.max_age_months as i64;
            if max_age < 0 || max_age >= AGE_MONTHS as i64 {
                return Err(Error::Validation(format!(
                    "{category} cost band ends at age {max_age}, which is outside 0..{}",
                    AGE_MONTHS - 1
                )));
            }
            if max_age <= previous {
                return Err(Error::Validation(format!(
                    "{category} cost bands must be ordered by increasing age"
                )));
            }
            previous = max_age;
            let mean = pack.f64(&band.mean_key).map_err(|_| {
                Error::MissingParameter(format!("{} (cost band)", band.mean_key))
            })?;
            if !mean.is_finite() || mean < 0.0 {
                return Err(Error::Validation(format!(
                    "{} must be finite and non-negative",
                    band.mean_key
                )));
            }
            let sd = band
                .sd_key
                .as_ref()
                .and_then(|key| pack.parameters.get(key))
                .and_then(|parameter| parameter.as_f64())
                .unwrap_or(0.0)
                .abs();
            resolved.push(ResolvedCostBand { max_age_months: max_age as usize, mean, sd });
        }
        match category {
            "outpatient" => banded.outpatient = resolved,
            "hospital" => banded.hospital = resolved,
            "icu" => banded.icu = resolved,
            _ => banded.non_ma = resolved,
        }
    }
    Ok(banded)
}

/// Interpolate sampled band costs over the age range.
///
/// Mirrors `approx_age_by_sim` in the R implementation: the value of a band
/// applies from the previous break up to and including its own break.
pub fn interpolate_cost_bands(bands: &[ResolvedCostBand], sampled: &[f64]) -> Vec<f64> {
    let mut costs = vec![0.0; AGE_MONTHS];
    if bands.is_empty() {
        return costs;
    }
    for (index, band) in bands.iter().enumerate() {
        let value = sampled.get(index).copied().unwrap_or(band.mean);
        let start = if index == 0 { 0 } else { bands[index - 1].max_age_months + 1 };
        for age in start..=band.max_age_months.min(AGE_MONTHS - 1) {
            costs[age] = value;
        }
    }
    costs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QalyParams {
    pub outpatient: UncertainValue,
    pub hospital: UncertainValue,
    pub non_ma: UncertainValue,
    pub wheezing: UncertainValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeTable {
    pub le: Vec<f64>,
    pub le_disc: Vec<f64>,
    pub hrqale: Vec<f64>,
    pub hrqale_disc: Vec<f64>,
}

impl LifeTable {
    pub fn from_matrix(matrix: &Matrix) -> Result<Self, Error> {
        matrix.validate_shape(AGE_MONTHS, 5)?;
        let mut le = Vec::with_capacity(AGE_MONTHS);
        let mut le_disc = Vec::with_capacity(AGE_MONTHS);
        let mut hrqale = Vec::with_capacity(AGE_MONTHS);
        let mut hrqale_disc = Vec::with_capacity(AGE_MONTHS);
        for row in 0..AGE_MONTHS {
            le.push(matrix.get(row, 1).max(0.0));
            le_disc.push(matrix.get(row, 2).max(0.0));
            hrqale.push(matrix.get(row, 3).max(0.0));
            hrqale_disc.push(matrix.get(row, 4).max(0.0));
        }
        Ok(Self { le, le_disc, hrqale, hrqale_disc })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInputs {
    pub births: f64,
    pub target_population: f64,
    pub stillbirth_rate: f64,
    pub pre_term_rate: f64,
    pub hospital: Matrix,
    pub icu: Matrix,
    pub outpatient_rate: Matrix,
    pub hcfr: Matrix,
    pub non_ma: Matrix,
    pub life: LifeTable,
    pub costs: CostParams,
    /// Age-banded unit costs; empty when the pack uses a single cost per category.
    pub banded_costs: BandedCosts,
    pub qaly: QalyParams,
    pub maternal: InterventionProduct,
    pub mab: InterventionProduct,
    pub discount_cost: f64,
    pub discount_effect: f64,
}

pub fn build_inputs(pack: &CountryPack) -> Result<ModelInputs, Error> {
    let births = pack.f64("births")?;
    if !births.is_finite() || births <= 0.0 {
        return Err(Error::Validation("births must be positive and finite".into()));
    }
    let stillbirth_rate = pack.f64_or("stillbirth_rate", 0.0).clamp(0.0, 0.5);
    let target_population = births * (1.0 - stillbirth_rate);
    let generated = build_generated_assets(pack)?;

    let hospital = pack
        .assets
        .hospital_age_calendar
        .clone()
        .or(generated.hospital_age_calendar)
        .ok_or_else(|| Error::Pack("hospital matrix could not be built".into()))?;
    hospital.validate_shape(AGE_MONTHS, CALENDAR_MONTHS)?;

    let icu = pack
        .assets
        .icu_age_calendar
        .clone()
        .or(generated.icu_age_calendar)
        .ok_or_else(|| Error::Pack("ICU matrix could not be built".into()))?;
    icu.validate_shape(AGE_MONTHS, CALENDAR_MONTHS)?;

    let outpatient_rate = pack
        .assets
        .outpatient_rate_age_sample
        .clone()
        .or(generated.outpatient_rate_age_sample)
        .ok_or_else(|| Error::Pack("outpatient matrix could not be built".into()))?;
    if outpatient_rate.rows != AGE_MONTHS || outpatient_rate.cols == 0 {
        return Err(Error::Validation(format!(
            "outpatient matrix must be {AGE_MONTHS} x samples, got {}x{}",
            outpatient_rate.rows, outpatient_rate.cols
        )));
    }

    let hcfr = pack
        .assets
        .hcfr_age
        .clone()
        .or(generated.hcfr_age)
        .ok_or_else(|| Error::Pack("hCFR matrix could not be built".into()))?;
    hcfr.validate_shape(AGE_MONTHS, 2)?;

    let non_ma = pack
        .assets
        .non_ma_sample_age
        .clone()
        .or(generated.non_ma_sample_age)
        .ok_or_else(|| Error::Pack("non-MA matrix could not be built".into()))?;
    if non_ma.rows == 0 || non_ma.cols != AGE_MONTHS {
        return Err(Error::Validation(format!(
            "non-MA matrix must be samples x {AGE_MONTHS}, got {}x{}",
            non_ma.rows, non_ma.cols
        )));
    }

    let life_matrix = pack
        .assets
        .life_table
        .clone()
        .or(generated.life_table)
        .ok_or_else(|| Error::Pack("life table could not be built".into()))?;
    let life = LifeTable::from_matrix(&life_matrix)?;

    let discount_cost = pack.f64_or("discount_rate_cost", 0.03).clamp(0.0, 0.25);
    let discount_effect = pack.f64_or("discount_rate_effect", 0.03).clamp(0.0, 0.25);
    let pre_term_rate = pack.f64_or("pre_term_rate", 0.0).clamp(0.0, 0.5);

    let costs = CostParams {
        outpatient: uncertain(pack, "cost_outpatient", "cost_outpatient_sd", 0.20)?,
        hospital: uncertain(pack, "cost_hospital", "cost_hospital_sd", 0.20)?,
        icu: uncertain(pack, "cost_icu", "cost_icu_sd", 0.20)?,
        non_ma: uncertain(pack, "cost_non_ma", "cost_non_ma_sd", 0.20)?,
        wheezing: uncertain(pack, "cost_wheezing", "cost_wheezing_sd", 0.20)?,
    };
    let banded_costs = build_banded_costs(pack)?;
    let qaly = QalyParams {
        outpatient: uncertain(pack, "qaly_loss_outpatient", "qaly_loss_outpatient_sd", 0.20)?,
        hospital: uncertain(pack, "qaly_loss_hospital", "qaly_loss_hospital_sd", 0.20)?,
        non_ma: uncertain(pack, "qaly_loss_non_ma", "qaly_loss_non_ma_sd", 0.20)?,
        wheezing: uncertain(pack, "qaly_loss_wheezing", "qaly_loss_wheezing_sd", 0.20)?,
    };

    let maternal = intervention_product(pack, "maternal", "Maternal RSV vaccine", "ve_maternal", 0.5)?;
    let mab = intervention_product(pack, "mab", "Monoclonal antibody", "ve_mab", 0.5)?;

    Ok(ModelInputs {
        births,
        target_population,
        stillbirth_rate,
        pre_term_rate,
        hospital,
        icu,
        outpatient_rate,
        hcfr,
        non_ma,
        life,
        costs,
        banded_costs,
        qaly,
        maternal,
        mab,
        discount_cost,
        discount_effect,
    })
}

fn uncertain(pack: &CountryPack, mean_key: &str, sd_key: &str, default_sd_fraction: f64) -> Result<UncertainValue, Error> {
    let mean = pack.f64(mean_key)?;
    if !mean.is_finite() || mean < 0.0 {
        return Err(Error::Validation(format!("{mean_key} must be finite and non-negative")));
    }
    let sd = pack
        .parameters
        .get(sd_key)
        .and_then(|p| p.as_f64())
        .unwrap_or(mean * default_sd_fraction)
        .max(0.0);
    Ok(UncertainValue { mean, sd })
}

fn efficacy(pack: &CountryPack, key: &str) -> Result<UncertainValue, Error> {
    let mean = pack.f64(key)?;
    if !(0.0..=1.0).contains(&mean) {
        return Err(Error::Validation(format!("{key} must be between 0 and 1")));
    }
    let sd = pack
        .parameters
        .get(&format!("{key}_sd"))
        .and_then(|p| p.as_f64())
        .unwrap_or(0.0)
        .max(0.0);
    Ok(UncertainValue { mean, sd })
}

fn intervention_product(pack: &CountryPack, prefix: &str, name: &str, ve_prefix: &str, default_duration: f64) -> Result<InterventionProduct, Error> {
    let price_low = pack
        .parameters
        .get(&format!("price_{prefix}_min"))
        .and_then(|p| p.as_f64())
        .or_else(|| pack.parameters.get(&format!("price_{prefix}")).and_then(|p| p.as_f64()))
        .ok_or_else(|| Error::MissingParameter(format!("price_{prefix}_min")))?;
    let price_high = pack
        .parameters
        .get(&format!("price_{prefix}_max"))
        .and_then(|p| p.as_f64())
        .unwrap_or(price_low)
        .max(price_low);
    if price_low < 0.0 {
        return Err(Error::Validation(format!("price_{prefix}_min must be non-negative")));
    }

    let profile = match prefix {
        "maternal" => pack.assets.efficacy_profile.as_ref().and_then(|p| p.maternal.clone()),
        _ => pack.assets.efficacy_profile.as_ref().and_then(|p| p.mab.clone()),
    };
    let efficacy = EfficacySet {
        hospital: efficacy(pack, &format!("{ve_prefix}_hospital"))?,
        outpatient: efficacy(pack, &format!("{ve_prefix}_outpatient"))?,
        icu: efficacy(pack, &format!("{ve_prefix}_icu"))?,
        death: efficacy(pack, &format!("{ve_prefix}_death"))?,
        non_ma: match efficacy(pack, &format!("{ve_prefix}_non_ma")) {
            Ok(value) => value,
            Err(Error::MissingParameter(_)) => efficacy(pack, &format!("{ve_prefix}_outpatient"))?,
            Err(err) => return Err(err),
        },
        profile,
    };

    let duration_years = pack
        .parameters
        .get(&format!("duration_{prefix}_years"))
        .and_then(|p| p.as_f64())
        .unwrap_or(default_duration);
    let duration_months = (duration_years * 12.0).round().clamp(0.0, 60.0) as usize;

    Ok(InterventionProduct {
        name: name.into(),
        price: PriceRange { low: price_low, high: price_high },
        administration_cost: pack.f64_or(&format!("admin_cost_{prefix}"), 0.0).max(0.0),
        wastage: pack.f64_or(&format!("wastage_{prefix}"), 0.0).clamp(0.0, 1.0),
        implementation_cost: pack.f64_or(&format!("implementation_cost_{prefix}"), 0.0).max(0.0),
        duration_months,
        efficacy,
    })
}

/// Build a deterministic illustrative asset set from high-level coverage targets.
/// These assets are explicitly synthetic and must never be treated as country evidence.
pub fn build_generated_assets(pack: &CountryPack) -> Result<Assets, Error> {
    let births = pack.f64("births")?;
    let inc0 = pack.f64_or("incidence_hospital_0_5m", 0.0279).max(0.0);
    let inc6 = pack.f64_or("incidence_hospital_6_11m", 0.0121).max(0.0);
    let ratio_out = pack.f64_or("ratio_outpatient_hospital", 5.354).max(0.0);
    let prop_non_ma = pack.f64_or("proportion_non_medical_attended", 0.4).clamp(0.0, 1.0);
    let icu_share = pack.f64_or("icu_share_of_hospital", 0.10).clamp(0.0, 1.0);
    let cfr_hosp = pack.f64_or("cfr_hospital", 0.0005).clamp(0.0, 1.0);
    let cfr_icu = pack.f64_or("cfr_icu", 0.0043).clamp(0.0, 1.0);
    let decay_hosp = pack.f64_or("age_decay_hospital", 0.92).clamp(0.5, 0.999);
    let decay_out = pack.f64_or("age_decay_outpatient", 0.90).clamp(0.5, 0.999);
    let decay_cfr = pack.f64_or("age_decay_cfr", 0.97).clamp(0.5, 0.999);
    let seasonality = pack.f64_or("seasonality_amplitude", 0.0).clamp(0.0, 0.8);
    let life_expectancy = pack.f64_or("life_expectancy_years", 76.7).max(1.0);
    let qol_mean = pack.f64_or("qol_mean", 0.9).clamp(0.0, 1.0);
    let disc_effect = pack.f64_or("discount_rate_effect", 0.03).clamp(0.0, 0.25);

    let target0 = births * inc0;
    let target6 = births * inc6;
    let ratio = if target0 > 0.0 && target6 > 0.0 {
        (target6 / target0).powf(1.0 / 6.0).clamp(0.5, 0.995)
    } else {
        0.9
    };
    let base = if ratio < 1.0 {
        target0 * (1.0 - ratio) / (1.0 - ratio.powi(6))
    } else {
        target0 / 6.0
    };

    let mut hosp_age = vec![0.0; AGE_MONTHS];
    for age in 0..6 {
        hosp_age[age] = base * ratio.powi(age as i32);
    }
    for age in 6..AGE_MONTHS {
        let anchor = if base > 0.0 { base * ratio.powi(5) } else { target6.max(0.0) / 6.0 };
        hosp_age[age] = anchor * decay_hosp.powi((age - 5) as i32);
    }

    let season_weights = season_weights(seasonality);
    let mut hospital = Matrix::filled(AGE_MONTHS, CALENDAR_MONTHS, 0.0)?;
    for age in 0..AGE_MONTHS {
        for month in 0..CALENDAR_MONTHS {
            hospital.set(age, month, hosp_age[age] * season_weights[month]);
        }
    }

    let mut icu = Matrix::filled(AGE_MONTHS, CALENDAR_MONTHS, 0.0)?;
    for age in 0..AGE_MONTHS {
        for month in 0..CALENDAR_MONTHS {
            icu.set(age, month, hospital.get(age, month) * icu_share);
        }
    }

    // The outpatient ratio is defined as (outpatient + non-MA) / hospital.
    let out_cases_age: Vec<f64> = hosp_age
        .iter()
        .enumerate()
        .map(|(age, value)| value * ratio_out / (1.0 + prop_non_ma) * decay_out.powi(age as i32) / decay_hosp.powi(age as i32))
        .collect();
    let out_rate_age: Vec<f64> = out_cases_age.iter().map(|cases| cases / births * 1000.0).collect();
    let mut outpatient = Matrix::filled(AGE_MONTHS, OUTPATIENT_SAMPLES, 0.0)?;
    for age in 0..AGE_MONTHS {
        for sample in 0..OUTPATIENT_SAMPLES {
            let jitter = if sample == 0 {
                1.0
            } else {
                DrawRng::new(20250101, (age * OUTPATIENT_SAMPLES + sample) as u32).uniform(0.85, 1.15)
            };
            outpatient.set(age, sample, out_rate_age[age] * jitter);
        }
    }

    let mut hcfr = Matrix::filled(AGE_MONTHS, 2, 0.0)?;
    let cfr_shape: Vec<f64> = (0..AGE_MONTHS).map(|age| decay_cfr.powi(age as i32)).collect();
    let weight_hosp: f64 = (0..12).map(|age| hosp_age[age] * cfr_shape[age]).sum();
    let base_hosp = if weight_hosp > 0.0 { cfr_hosp * hosp_age[..12].iter().sum::<f64>() / weight_hosp } else { cfr_hosp };
    let weight_icu: f64 = (0..12).map(|age| hosp_age[age] * icu_share * cfr_shape[age]).sum();
    let base_icu = if weight_icu > 0.0 { cfr_icu * hosp_age[..12].iter().sum::<f64>() * icu_share / weight_icu } else { cfr_icu };
    for age in 0..AGE_MONTHS {
        hcfr.set(age, 0, (base_hosp * cfr_shape[age]).clamp(0.0, 1.0));
        hcfr.set(age, 1, (base_icu * cfr_shape[age]).clamp(0.0, 1.0));
    }

    let mut non_ma = Matrix::filled(NON_MA_SAMPLES, AGE_MONTHS, prop_non_ma)?;
    for sample in 1..NON_MA_SAMPLES {
        for age in 0..AGE_MONTHS {
            let jitter = DrawRng::new(20250102, (sample * AGE_MONTHS + age) as u32).uniform(0.85, 1.15);
            non_ma.set(sample, age, (prop_non_ma * jitter).clamp(0.0, 1.0));
        }
    }

    let mut life = Matrix::filled(AGE_MONTHS, 5, 0.0)?;
    for age in 0..AGE_MONTHS {
        let le = (life_expectancy - age as f64 / 12.0).max(0.0);
        let le_disc = discounted_life_years(le, disc_effect);
        life.set(age, 0, age as f64 / 12.0);
        life.set(age, 1, le);
        life.set(age, 2, le_disc);
        life.set(age, 3, le * qol_mean);
        life.set(age, 4, le_disc * qol_mean);
    }

    Ok(Assets {
        hospital_age_calendar: Some(hospital),
        icu_age_calendar: Some(icu),
        outpatient_rate_age_sample: Some(outpatient),
        hcfr_age: Some(hcfr),
        non_ma_sample_age: Some(non_ma),
        life_table: Some(life),
        cost_bands: None,
        efficacy_profile: None,
    })
}

fn season_weights(amplitude: f64) -> Vec<f64> {
    if amplitude <= 1e-9 {
        return vec![1.0 / CALENDAR_MONTHS as f64; CALENDAR_MONTHS];
    }
    let raw: Vec<f64> = (0..CALENDAR_MONTHS)
        .map(|month| {
            // Peak in December (month index 11), trough in June.
            let phase = std::f64::consts::TAU * (month as f64 - 11.0) / CALENDAR_MONTHS as f64;
            1.0 + amplitude * phase.cos()
        })
        .map(|value| value.max(0.05))
        .collect();
    let total: f64 = raw.iter().sum();
    raw.into_iter().map(|value| value / total).collect()
}

/// Discount a stream of `years` annual units starting one year from now.
///
/// The whole years are discounted as an annuity and the fractional remainder is
/// charged at the next year, which is the standard approach. The R implementation
/// (`get_discounted_life_years`, `functions/RSV_get_life_table.R`) charges the
/// fractional remainder at the *ceiling* year's full annuity, which over-counts
/// discounted life expectancy by roughly 40 % for fractional ages. That difference
/// is a deliberate correction in the rewrite and is listed in the deviations
/// section of `docs/model.md`.
pub fn discounted_life_years(years: f64, rate: f64) -> f64 {
    if !years.is_finite() || years <= 0.0 {
        return 0.0;
    }
    if rate <= 0.0 {
        return years;
    }
    if years < 1.0 {
        return years / (1.0 + rate);
    }
    let full = years.floor() as i32;
    let mut sum = 0.0;
    for year in 1..=full {
        sum += 1.0 / (1.0 + rate).powi(year);
    }
    let fraction = years - full as f64;
    sum + fraction / (1.0 + rate).powi(full + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::demo_pack;

    #[test]
    fn demo_assets_have_expected_shapes() {
        let pack = demo_pack();
        let assets = build_generated_assets(&pack).unwrap();
        assert_eq!(assets.hospital_age_calendar.unwrap().rows, AGE_MONTHS);
        assert_eq!(assets.outpatient_rate_age_sample.unwrap().cols, OUTPATIENT_SAMPLES);
        let inputs = build_inputs(&pack).unwrap();
        assert!(inputs.target_population > 0.0);
        assert_eq!(inputs.life.le.len(), AGE_MONTHS);
    }

    #[test]
    fn discounted_life_years_is_monotone() {
        assert!(discounted_life_years(10.0, 0.03) < 10.0);
        assert!(discounted_life_years(10.0, 0.0) == 10.0);
    }
}
