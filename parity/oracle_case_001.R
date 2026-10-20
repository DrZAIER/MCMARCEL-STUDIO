#!/usr/bin/env Rscript
#
# Parity oracle for the McMarcel Studio rewrite.
#
# The R implementation in the parent directory is the validation oracle for the
# Rust engine. This script recomputes a set of shared quantities with the *real*
# R functions and writes them to JSON; the Rust integration test
# (`crates/mcmarcel-core/tests/parity.rs`) recomputes the same quantities and
# compares them.
#
# Usage:
#   Rscript parity/oracle_case_001.R [output.json]
#
# The random streams of R and Rust are intentionally different, so the fixture
# covers the deterministic shared machinery: cohort conversion, protection by
# cohort, the unit-price formula, the gamma parameterisation, age interpolation
# and the net monetary benefit.

suppressPackageStartupMessages(library(jsonlite))

# Locate the repository root from this script's location.
args <- commandArgs(trailingOnly = FALSE)
file_arg <- grep("^--file=", args, value = TRUE)
script_path <- normalizePath(sub("^--file=", "", file_arg[1]))
repo_root <- normalizePath(file.path(dirname(script_path), "..", ".."))

source(file.path(repo_root, "functions", "RSV_get_burden.R"))
source(file.path(repo_root, "functions", "RSV_get_incidence.R"))
source(file.path(repo_root, "functions", "RSV_get_life_table.R"))
source(file.path(repo_root, "functions", "MISC_CEA.R"))

out_path <- if (length(commandArgs(trailingOnly = TRUE)) > 0) {
  commandArgs(trailingOnly = TRUE)[1]
} else {
  file.path(dirname(script_path), "expected", "case_001.json")
}

# ---------------------------------------------------------------- fixtures ---
# A deterministic 60 x 12 time-age matrix: increasing with age, seasonal with a
# December peak, so both the age and the calendar dimension are exercised.
set.seed(20260120)
age_effect <- 1 + 0.02 * (0:59)
season <- 1 + 0.25 * cos(2 * pi * ((0:11) - 11) / 12)
time_age <- matrix(0, nrow = 60, ncol = 12)
for (age in 0:59) {
  for (month in 0:11) {
    time_age[age + 1, month + 1] <- age_effect[age + 1] * season[month + 1]
  }
}

# A deterministic age-specific efficacy vector (12 months of protection).
efficacy <- c(rep(0.85, 6), rep(0.60, 6))

# ------------------------------------------------------------------ case 1 ---
# Cohort conversion: time-age matrix -> cohort-age matrix.
cohort <- time_age_to_cohort_age(time_age)

# ------------------------------------------------------------------ case 2 ---
# Protection by cohort for a year-round programme without catch-up.
protection_year_round <- get_protection_by_cohort(
  f_time = efficacy,
  intervention_start = 1,
  intervention_end = 12,
  coverage = 0.95,
  coverage_catch_up = 0
)

# ------------------------------------------------------------------ case 3 ---
# Protection by cohort for a seasonal programme with catch-up.
protection_seasonal <- get_protection_by_cohort(
  f_time = efficacy,
  intervention_start = 9,
  intervention_end = 3,
  coverage = 0.95,
  coverage_catch_up = 0.4
)

# ------------------------------------------------------------------ case 4 ---
# Unit price of one dose (defect C7: one formula for both products).
unit_price_maternal <- compute_unit_price(
  admin_cost = 12.5,
  dose_price = 95.0,
  wastage = 0.05,
  implementation_cost = 250000.0,
  doses = 120000.0
)
unit_price_mab <- compute_unit_price(
  admin_cost = 380.0,
  dose_price = 2100.0,
  wastage = 0.02,
  implementation_cost = 500000.0,
  doses = 95000.0
)

# ------------------------------------------------------------------ case 5 ---
# Gamma parameterisation used by `sample_rgamma` (shape/rate).
gamma_mean <- 420.0
gamma_sd <- 85.0
gamma_shape <- (gamma_mean^2) / (gamma_sd^2)
gamma_rate <- 1 / ((gamma_sd^2) / gamma_mean)   # rgamma(shape, rate)
gamma_scale <- (gamma_sd^2) / gamma_mean          # 1 / rate

# ------------------------------------------------------------------ case 6 ---
# Age interpolation of a two-band cost (constant interpolation, `rule = 2`).
age_breaks <- c(0, 11, 59)
age_sim <- matrix(c(120.0, 240.0), nrow = 2, ncol = 1)
interpolated <- approx_age_by_sim(age_sim, age_breaks)

# ------------------------------------------------------------------ case 7 ---
# Discounting vector over the age range (1 / (1 + r)^year).
disc_rate_cost <- 0.03
disc_rate_effect <- 0.03
years_after_vaccin <- rep(0:4, each = 12)
disc_time_cost <- 1 / ((1 + disc_rate_cost)^years_after_vaccin)
disc_time_effect <- 1 / ((1 + disc_rate_effect)^years_after_vaccin)

# ------------------------------------------------------------------ case 8 ---
# Net monetary benefit (MISC_CEA.R) for a set of decision-relevant points.
nb <- get_net_benefit(
  daly_averted = c(0, 25.5, 60.25, 120.0),
  incr_cost_disc = c(0, 1200000, 4800000, 13500000),
  wtp_level = 4188.0
)

fixture <- list(
  fixture = "case_001",
  description = "Deterministic shared machinery between the R implementation and the Rust engine.",
  r_version = R.version.string,
  generated_by = "parity/oracle_case_001.R",
  time_age_matrix = time_age,
  cohort_matrix = cohort,
  protection_year_round = protection_year_round,
  protection_seasonal = protection_seasonal,
  efficacy_vector = efficacy,
  unit_price_maternal = unit_price_maternal,
  unit_price_mab = unit_price_mab,
  gamma_shape = gamma_shape,
  gamma_rate = gamma_rate,
  gamma_scale = gamma_scale,
  gamma_mean = gamma_mean,
  gamma_sd = gamma_sd,
  age_interpolated = interpolated,
  age_breaks = age_breaks,
  discount_cost = disc_time_cost,
  discount_effect = disc_time_effect,
  net_benefit = nb
)

dir.create(dirname(out_path), recursive = TRUE, showWarnings = FALSE)
write(toJSON(fixture, auto_unbox = TRUE, digits = 12, pretty = TRUE), out_path)
cat("wrote", out_path, "\n")
