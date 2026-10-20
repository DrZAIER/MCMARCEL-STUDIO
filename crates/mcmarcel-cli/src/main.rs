use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use mcmarcel_core::engine::run_pack;
use mcmarcel_core::pack::demo_pack;
use mcmarcel_core::types::{CountryPack, RunMode, RunSpec};
use mcmarcel_core::validate::validate_pack;
use mcmarcel_core::ENGINE_VERSION;

#[derive(Parser)]
#[command(name = "mcmarcel", version, about = "McMarcel Studio headless runner")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Write a synthetic demo country pack.
    Demo {
        #[arg(long, default_value = "packs/tunisia-demo.json")]
        out: PathBuf,
    },
    /// Export the country-pack questionnaire as a standalone toolkit.
    Questionnaire {
        #[arg(long, default_value = "docs/questionnaire")]
        out: PathBuf,
        /// Also write the machine-readable catalogue as JSON.
        #[arg(long, default_value_t = true)]
        json: bool,
    },
    /// Validate a country pack without running the model.
    Validate {
        #[arg(long)]
        pack: PathBuf,
        #[arg(long, default_value = "fixture")]
        mode: String,
    },
    /// Run the probabilistic cohort model and write result artifacts.
    Run {
        #[arg(long)]
        pack: Option<PathBuf>,
        #[arg(long, default_value = "output")]
        out: PathBuf,
        #[arg(long, default_value_t = 500)]
        draws: u32,
        #[arg(long, default_value_t = 20_260_120)]
        seed: u64,
        #[arg(long, default_value = "fixture")]
        mode: String,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Demo { out } => {
            let pack = demo_pack();
            pack.save_json_file(&out)?;
            println!("wrote synthetic demo pack to {}", out.display());
        }
        Command::Questionnaire { out, json } => {
            let catalogue = mcmarcel_core::questionnaire::catalogue();
            fs::create_dir_all(&out)?;
            let markdown = out.join("questionnaire.md");
            fs::write(&markdown, catalogue.to_markdown())?;
            println!("wrote {}", markdown.display());
            let csv = out.join("questionnaire.csv");
            fs::write(&csv, catalogue.to_csv())?;
            println!("wrote {}", csv.display());
            if json {
                let catalogue_path = out.join("questionnaire.json");
                fs::write(&catalogue_path, serde_json::to_string_pretty(&catalogue)?)?;
                println!("wrote {}", catalogue_path.display());
            }
            println!(
                "{} questions in {} sections",
                catalogue.questions.len(),
                catalogue.groups.len()
            );
        }
        Command::Validate { pack, mode } => {
            let pack = CountryPack::from_json_file(pack)?;
            let mode = parse_mode(&mode)?;
            let report = validate_pack(&pack, mode);
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::Run { pack, out, draws, seed, mode } => {
            let pack = match pack {
                Some(path) => CountryPack::from_json_file(path)?,
                None => demo_pack(),
            };
            let mode = parse_mode(&mode)?;
            let spec = RunSpec { draws, seed, mode };
            let result = run_pack(&pack, &spec)?;
            fs::create_dir_all(&out)?;

            let result_json = serde_json::to_string_pretty(&result)?;
            fs::write(out.join("results.json"), result_json)?;

            let mut writer = csv::Writer::from_path(out.join("summary.csv"))?;
            writer.write_record([
                "scenario_id",
                "scenario",
                "coverage_maternal",
                "coverage_mab",
                "averted_cases",
                "averted_hospital",
                "averted_deaths",
                "medical_cost_averted",
                "intervention_cost",
                "incremental_cost",
                "qaly_averted",
                "fvp",
                "doses",
            ])?;
            for scenario in &result.scenarios {
                writer.write_record([
                    scenario.scenario_id.clone(),
                    scenario.scenario_name.clone(),
                    scenario.coverage_maternal.to_string(),
                    scenario.coverage_mab.to_string(),
                    scenario.mean_averted_cases.to_string(),
                    scenario.mean_averted_hospital.to_string(),
                    scenario.mean_averted_deaths.to_string(),
                    scenario.mean_medical_cost_averted.to_string(),
                    scenario.mean_intervention_cost.to_string(),
                    scenario.mean_incremental_cost.to_string(),
                    scenario.mean_qaly_averted.to_string(),
                    scenario.mean_fvp.to_string(),
                    scenario.mean_doses.to_string(),
                ])?;
            }
            writer.flush()?;

            let manifest = serde_json::json!({
                "engine": "mcmarcel-rs",
                "engine_version": ENGINE_VERSION,
                "pack_id": pack.country.id,
                "pack_name": pack.country.name,
                "pack_hash": result.pack_hash,
                "mode": match mode { RunMode::Production => "production", RunMode::Fixture => "fixture" },
                "draws": draws,
                "seed": seed,
                "fixture_stamp": result.fixture_stamp,
                "created_at_ms": result.created_at_ms,
            });
            fs::write(out.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;

            println!("wrote results to {}", out.display());
            if let Some(stamp) = &result.fixture_stamp {
                println!("WARNING: {stamp}");
            }
            println!(
                "{} scenarios, {} draws, pack hash {}",
                result.scenarios.len(),
                result.draws,
                &result.pack_hash[..12.min(result.pack_hash.len())]
            );
        }
    }
    Ok(())
}

fn parse_mode(value: &str) -> Result<RunMode, Box<dyn std::error::Error>> {
    match value.to_ascii_lowercase().as_str() {
        "production" => Ok(RunMode::Production),
        "fixture" => Ok(RunMode::Fixture),
        other => Err(format!("unknown mode '{other}', expected 'fixture' or 'production'").into()),
    }
}
