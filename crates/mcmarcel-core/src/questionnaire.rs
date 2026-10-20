//! The country-pack questionnaire.
//!
//! This module is the single source of truth for *what a country pack needs*:
//! every parameter, its definition, its unit, where the value comes from and how
//! its evidence has to be classified. Three consumers share it:
//!
//! * the guided wizard in the interface, which walks an analyst through the
//!   questionnaire and builds the pack;
//! * the `mcmarcel-cli questionnaire` command, which exports the same content as
//!   a standalone Markdown and CSV toolkit that can be used offline;
//! * the tests, which check that the catalogue and the engine agree.
//!
//! The catalogue is deliberately independent of any country: it describes the
//! *questions*, not the answers. The Belgian reference pack and the synthetic
//! demo are examples of completed questionnaires.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{
    CountryMeta, CountryPack, EvidenceStatus, Matrix, Parameter, RunMode, Scenario, SCHEMA_VERSION,
};

/// The kind of value a question expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionKind {
    /// A number.
    Number,
    /// A number with a unit, shown with its unit in the interface.
    Quantity,
    /// Free text.
    Text,
    /// A proportion between 0 and 1.
    Proportion,
}

impl QuestionKind {
    /// Whether the engine needs a numeric value for this question.
    pub fn is_numeric(self) -> bool {
        !matches!(self, QuestionKind::Text)
    }
}

/// A place where the value of a parameter can be found.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceSource {
    /// The citation, file or database the value comes from.
    pub citation: String,
    /// What the source provides and how it is used.
    #[serde(default)]
    pub note: String,
    /// `true` when the source is the reference implementation of the framework.
    #[serde(default)]
    pub reference_model: bool,
}

/// A section of the questionnaire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub title: String,
    /// What the section covers and why it is needed.
    pub description: String,
    /// Where the section sits in the questionnaire.
    pub order: u32,
    /// `true` when the questions are needed before the engine can run at all.
    pub required_for_run: bool,
}

/// One question of the questionnaire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    /// Parameter key, identical to the key in the country pack.
    pub key: String,
    /// Short name of the parameter.
    pub label: String,
    /// Group the question belongs to.
    pub group: String,
    /// Unit of the value.
    pub unit: String,
    /// Kind of value expected.
    pub kind: QuestionKind,
    /// What the parameter means, in plain language.
    pub definition: String,
    /// What the model does with it, and what changes if it is wrong.
    pub why_it_matters: String,
    /// How to obtain a defensible value.
    pub how_to_source: String,
    /// Sources that are known to carry this value, when they exist.
    #[serde(default)]
    pub reference_sources: Vec<ReferenceSource>,
    /// Plausible range of the value.
    #[serde(default)]
    pub range: Option<(f64, f64)>,
    /// How to classify the evidence for this parameter.
    pub evidence_guidance: String,
    /// What the parameter drives in the results.
    #[serde(default)]
    pub blocks: Vec<String>,
    /// `true` when the engine does not need the parameter.
    #[serde(default)]
    pub optional: bool,
}

impl Question {
    /// The parameter key of the standard deviation that belongs to this question.
    pub fn sd_key(&self) -> String {
        format!("{}_sd", self.key)
    }
}

/// The whole questionnaire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalogue {
    pub groups: Vec<Group>,
    pub questions: Vec<Question>,
}

impl Catalogue {
    /// The questions of a group, in catalogue order.
    pub fn questions_in_group(&self, group: &str) -> Vec<&Question> {
        self.questions.iter().filter(|q| q.group == group).collect()
    }

    /// A question by parameter key.
    pub fn question(&self, key: &str) -> Option<&Question> {
        self.questions.iter().find(|q| q.key == key)
    }

    /// The groups in questionnaire order.
    pub fn ordered_groups(&self) -> Vec<&Group> {
        let mut groups: Vec<&Group> = self.groups.iter().collect();
        groups.sort_by_key(|group| group.order);
        groups
    }

    /// How far a pack has come through the questionnaire.
    pub fn progress(&self, pack: &CountryPack) -> Progress {
        let mut answered = 0usize;
        let mut missing = 0usize;
        let mut blocking = 0usize;
        let mut by_group: BTreeMap<String, GroupProgress> = BTreeMap::new();
        for question in &self.questions {
            let entry = by_group.entry(question.group.clone()).or_insert(GroupProgress {
                answered: 0,
                total: 0,
                blocking: 0,
            });
            entry.total += 1;
            let parameter = pack.parameters.get(&question.key);
            let has_value = parameter
                .and_then(|p| p.value.as_ref())
                .map(|value| !value.is_null())
                .unwrap_or(false);
            if has_value {
                answered += 1;
                entry.answered += 1;
            } else if !question.optional {
                missing += 1;
            }
            if parameter
                .map(|p| p.status.blocks_production())
                .unwrap_or(true)
                && !question.optional
            {
                blocking += 1;
                entry.blocking += 1;
            }
        }
        Progress { answered, total: self.questions.len(), missing, blocking, by_group }
    }

    /// The questionnaire as a Markdown document, for offline use.
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# McMarcel Studio — country pack questionnaire\n\n");
        out.push_str(
            "This questionnaire lists every parameter a McMarcel Studio country pack needs, with the definition, \
             the unit, where a defensible value comes from and how its evidence has to be classified.\n\n",
        );
        out.push_str(
            "Fill one row per parameter. A parameter is only usable for a production run when it carries a value, \
             an evidence status and a source. `MISSING` and `SYNTHETIC` block production mode; the engine stamps \
             every run made with them.\n\n",
        );
        out.push_str("## Evidence statuses\n\n");
        out.push_str("| Status | Meaning | Blocks production |\n|---|---|---|\n");
        for (status, meaning) in [
            (EvidenceStatus::Ok, "direct, reviewed country evidence with a citation"),
            (EvidenceStatus::Derived, "calculated from documented country inputs; the derivation is recorded"),
            (EvidenceStatus::Transfer, "borrowed from another setting and explicitly flagged"),
            (EvidenceStatus::Convention, "a modelling choice rather than an empirical estimate"),
            (EvidenceStatus::Missing, "no defensible value available"),
            (EvidenceStatus::Synthetic, "demo or fixture data"),
        ] {
            out.push_str(&format!(
                "| `{}` | {} | {} |\n",
                status.as_str(),
                meaning,
                if status.blocks_production() { "yes" } else { "no" }
            ));
        }
        out.push('\n');
        for group in self.ordered_groups() {
            out.push_str(&format!("## {}. {}\n\n", group.order, group.title));
            out.push_str(&format!("{}\n\n", group.description));
            for question in self.questions_in_group(&group.id) {
                out.push_str(&format!("### {} — {}\n\n", question.key, question.label));
                out.push_str(&format!("- **Unit:** {}\n", question.unit));
                out.push_str(&format!("- **Definition:** {}\n", question.definition));
                out.push_str(&format!("- **Why it matters:** {}\n", question.why_it_matters));
                out.push_str(&format!("- **How to source it:** {}\n", question.how_to_source));
                if let Some((min, max)) = question.range {
                    out.push_str(&format!("- **Range:** {min} to {max}\n"));
                }
                out.push_str(&format!("- **Evidence:** {}\n", question.evidence_guidance));
                if !question.blocks.is_empty() {
                    out.push_str(&format!("- **Drives:** {}\n", question.blocks.join(", ")));
                }
                if !question.reference_sources.is_empty() {
                    out.push_str("- **Known sources:**\n");
                    for source in &question.reference_sources {
                        out.push_str(&format!("  - {} — {}\n", source.citation, source.note));
                    }
                }
                out.push_str("\n| Value | Evidence status | Source | Reviewer note |\n|---|---|---|---|\n");
                out.push_str("|  |  |  |  |\n\n");
            }
        }
        out
    }

    /// The questionnaire as CSV, one row per question.
    pub fn to_csv(&self) -> String {
        let mut out = String::new();
        out.push_str("group,order,key,label,unit,kind,definition,why_it_matters,how_to_source,range_min,range_max,evidence_guidance,drives,known_sources,value,evidence_status,source,reviewer_note\n");
        for group in self.ordered_groups() {
            for question in self.questions_in_group(&group.id) {
                let (min, max) = question.range.unwrap_or((f64::NAN, f64::NAN));
                let sources = question
                    .reference_sources
                    .iter()
                    .map(|s| s.citation.replace('\n', " ").replace(',', ";"))
                    .collect::<Vec<_>>()
                    .join(" | ");
                let field = |value: &str| value.replace('\n', " ").replace('"', "'");
                out.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},,,,,,\n",
                    group.id,
                    group.order,
                    field(&question.key),
                    field(&question.label),
                    field(&question.unit),
                    question.kind_name(),
                    field(&question.definition),
                    field(&question.why_it_matters),
                    field(&question.how_to_source),
                    if min.is_nan() { String::new() } else { format!("{min}") },
                    if max.is_nan() { String::new() } else { format!("{max}") },
                    field(&question.evidence_guidance),
                    field(&question.blocks.join("; ")),
                    field(&sources),
                ));
            }
        }
        out
    }
}

impl Question {
    fn kind_name(&self) -> &'static str {
        match self.kind {
            QuestionKind::Number => "number",
            QuestionKind::Quantity => "quantity",
            QuestionKind::Text => "text",
            QuestionKind::Proportion => "proportion",
        }
    }
}

/// How far a pack has come through the questionnaire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub answered: usize,
    pub total: usize,
    pub missing: usize,
    pub blocking: usize,
    pub by_group: BTreeMap<String, GroupProgress>,
}

/// Progress within one group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupProgress {
    pub answered: usize,
    pub total: usize,
    pub blocking: usize,
}

/// The identity of the country a questionnaire is filled out for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryIdentity {
    pub name: String,
    pub iso3: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default = "default_price_year")]
    pub price_year: i32,
    #[serde(default = "default_perspective")]
    pub perspective: String,
    #[serde(default)]
    pub region: String,
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default)]
    pub notes: String,
}

fn default_price_year() -> i32 {
    2024
}

fn default_perspective() -> String {
    "healthcare".into()
}

fn default_locale() -> String {
    "en".into()
}

/// One answered question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    /// The value, when it is known.
    #[serde(default)]
    pub value: Option<Value>,
    /// The evidence status of the value.
    #[serde(default)]
    pub status: EvidenceStatus,
    /// Where the value comes from.
    #[serde(default)]
    pub source: Option<String>,
    /// A reviewer note.
    #[serde(default)]
    pub notes: Option<String>,
}

/// A completed questionnaire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub country: CountryIdentity,
    #[serde(default)]
    pub answers: BTreeMap<String, Answer>,
}

impl Submission {
    /// Build the country pack described by the questionnaire.
    pub fn to_pack(&self) -> Result<CountryPack, crate::Error> {
        let catalogue = catalogue();
        let name = self.country.name.trim();
        let iso3 = self.country.iso3.trim().to_ascii_uppercase();
        if name.is_empty() || iso3.len() != 3 {
            return Err(crate::Error::Validation(
                "the questionnaire needs a country name and a three-letter ISO3 code".into(),
            ));
        }
        let id = format!("{}-{}", iso3.to_ascii_lowercase(), crate_now_ms());
        let mut parameters: BTreeMap<String, Parameter> = BTreeMap::new();
        for question in &catalogue.questions {
            let answer = self.answers.get(&question.key);
            let (value, status, source, notes) = match answer {
                Some(answer) => (
                    answer.value.clone().filter(|value| !value.is_null()),
                    answer.status,
                    answer.source.clone().filter(|source| !source.trim().is_empty()),
                    answer.notes.clone().filter(|note| !note.trim().is_empty()),
                ),
                None => (None, EvidenceStatus::Missing, None, None),
            };
            let mut parameter = Parameter {
                key: question.key.clone(),
                label: question.label.clone(),
                unit: question.unit.clone(),
                value,
                low: None,
                high: None,
                status,
                source,
                source_ref: None,
                url: None,
                notes,
                blocks_outputs: question.blocks.clone(),
            };
            if question.kind.is_numeric() {
                parameter.value = parameter.value.filter(|value| value.is_number());
            } else {
                parameter.value = parameter.value.filter(|value| value.is_string());
            }
            parameters.insert(question.key.clone(), parameter);
        }
        let country = CountryMeta {
            id,
            name: name.to_string(),
            iso3,
            currency: self.country.currency.trim().to_string(),
            price_year: self.country.price_year,
            locale: self.country.locale.trim().to_string(),
            perspective: self.country.perspective.trim().to_string(),
            region: self.country.region.trim().to_string(),
            notes: self.country.notes.trim().to_string(),
        };
        Ok(CountryPack {
            schema_version: SCHEMA_VERSION,
            country,
            parameters,
            assets: Default::default(),
            scenarios: vec![Scenario::reference()],
            notes: "Created with the McMarcel Studio country pack questionnaire.".into(),
        })
    }
}

fn crate_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

/// A blank parameter matrix, used when a pack has to carry empty assets.
pub fn empty_matrix(rows: usize, cols: usize) -> Matrix {
    Matrix { rows, cols, data: vec![0.0; rows * cols] }
}

/// The mode a freshly completed questionnaire starts in.
pub fn initial_mode(pack: &CountryPack) -> RunMode {
    if pack.parameters.values().any(|p| p.status.blocks_production()) {
        RunMode::Fixture
    } else {
        RunMode::Production
    }
}

/// The questionnaire.
pub fn catalogue() -> Catalogue {
    let mut questions: Vec<Question> = Vec::new();

    questions.extend(demography_questions());
    questions.extend(epidemiology_questions());
    questions.extend(cost_questions());
    questions.extend(utility_questions());
    questions.extend(product_questions("maternal", "Maternal RSV vaccine", "maternal"));
    questions.extend(product_questions("mab", "Monoclonal antibody", "mab"));
    questions.extend(analysis_questions());

    Catalogue {
        groups: vec![
            Group {
                id: "demography".into(),
                title: "Demography".into(),
                description: "The size and shape of the birth cohort the model follows from birth to five years.".into(),
                order: 1,
                required_for_run: true,
            },
            Group {
                id: "epidemiology".into(),
                title: "Epidemiology".into(),
                description: "How often RSV occurs in the first year of life, how severe it is and how it is distributed over the calendar year.".into(),
                order: 2,
                required_for_run: true,
            },
            Group {
                id: "costs".into(),
                title: "Health system costs".into(),
                description: "What one case of each type costs the health system, in the currency and the price year of the analysis.".into(),
                order: 3,
                required_for_run: true,
            },
            Group {
                id: "utilities".into(),
                title: "QALY losses".into(),
                description: "The health-related quality of life lost per case, used to convert cases into QALYs.".into(),
                order: 4,
                required_for_run: true,
            },
            Group {
                id: "maternal".into(),
                title: "Maternal RSV vaccine".into(),
                description: "Efficacy, protection duration, coverage, price and delivery costs of the maternal vaccine.".into(),
                order: 5,
                required_for_run: true,
            },
            Group {
                id: "mab".into(),
                title: "Monoclonal antibody".into(),
                description: "Efficacy, protection duration, coverage, price and delivery costs of the monoclonal antibody.".into(),
                order: 6,
                required_for_run: true,
            },
            Group {
                id: "analysis".into(),
                title: "Analysis conventions".into(),
                description: "Perspective, discounting and the willingness-to-pay thresholds used for the decision analysis.".into(),
                order: 7,
                required_for_run: false,
            },
        ],
        questions,
    }
}

fn question(
    key: &str,
    label: &str,
    group: &str,
    unit: &str,
    kind: QuestionKind,
    definition: &str,
    why_it_matters: &str,
    how_to_source: &str,
    range: Option<(f64, f64)>,
    evidence: &str,
    blocks: &[&str],
    optional: bool,
) -> Question {
    Question {
        key: key.into(),
        label: label.into(),
        group: group.into(),
        unit: unit.into(),
        kind,
        definition: definition.into(),
        why_it_matters: why_it_matters.into(),
        how_to_source: how_to_source.into(),
        reference_sources: Vec::new(),
        range,
        evidence_guidance: evidence.into(),
        blocks: blocks.iter().map(|item| item.to_string()).collect(),
        optional,
    }
}

fn with_sources(mut question: Question, sources: &[(&str, &str)]) -> Question {
    question.reference_sources = sources
        .iter()
        .map(|(citation, note)| ReferenceSource {
            citation: (*citation).to_string(),
            note: (*note).to_string(),
            reference_model: true,
        })
        .collect();
    question
}

fn demography_questions() -> Vec<Question> {
    vec![
        with_sources(
            question(
                "births",
                "Annual births",
                "demography",
                "births per year",
                QuestionKind::Quantity,
                "The number of live births in the country in the price year of the analysis. The model follows a single birth cohort of this size from birth to 60 months of age.",
                "Every burden estimate scales with it: cases, admissions, deaths, costs and QALYs are all proportional to the cohort size.",
                "National statistics office (birth registrations) or the UN World Population Prospects; a World Bank crude birth rate applied to the population of the same year also works, but say so in the source.",
                Some((1.0, 100_000_000.0)),
                "Direct national count. Use `ok` with the statistics-office citation; `transfer` if a regional estimate stands in.",
                &["cases", "costs", "qalys", "doses"],
                false,
            ),
            &[(
                "input/20250106_BE_country_details.csv (target_population, BEL 2024)",
                "The reference implementation reads the cohort size from a country-details file.",
            )],
        ),
        with_sources(
            question(
                "stillbirth_rate",
                "Stillbirth rate",
                "demography",
                "proportion",
                QuestionKind::Proportion,
                "The share of pregnancies that end in a stillbirth. The cohort that enters the model is births × (1 − stillbirth rate).",
                "A stillbirth rate that is too low inflates every burden estimate. It also changes the number of doses the maternal programme has to deliver, because the vaccine is given during pregnancy.",
                "National perinatal registry or vital statistics; WHO and UNICEF country estimates are an acceptable transfer.",
                Some((0.0, 0.5)),
                "National registry: `ok`. Regional estimate: `transfer`.",
                &["cases", "costs", "maternal doses"],
                false,
            ),
            &[(
                "input/20250106_BE_country_details.csv (stillbirth_rate)",
                "The reference implementation takes it from the same country-details file as the births.",
            )],
        ),
        with_sources(
            question(
                "pre_term_rate",
                "Pre-term birth rate",
                "demography",
                "proportion",
                QuestionKind::Proportion,
                "The share of live births before 37 weeks of gestation. The maternal vaccine is given to the mother, so a pre-term infant is effectively unprotected: maternal coverage is scaled by (1 − pre-term rate).",
                "Too low a value overstates the benefit of the maternal vaccine; too high a value understates it.",
                "National perinatal registry; WHO/UNICEF estimates; the McMarcel configuration reads it as the incomplete maternal transfer rate.",
                Some((0.0, 0.5)),
                "National registry: `ok`. Regional estimate: `transfer`.",
                &["maternal efficacy"],
                false,
            ),
            &[(
                "input/20250106_BE_country_details.csv (incomplete_maternal_transfer_rate)",
                "The reference implementation reads the pre-term rate from the country-details file.",
            )],
        ),
        with_sources(
            question(
                "life_expectancy_years",
                "Life expectancy at birth",
                "demography",
                "years",
                QuestionKind::Quantity,
                "Life expectancy at birth, used only when the pack does not carry a life table asset and the engine has to generate one.",
                "It drives the years of life lost by an RSV death and therefore the QALY loss of deaths.",
                "National life table, or the UN World Population Prospects for the same year and country.",
                Some((30.0, 100.0)),
                "Official life table: `ok`. UN estimate: `transfer`.",
                &["qalys"],
                true,
            ),
            &[],
        ),
        with_sources(
            question(
                "qol_mean",
                "Mean population utility",
                "demography",
                "utility",
                QuestionKind::Quantity,
                "The average health-related quality of life of the general population. It is used to turn life expectancy into health-related QALYs when the life table has to be generated.",
                "Deaths are converted into QALYs with the life table, so this value scales the mortality-related part of the QALY loss.",
                "A national or regional EQ-5D / EQ-VAS population norm set (the reference implementation uses the Belgian 2018 EQ-5D norms).",
                Some((0.5, 1.0)),
                "Population norms: `ok` with the survey citation. Norms from another country: `transfer`.",
                &["qalys"],
                true,
            ),
            &[(
                "input/eq5d/popnormINDEX.rds",
                "EQ-5D Belgian general population norms (BE, 2018) used by the reference implementation.",
            )],
        ),
    ]
}

fn epidemiology_questions() -> Vec<Question> {
    vec![
        with_sources(
            question(
                "incidence_hospital_0_5m",
                "Hospital incidence, 0–5 months",
                "epidemiology",
                "cases per infant",
                QuestionKind::Quantity,
                "The number of hospital admissions for RSV per infant in the first five months of life. The engine scales it to the cohort and distributes it over the calendar months with the seasonality profile.",
                "It is the single largest driver of the burden: every downstream outcome (deaths, costs, QALYs) is proportional to it.",
                "National hospital discharge data (ICD-10 J21 or the local RSV code) per age in months, for at least two seasons so that a seasonal profile can be derived.",
                Some((0.0, 0.5)),
                "National discharge data: `ok` with the extraction described. Adjusted or modelled rates: `derived` with the derivation recorded.",
                &["cases", "deaths", "costs", "qalys"],
                false,
            ),
            &[(
                "input/BELX_totalnrhospadmissions_byage_bycalendarmonth_perc_S2018_2019_4seasons.csv",
                "The reference implementation reads a 60 × 12 age-by-calendar-month admission matrix and scales it to the cohort.",
            )],
        ),
        with_sources(
            question(
                "incidence_hospital_6_11m",
                "Hospital incidence, 6–11 months",
                "epidemiology",
                "cases per infant",
                QuestionKind::Quantity,
                "The number of hospital admissions for RSV per infant between six and eleven months of age.",
                "The second RSV season is much smaller than the first, so this value determines how much of the burden sits outside the first year — where catch-up programmes and the protection duration matter most.",
                "Same source as the 0–5 month incidence, split by age in months.",
                Some((0.0, 0.5)),
                "National discharge data: `ok`. Modelled: `derived`.",
                &["cases", "deaths", "costs", "qalys"],
                false,
            ),
            &[],
        ),
        with_sources(
            question(
                "ratio_outpatient_hospital",
                "Outpatient to hospital ratio",
                "epidemiology",
                "ratio",
                QuestionKind::Quantity,
                "The number of outpatient consultations per hospital admission. The engine converts the hospital admission profile into outpatient cases and spreads them over the calendar months with the hospital seasonality.",
                "Outpatient cases dominate the number of cases and carry a unit cost and a QALY loss each, so the ratio drives most of the total cost.",
                "Ambulatory (outpatient) data for the same population and period, divided by the admission count of the same period.",
                Some((0.0, 100.0)),
                "National ambulatory data: `ok`. Ratio borrowed from another country: `transfer`.",
                &["cases", "costs", "qalys"],
                false,
            ),
            &[(
                "input/ESPX_outpatient_incidence_1000_infant_months.csv",
                "The reference implementation uses outpatient incidence per 1,000 infant-months, with epidemiological samples for the uncertainty.",
            )],
        ),
        with_sources(
            question(
                "proportion_non_medical_attended",
                "Non-medically-attended proportion",
                "epidemiology",
                "proportion",
                QuestionKind::Proportion,
                "The share of medically attended RSV cases that never reach a clinician (pharmacy advice, telephone advice, no care). The engine adds these cases on top of the attended ones.",
                "They add cases and costs but are invisible in hospital and outpatient data, so they are often underestimated.",
                "Household surveys, retail pharmacy sales data or sentinel primary-care data for the same period.",
                Some((0.0, 1.0)),
                "National survey: `ok`. Regional pharmacy data: `transfer`.",
                &["cases", "costs", "qalys"],
                false,
            ),
            &[(
                "input/RESCEUX_nonMA_cases_0-59m.csv",
                "The reference implementation reads a sample of non-medically-attended proportions by age month.",
            )],
        ),
        with_sources(
            question(
                "icu_share_of_hospital",
                "ICU share of hospital admissions",
                "epidemiology",
                "proportion",
                QuestionKind::Proportion,
                "The number of ICU admissions per hospital admission. Only used when the pack does not carry an ICU matrix.",
                "ICU cases carry the highest unit cost and the highest case-fatality risk, so this value moves the cost and the death estimates.",
                "National ICU registry or hospital discharge data with an intensive-care flag.",
                Some((0.0, 1.0)),
                "National registry: `ok`. Literature estimate: `transfer`.",
                &["deaths", "costs"],
                false,
            ),
            &[(
                "input/BELX_totalnrICUadmissions_byage_bycalendarmonth_perc_S2018_2019_4seasons.csv",
                "The reference implementation reads a separate 60 × 12 ICU matrix.",
            )],
        ),
        with_sources(
            question(
                "cfr_hospital",
                "Hospital case-fatality risk",
                "epidemiology",
                "proportion",
                QuestionKind::Proportion,
                "The probability that a child hospitalised with RSV dies, by age in months. The engine multiplies the admissions with this risk to obtain deaths.",
                "Deaths drive the life-years and health-related QALYs lost, so this value has an outsized effect on the QALY loss and on every cost-effectiveness ratio.",
                "National hospital mortality data by age (discharge diagnosis RSV), or the published case-fatality series used by the framework.",
                Some((0.0, 1.0)),
                "National mortality data: `ok`. Published series from another country: `transfer`.",
                &["deaths", "qalys"],
                false,
            ),
            &[(
                "input/BELX_hCFR_byAge.csv",
                "The reference implementation reads the age-specific hospital and ICU case-fatality risks from one file.",
            )],
        ),
        with_sources(
            question(
                "cfr_icu",
                "ICU case-fatality risk",
                "epidemiology",
                "proportion",
                QuestionKind::Proportion,
                "The probability that a child admitted to intensive care with RSV dies, by age in months.",
                "ICU deaths are the most expensive and the most damaging in QALY terms, so this value can dominate the mortality estimate.",
                "National ICU registry or hospital mortality data by age.",
                Some((0.0, 1.0)),
                "National registry: `ok`. Published series: `transfer`.",
                &["deaths", "qalys"],
                false,
            ),
            &[(
                "input/BELX_hCFR_byAge.csv",
                "Same file as the hospital case-fatality risk.",
            )],
        ),
        question(
            "age_decay_hospital",
            "Hospital age decay",
            "epidemiology",
            "per month",
            QuestionKind::Quantity,
            "The monthly factor by which the hospital admission rate falls with age. A factor of 1 means the rate does not decay; 0.97 means three per cent less each month.",
            "The engine uses it to build an age profile when no admission matrix is supplied, so it shapes the whole burden curve.",
            "Fit the decay to the national age profile of RSV admissions, or set it to 1 when a full age-by-month matrix is supplied.",
            Some((0.5, 1.0)),
            "Fitted to national data: `derived`, with the fit described. A default: `convention`.",
            &["cases", "deaths", "costs"],
            true,
        ),
        question(
            "age_decay_outpatient",
            "Outpatient age decay",
            "epidemiology",
            "per month",
            QuestionKind::Quantity,
            "The monthly factor by which the outpatient rate falls with age, used when no outpatient matrix is supplied.",
            "Together with the hospital profile it sets the shape of the outpatient burden, which is the largest cost component.",
            "Fit to the national age profile of outpatient consultations, or set it to 1 when a matrix is supplied.",
            Some((0.5, 1.0)),
            "Fitted to national data: `derived`. A default: `convention`.",
            &["cases", "costs"],
            true,
        ),
        question(
            "age_decay_cfr",
            "Case-fatality age decay",
            "epidemiology",
            "per month",
            QuestionKind::Quantity,
            "The monthly factor by which the case-fatality risk falls with age, used when no case-fatality profile is supplied.",
            "It changes how the deaths are distributed over the first year of life.",
            "Fit to the national age profile of RSV deaths, or set it to 1 when an age profile is supplied.",
            Some((0.5, 1.0)),
            "Fitted to national data: `derived`. A default: `convention`.",
            &["deaths"],
            true,
        ),
        question(
            "seasonality_amplitude",
            "Seasonality amplitude",
            "epidemiology",
            "0 to 0.8",
            QuestionKind::Quantity,
            "The strength of the calendar-month seasonality of RSV admissions. Zero is a flat year, 0.8 is a very peaked year with a December maximum.",
            "Seasonality decides which birth cohorts are inside a seasonal programme window, and therefore how much a catch-up programme can add.",
            "Derive it from the national age-by-calendar-month admission matrix (or set it to 0 and supply the matrix itself).",
            Some((0.0, 0.8)),
            "Derived from national data: `derived`. A default: `convention`.",
            &["cases", "programme coverage"],
            true,
        ),
    ]
}

fn cost_questions() -> Vec<Question> {
    let mut questions = Vec::new();
    for (key, label, definition, why, source, blocks) in [
        (
            "cost_outpatient",
            "Outpatient cost per case",
            "The cost of one outpatient RSV consultation, in the currency of the analysis and in the price year.",
            "Outpatient cases are the most frequent, so this value contributes a large part of the medical cost that prevention can avert.",
            "Insurance claims, national tariff or costing study for the price year; convert with the price-year index and state the index used.",
            &["medical cost", "incremental cost"][..],
        ),
        (
            "cost_hospital",
            "Hospital cost per admission",
            "The cost of one hospital admission for RSV, excluding intensive care, in the currency of the analysis and the price year.",
            "Hospital cost is the largest cost per case and the value that the cost-effectiveness result is most sensitive to.",
            "Insurance claims or a national tariff; state whether the cost includes the admission and the ward stay only.",
            &["medical cost", "incremental cost"][..],
        ),
        (
            "cost_icu",
            "ICU cost per admission",
            "The cost of one intensive-care admission for RSV, in the currency of the analysis and the price year.",
            "ICU cases are few but expensive; they dominate the tail of the cost distribution.",
            "Insurance claims or a national tariff for intensive care.",
            &["medical cost", "incremental cost"][..],
        ),
        (
            "cost_non_ma",
            "Non-medically-attended cost per case",
            "The cost of managing one case that never reaches a clinician, in the currency of the analysis and the price year.",
            "Small per case, but these cases are frequent, so the total is not negligible.",
            "Pharmacy costs, telephone consultation tariffs or a costing study of the non-attended pathway.",
            &["medical cost"][..],
        ),
        (
            "cost_wheezing",
            "Wheezing sequelae cost",
            "The cost per child-year of recurrent wheezing and asthma after an RSV hospital admission.",
            "The recurrent-wheezing module is not implemented in this engine version, so this value is not applied to the results.",
            "If you have a value, keep it and document that the engine cannot apply it yet; otherwise set it to zero and record the omission.",
            &[],
        ),
    ] {
        questions.push(with_sources(
            question(
                key,
                label,
                "costs",
                "local currency per case",
                QuestionKind::Quantity,
                definition,
                why,
                source,
                Some((0.0, 1_000_000.0)),
                "National tariff or claims for the price year: `ok` with the citation. Cost from another country: `transfer`. An international reference price: `transfer` with the country named.",
                blocks,
                key == "cost_wheezing",
            ),
            &[(
                "input/BELX_cost_perspectives_regions.csv (payer perspective)",
                "The reference implementation reads the unit costs per perspective and region, with a mean and a confidence interval or standard deviation.",
            )],
        ));
        questions.push(question(
            &format!("{key}_sd"),
            &format!("{label}, standard deviation"),
            "costs",
            "local currency per case",
            QuestionKind::Quantity,
            "The standard deviation of the sampled unit cost. The engine samples the cost from a gamma distribution with this mean and standard deviation.",
            "A cost without uncertainty understates the width of the cost-effectiveness plane and can hide a strategy that is not robust.",
            "Derive it from the published confidence interval of the cost as (upper − lower) / (2 × 1.96), or from the variability of the claims.",
            Some((0.0, 1_000_000.0)),
            "Derived from a published interval: `derived`, with the interval cited. A zero standard deviation is a convention: say so.",
            blocks,
            key == "cost_wheezing",
        ));
    }
    questions
}

fn utility_questions() -> Vec<Question> {
    let mut questions = Vec::new();
    for (key, label, definition, why, source) in [
        (
            "qaly_loss_outpatient",
            "QALY loss, outpatient",
            "The health-related quality of life lost by one outpatient RSV case, in QALYs per case.",
            "Outpatient cases are the most frequent, so this value determines most of the non-fatal QALY loss.",
            "Published utility decrements for acute respiratory infection in children, adapted to the local setting, or an EQ-5D study of RSV cases.",
        ),
        (
            "qaly_loss_hospital",
            "QALY loss, hospital",
            "The health-related quality of life lost by one hospitalised RSV case, in QALYs per case. The same value is applied to ICU cases.",
            "It converts the most severe and most expensive cases into QALYs.",
            "Published utility decrements for hospitalised RSV in infants, or a local EQ-5D follow-up of hospitalised cases.",
        ),
        (
            "qaly_loss_non_ma",
            "QALY loss, non-medically-attended",
            "The health-related quality of life lost by one medically unattended RSV case, in QALYs per case.",
            "These cases are frequent, so a plausible but wrong value can dominate the QALY loss.",
            "Published decrements for ambulatory respiratory infection, or a survey of families of children with RSV.",
        ),
        (
            "qaly_loss_wheezing",
            "QALY loss, wheezing",
            "The health-related quality of life lost per child-year of recurrent wheezing and asthma after an RSV admission.",
            "The recurrent-wheezing module is not implemented in this engine version, so this value is not applied to the results.",
            "If you have a value, keep it and document that the engine cannot apply it yet; otherwise set it to zero and record the omission.",
        ),
    ] {
        questions.push(with_sources(
            question(
                key,
                label,
                "utilities",
                "QALY per case",
                QuestionKind::Quantity,
                definition,
                why,
                source,
                Some((0.0, 1.0)),
                "Published decrements for the same country: `ok` with the citation. Decrement from another country: `transfer`, with the adaptation described.",
                &["qalys"],
                key == "qaly_loss_wheezing",
            ),
            &[(
                "functions/RSV_config.R (QALYloss_hosp_mean, QALYloss_outpatient_mean, QALY_non_ma_mean)",
                "The reference implementation hard-codes the Belgian QALY losses and derives their standard error from the published confidence interval.",
            )],
        ));
        questions.push(question(
            &format!("{key}_sd"),
            &format!("{label}, standard deviation"),
            "utilities",
            "QALY per case",
            QuestionKind::Quantity,
            "The standard deviation of the sampled utility loss, derived from the published confidence interval.",
            "It controls how wide the QALY uncertainty is and therefore how often a strategy changes rank.",
            "Derive it from the published confidence interval as (upper − lower) / (2 × 1.96).",
            Some((0.0, 1.0)),
            "Derived from a published interval: `derived`, with the interval cited.",
            &["qalys"],
            key == "qaly_loss_wheezing",
        ));
    }
    questions
}

fn product_questions(group: &str, product: &str, prefix: &str) -> Vec<Question> {
    let mut questions = Vec::new();
    let (file_ma, file_hosp, file_severe) = if group == "maternal" {
        (
            "input/VE_maternal_CI_MA.csv",
            "input/VE_maternal_CI_Hosp_MA.csv",
            "input/VE_maternal_CI_Seve_MA.csv",
        )
    } else {
        (
            "input/VE_mab_CI_MA.csv",
            "input/VE_mab_CI_Hosp_MA.csv",
            "input/VE_mab_CI_Seve_MA.csv",
        )
    };
    for (outcome, label, definition, why, source) in [
        (
            "hospital",
            "hospitalisation",
            "The efficacy of the product against RSV hospitalisation.",
            "Hospital admissions are the most expensive outcome, so this value drives the cost-effectiveness of the programme.",
            "The pivotal trial or its pooled estimate for the same age group and outcome, with its confidence interval.",
        ),
        (
            "outpatient",
            "outpatient (medically attended) infection",
            "The efficacy of the product against medically attended RSV infection of any severity.",
            "Most cases are outpatient, so this value determines the number of cases the programme averts.",
            "The pivotal trial estimate for the same outcome and age group, with its confidence interval.",
        ),
        (
            "icu",
            "intensive-care admission",
            "The efficacy of the product against RSV intensive-care admission.",
            "ICU cases drive the mortality and the tail of the cost distribution.",
            "The pivotal trial or its severe-outcome estimate for the same age group, with its confidence interval.",
        ),
        (
            "death",
            "death",
            "The efficacy of the product against RSV death. The reference implementation sets this equal to the intensive-care efficacy.",
            "Deaths are the outcome with the largest quality-of-life consequence, so this value matters even though it is not observed directly in most trials.",
            "The trial estimate for the severe composite, or the intensive-care efficacy with the reasoning recorded.",
        ),
        (
            "non_ma",
            "medically unattended infection",
            "The efficacy of the product against RSV infection that never reaches a clinician.",
            "These cases are frequent; an efficacy of zero here means the programme only prevents visible disease.",
            "The trial estimate for the same outcome, or the all-cause efficacy with the reasoning recorded.",
        ),
    ] {
        let key = format!("ve_{prefix}_{outcome}");
        questions.push(with_sources(
            question(
                &key,
                &format!("{product} efficacy, {label}"),
                group,
                "proportion",
                QuestionKind::Proportion,
                definition,
                why,
                source,
                Some((0.0, 1.0)),
                "Trial estimate for the same country or a multi-country trial including it: `ok` with the citation. Estimate from another setting: `transfer`, with the population named.",
                &["averted cases", "averted costs", "averted qalys"],
                false,
            ),
            &[
                (file_ma, "Efficacy against medically attended infection, by age, with bootstrap samples."),
                (file_hosp, "Efficacy against hospitalisation, by age, with bootstrap samples."),
                (file_severe, "Efficacy against the severe composite, by age, with bootstrap samples."),
            ],
        ));
        questions.push(question(
            &format!("{key}_sd"),
            &format!("{product} efficacy, {label}: standard deviation"),
            group,
            "proportion",
            QuestionKind::Quantity,
            "The standard deviation of the sampled efficacy, derived from the confidence interval of the trial estimate.",
            "It decides how often the programme misses its expected effect, and therefore how often the cost-effectiveness result changes.",
            "Derive it from the published confidence interval as (upper − lower) / (2 × 1.96), or from the bootstrap distribution in the efficacy file.",
            Some((0.0, 1.0)),
            "Derived from a published interval: `derived`, with the interval cited.",
            &["averted cases", "averted costs", "averted qalys"],
            false,
        ));
    }
    questions.push(question(
        &format!("duration_{prefix}_years"),
        &format!("{product} protection duration"),
        group,
        "years",
        QuestionKind::Quantity,
        "How long the protection lasts after the dose is given. The engine applies the efficacy for that many months of age.",
        "A duration that is too long credits the programme with protection it does not give, and a duration that is too short understates it.",
        "The trial follow-up that supports the duration, or the regulatory label. State whether the protection wanes over time; if it does, supply an age-specific efficacy profile instead.",
        Some((0.0, 5.0)),
        "Trial follow-up: `ok` with the citation. Regulatory label: `ok`.",
        &["averted cases", "averted costs"],
        false,
    ));
    questions.push(question(
        &format!("coverage_{prefix}"),
        &format!("{product} coverage"),
        group,
        "proportion",
        QuestionKind::Proportion,
        "The share of the eligible cohort that receives the product. Every scenario can override it; this value is the default.",
        "Coverage scales the whole benefit linearly, so it is the assumption an analyst is most often asked to defend.",
        "The coverage assumed in the study protocol, the expected uptake in the first year, or the national immunisation coverage for a comparable programme.",
        Some((0.0, 1.0)),
        "Planned programme coverage from the study protocol: `ok`. Observed coverage from another country: `transfer`.",
        &["averted cases", "averted costs", "doses"],
        false,
    ));
    questions.push(question(
        &format!("price_{prefix}_min"),
        &format!("{product} price, low"),
        group,
        "local currency per dose",
        QuestionKind::Quantity,
        "The lower bound of the dose price. When the two price bounds are equal, the price is deterministic.",
        "The price drives the intervention cost, which is the dominant term in the incremental cost, and it drives the price threshold P*.",
        "The confidential price from the manufacturer, the price used in the study protocol, or a published reference price. Always record the price year and whether the price is confidential.",
        Some((0.0, 100_000.0)),
        "Study price: `ok` with the citation. List price: `ok`. Reference price from another country: `transfer`.",
        &["intervention cost", "icer", "pstar"],
        false,
    ));
    questions.push(question(
        &format!("price_{prefix}_max"),
        &format!("{product} price, high"),
        group,
        "local currency per dose",
        QuestionKind::Quantity,
        "The upper bound of the dose price, sampled uniformly between the low and the high bound.",
        "A price range widens the cost-effectiveness plane and shows how much the conclusion depends on the price.",
        "The same source as the low bound; a scenario analysis range is a legitimate choice if it is documented as such.",
        Some((0.0, 100_000.0)),
        "Same status as the low bound.",
        &["intervention cost", "icer", "pstar"],
        false,
    ));
    questions.push(question(
        &format!("admin_cost_{prefix}"),
        &format!("{product} administration cost"),
        group,
        "local currency per dose",
        QuestionKind::Quantity,
        "The cost of delivering one dose: the health-care professional time and the consumables, per dose.",
        "It is added to the price of every dose, so it matters most for expensive products with small cohorts.",
        "The programme budget or the costing study of the delivery pathway.",
        Some((0.0, 100_000.0)),
        "Programme budget: `ok`. Unit-costing study: `ok`.",
        &["intervention cost"],
        false,
    ));
    questions.push(question(
        &format!("implementation_cost_{prefix}"),
        &format!("{product} implementation cost"),
        group,
        "local currency",
        QuestionKind::Quantity,
        "The fixed cost of setting the programme up (training, information, cold chain), divided over the doses of the cohort.",
        "It is a fixed cost spread over one cohort, so it matters most for small cohorts and for expensive products.",
        "The programme budget for the first year.",
        Some((0.0, 1_000_000_000.0)),
        "Programme budget: `ok`.",
        &["intervention cost"],
        false,
    ));
    questions.push(question(
        &format!("wastage_{prefix}"),
        &format!("{product} wastage"),
        group,
        "proportion",
        QuestionKind::Proportion,
        "The share of doses that are lost: multi-dose vials, missed appointments, wastage at the end of the campaign.",
        "Wastage raises the number of doses that have to be bought, and therefore the intervention cost.",
        "The programme's own wastage rate, or the wastage rate of a comparable immunisation programme in the same health system.",
        Some((0.0, 1.0)),
        "Programme data: `ok`. Comparable programme: `transfer`.",
        &["intervention cost", "doses"],
        false,
    ));
    questions
}

fn analysis_questions() -> Vec<Question> {
    vec![
        question(
            "discount_rate_cost",
            "Cost discount rate",
            "analysis",
            "per year",
            QuestionKind::Quantity,
            "The annual rate at which future costs are discounted.",
            "Discounting lowers the value of a prevention programme whose costs are now and whose benefits are later, or the other way round when the intervention is cheap and durable.",
            "The national guideline on health-economic evaluation; the reference implementation uses 3 %.",
            Some((0.0, 0.15)),
            "National guideline: `ok` with the citation. A convention: `convention`.",
            &["costs", "icer"],
            false,
        ),
        question(
            "discount_rate_effect",
            "Effect discount rate",
            "analysis",
            "per year",
            QuestionKind::Quantity,
            "The annual rate at which future health effects are discounted.",
            "It changes the weight of the deaths averted in the first months of life against the cases averted later.",
            "The national guideline; the reference implementation uses 1.5 %.",
            Some((0.0, 0.15)),
            "National guideline: `ok` with the citation. A convention: `convention`.",
            &["qalys", "icer"],
            false,
        ),
        question(
            "gdp_per_capita",
            "GDP per capita",
            "analysis",
            "local currency per capita",
            QuestionKind::Quantity,
            "The gross domestic product per capita of the country in the price year. It is only used to derive the willingness-to-pay thresholds.",
            "The thresholds decide at which price a strategy is considered good value for money, so the GDP choice shapes the headline conclusion.",
            "National accounts (Eurostat, World Bank) for the price year, in the currency of the analysis.",
            Some((0.0, 1_000_000.0)),
            "National accounts: `ok` with the citation. International estimate: `transfer`.",
            &["inmb", "pstar"],
            true,
        ),
        question(
            "gdp_fx_usd",
            "Exchange rate, local currency per USD",
            "analysis",
            "local currency per USD",
            QuestionKind::Quantity,
            "The exchange rate used to convert a dollar value into the currency of the analysis.",
            "Only needed when a source is reported in dollars; the engine itself works in the local currency.",
            "The central bank or the international source that published the dollar figure.",
            Some((0.0, 100_000.0)),
            "Central bank: `ok`.",
            &[],
            true,
        ),
        question(
            "wtp_multiplier_0_5",
            "Willingness-to-pay threshold, 0.5 × GDP per capita",
            "analysis",
            "multiple of GDP per capita",
            QuestionKind::Quantity,
            "The lower decision threshold: half of GDP per capita per QALY.",
            "It is a decision convention, not an estimate; the results are reported against all three thresholds so that the reader can judge the choice.",
            "The study protocol or the national guideline that sets the threshold.",
            Some((0.0, 10.0)),
            "Decision convention: `convention`, with the protocol cited.",
            &["inmb", "pstar"],
            true,
        ),
        question(
            "wtp_multiplier_1",
            "Willingness-to-pay threshold, 1 × GDP per capita",
            "analysis",
            "multiple of GDP per capita",
            QuestionKind::Quantity,
            "The central decision threshold: one times GDP per capita per QALY.",
            "It is the threshold most often used for the headline conclusion.",
            "The study protocol or the national guideline that sets the threshold.",
            Some((0.0, 10.0)),
            "Decision convention: `convention`, with the protocol cited.",
            &["inmb", "pstar"],
            true,
        ),
        question(
            "wtp_multiplier_3",
            "Willingness-to-pay threshold, 3 × GDP per capita",
            "analysis",
            "multiple of GDP per capita",
            QuestionKind::Quantity,
            "The upper decision threshold: three times GDP per capita per QALY.",
            "It shows which strategies become good value for money under a more generous threshold.",
            "The study protocol or the national guideline that sets the threshold.",
            Some((0.0, 10.0)),
            "Decision convention: `convention`, with the protocol cited.",
            &["inmb", "pstar"],
            true,
        ),
        question(
            "analysis_perspective",
            "Analysis perspective",
            "analysis",
            "",
            QuestionKind::Text,
            "Whose costs and benefits are counted: the health-care payer, the health system, or society.",
            "A societal perspective adds productivity losses, which the current engine does not model; say which perspective the results are in.",
            "The study protocol and the national guideline.",
            None,
            "Decision convention: `convention`, with the protocol cited.",
            &["costs"],
            false,
        ),
        question(
            "currency",
            "Currency",
            "analysis",
            "",
            QuestionKind::Text,
            "The ISO 4217 code of the currency in which every monetary value of the pack is expressed.",
            "All costs have to be in the same currency; the engine does not convert them.",
            "The study protocol and the price year.",
            None,
            "Fact: `ok`.",
            &["costs", "icer"],
            false,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::demo_pack;

    #[test]
    fn catalogue_covers_every_parameter_of_the_demo_pack() {
        let catalogue = catalogue();
        for key in demo_pack().parameters.keys() {
            assert!(
                catalogue.question(key).is_some(),
                "the catalogue does not cover the parameter '{key}'"
            );
        }
    }

    #[test]
    fn every_question_has_a_definition_and_a_source() {
        let catalogue = catalogue();
        for question in &catalogue.questions {
            assert!(!question.definition.is_empty(), "{} has no definition", question.key);
            assert!(
                !question.how_to_source.is_empty(),
                "{} has no sourcing guidance",
                question.key
            );
            assert!(
                !question.evidence_guidance.is_empty(),
                "{} has no evidence guidance",
                question.key
            );
            assert!(
                catalogue.questions_in_group(&question.group).iter().any(|q| q.key == question.key),
                "{} belongs to an unknown group",
                question.key
            );
        }
    }

    #[test]
    fn a_submission_becomes_a_pack_with_provenance() {
        let catalogue = catalogue();
        let mut answers = BTreeMap::new();
        answers.insert(
            "births".to_string(),
            Answer {
                value: Some(Value::from(60_000)),
                status: EvidenceStatus::Ok,
                source: Some("National statistics office, 2024".into()),
                notes: None,
            },
        );
        let submission = Submission {
            country: CountryIdentity {
                name: "Ghana".into(),
                iso3: "GHA".into(),
                currency: "GHS".into(),
                price_year: 2024,
                perspective: "healthcare".into(),
                region: "Africa".into(),
                locale: "en".into(),
                notes: String::new(),
            },
            answers,
        };
        let pack = submission.to_pack().expect("pack");
        assert_eq!(pack.country.iso3, "GHA");
        assert_eq!(pack.parameters["births"].value, Some(Value::from(60_000)));
        assert_eq!(pack.parameters["births"].status, EvidenceStatus::Ok);
        assert_eq!(
            pack.parameters["births"].source.as_deref(),
            Some("National statistics office, 2024")
        );
        // Unanswered questions are missing, never silently zero.
        assert_eq!(pack.parameters["cfr_hospital"].value, None);
        assert_eq!(pack.parameters["cfr_hospital"].status, EvidenceStatus::Missing);
        assert!(pack.scenarios.iter().any(|scenario| scenario.is_reference));
        assert_eq!(pack.parameters.len(), catalogue.questions.len());
    }

    #[test]
    fn progress_counts_answered_and_blocking_questions() {
        let catalogue = catalogue();
        let pack = demo_pack();
        let progress = catalogue.progress(&pack);
        assert_eq!(progress.total, catalogue.questions.len());
        assert!(progress.answered > 0);
        assert!(progress.blocking > 0, "the synthetic demo blocks production");
    }

    #[test]
    fn the_toolkit_exports_markdown_and_csv() {
        let catalogue = catalogue();
        let markdown = catalogue.to_markdown();
        assert!(markdown.contains("## 1. Demography"));
        assert!(markdown.contains("### births — Annual births"));
        assert!(markdown.contains("Evidence statuses"));
        let csv = catalogue.to_csv();
        assert!(csv.lines().count() > catalogue.questions.len());
        assert!(csv.starts_with("group,order,key,label"));
    }
}
