#!/usr/bin/env Rscript
#
# Extract the Belgian upstream reference values that the Rust engine is checked
# against.
#
# The source of truth is the committed upstream fixture
# `tests/reference/belgium_upstream/` (produced by an unmodified RSV_main.R run).
# This script copies the decision-relevant numbers into a small JSON file that
# the Rust test suite reads.
#
# Usage:
#   Rscript mcmarcel-rs/parity/extract_belgium_reference.R [output.json]

suppressPackageStartupMessages(library(jsonlite))

args <- commandArgs(trailingOnly = FALSE)
file_arg <- grep("^--file=", args, value = TRUE)
script_path <- normalizePath(sub("^--file=", "", file_arg[1]))
rs_root <- normalizePath(file.path(dirname(script_path), ".."))
source(file.path(dirname(script_path), "mcmarcel_root.R"))
repo_root <- resolve_mcmarcel_root(script_path)

out_path <- if (length(commandArgs(trailingOnly = TRUE)) > 0) {
  commandArgs(trailingOnly = TRUE)[1]
} else {
  file.path(rs_root, "parity", "expected", "belgium_reference.json")
}

ref_dir <- file.path(repo_root, "tests", "reference", "belgium_upstream")
summary_stats <- read.csv(file.path(ref_dir, "RSV_BELX_basecase_summary_statistics_mean.csv"),
                          check.names = FALSE, stringsAsFactors = FALSE)

# The upstream summary table is the mean over the 20 upstream simulations.
pick <- function(scenario_id, column) summary_stats[[column]][summary_stats$scenario_id == scenario_id]

reference <- list(
  fixture = "belgium_upstream",
  source = "tests/reference/belgium_upstream/RSV_BELX_basecase_summary_statistics_mean.csv",
  description = paste(
    "Mean values of the upstream Belgian base case (20 simulations, seed 20190118).",
    "The Rust engine is compared against these numbers with a documented tolerance:"
  ),
  tolerance = list(
    cases = 0.05,
    hospital = 0.05,
    deaths = 0.10,
    qaly = 0.05,
    incremental_cost = 0.10
  ),
  scenarios = lapply(seq_len(nrow(summary_stats)), function(row) {
    id <- summary_stats$scenario_id[row]
    list(
      scenario_id = id,
      intervention = summary_stats$intervention[row],
      scenario = summary_stats$scenario[row],
      total_cases_disc_averted = pick(id, "total_cases_disc_averted"),
      hosp_cases_disc_averted = pick(id, "hosp_cases_disc_averted"),
      rsv_deaths_disc_averted = pick(id, "rsv_deaths_disc_averted"),
      total_QALY_disc_averted = pick(id, "total_QALY_disc_averted"),
      intervention_cost_program = pick(id, "intervention_cost_program"),
      incremental_cost_disc_program = pick(id, "incremental_cost_disc_program")
    )
  })
)

dir.create(dirname(out_path), recursive = TRUE, showWarnings = FALSE)
write(toJSON(reference, auto_unbox = TRUE, digits = NA, pretty = TRUE), out_path)
cat("wrote", out_path, "with", nrow(summary_stats), "reference scenarios\n")
