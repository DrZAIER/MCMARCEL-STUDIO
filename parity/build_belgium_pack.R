#!/usr/bin/env Rscript
#
# Build the McMarcel Studio country pack for the Belgian reference case.
#
# The R implementation in the parent directory is the validation oracle. This
# script reads the *same* input files with the *same* transformations that
# `functions/RSV_get_incidence.R` and `functions/RSV_config.R` apply, and writes
# the result as a McMarcel Studio country pack (JSON).
#
# Usage:
#   Rscript mcmarcel-rs/parity/build_belgium_pack.R [output.json]
#
# The published Belgian inputs in this repository are the perturbed public
# release (files carrying an X in their name). The pack therefore marks every
# perturbed value as SYNTHETIC, which blocks production mode and stamps every
# run: the pack is a methodological reference, not a country estimate.

suppressPackageStartupMessages({
  library(jsonlite)
})

args <- commandArgs(trailingOnly = FALSE)
file_arg <- grep("^--file=", args, value = TRUE)
script_path <- normalizePath(sub("^--file=", "", file_arg[1]))
rs_root <- normalizePath(file.path(dirname(script_path), ".."))
source(file.path(dirname(script_path), "mcmarcel_root.R"))
repo_root <- resolve_mcmarcel_root(script_path)
# The upstream configuration reads its input files relative to the working
# directory, so the script runs from the root of the R project.
setwd(repo_root)

source(file.path(repo_root, "functions", "RSV_load_all.R"))

out_path <- if (length(commandArgs(trailingOnly = TRUE)) > 0) {
  commandArgs(trailingOnly = TRUE)[1]
} else {
  file.path(rs_root, "packs", "belgium-belx.json")
}

input_dir <- file.path(repo_root, "input")
find_input <- function(stem) {
  hits <- list.files(input_dir, pattern = "\\.csv$", full.names = TRUE)
  exact <- hits[basename(hits) == paste0(stem, ".csv")]
  if (length(exact) > 0) return(exact[1])
  partial <- hits[grepl(stem, basename(hits), fixed = TRUE)]
  if (length(partial) == 0) stop("input file not found: ", stem)
  partial[1]
}

## ------------------------------------------------------------------ helpers ---
param <- function(key, label, value, unit, status, source = NULL, notes = NULL, low = NULL, high = NULL) {
  list(
    key = key,
    label = label,
    unit = unit,
    value = value,
    low = low,
    high = high,
    status = status,
    source = source,
    source_ref = NULL,
    url = NULL,
    notes = notes,
    blocks_outputs = list()
  )
}

matrix_json <- function(data, rows, cols) {
  list(rows = rows, cols = cols, data = as.numeric(t(data)))
}

## ------------------------------------------------------------------- inputs ---
cfg <- openxlsx::read.xlsx(file.path(repo_root, "config", "RSV_BELX_basecase.xlsx"),
                           sheet = "scenarios", na.strings = "-")

country_file <- find_input("20250106_BE_country_details")
country_data <- read.csv(country_file, header = TRUE, stringsAsFactors = FALSE)
flag <- country_data$population_code == "BEL" & country_data$year == 2024
if (!any(flag)) stop("no Belgian 2024 demographics found")
target_population <- country_data$target_population[flag][1]
stillbirth_rate <- country_data$stillbirth_rate[flag][1]
pre_term_rate <- country_data$incomplete_maternal_transfer_rate[flag][1]

hosp_file <- find_input("totalnrhospadmissions")
icu_file <- find_input("totalnrICUadmissions")
hosp_raw <- read.csv(hosp_file, header = TRUE)
icu_raw <- read.csv(icu_file, header = TRUE)
hosp_cohort <- 108680   # config$hospdata_birth_cohort
icu_cohort <- 108680    # config$icudata_birth_cohort
hosp_seasons <- 1       # config$hospdata_num_seasons
icu_seasons <- 1        # config$icudata_num_seasons

hospital_matrix <- as.matrix(hosp_raw[, -1]) / hosp_seasons / hosp_cohort * target_population
icu_matrix <- as.matrix(icu_raw[, -1]) / icu_seasons / icu_cohort * target_population

outpatient_file <- find_input("outpatient_incidence")
outpatient_matrix <- as.matrix(read.csv(outpatient_file, header = TRUE)[, -1])  # 60 x n samples

non_ma_file <- find_input("nonMA_cases")
non_ma_matrix <- as.matrix(read.csv(non_ma_file, header = TRUE))                  # n samples x 60 ages

hcfr_file <- find_input("hCFR_byAge")
hcfr_raw <- read.csv(hcfr_file, header = TRUE)
hcfr_matrix <- cbind(hcfr_raw$hCFR_hosp, hcfr_raw$hCFR_icu)                       # hospital, ICU

## ------------------------------------------------------------- life table ---
life_table <- get_life_table_BEL2023(0.015, qol_norms_file = file.path(input_dir, "eq5d", "popnormINDEX.rds"))
life_table <- life_table[1:60, ]
life_matrix <- cbind(
  age_years = (0:59) / 12,
  le = life_table$LE,
  le_disc = life_table$LE_disc,
  hrqale = life_table$HrQALE,
  hrqale_disc = life_table$HrQALE_disc
)

qol_norms <- popnormINDEX_BE(file.path(input_dir, "eq5d", "popnormINDEX.rds"))$mean
qol_mean <- mean(as.numeric(qol_norms[1:min(60, length(qol_norms))]), na.rm = TRUE)

## ------------------------------------------------------------------ costs ---
cost_file <- find_input("cost_perspectives_regions")
db_cost <- read.csv(cost_file, header = TRUE)
db_cost <- db_cost[db_cost$perspective == "payer", ]
se_from_ci <- function(lci, uci) abs((uci - lci) / (2 * 1.96))

outpatient_bands <- list(
  list(max_age_months = 11L,  mean_key = "cost_outpatient_under1y", sd_key = "cost_outpatient_under1y_sd", file_mean = "cost_outpatient_under1y_mean_BEL", file_lci = "cost_outpatient_under1y_lci_BEL", file_uci = "cost_outpatient_under1y_uci_BEL", label = "Outpatient cost, under 1 year"),
  list(max_age_months = 23L,  mean_key = "cost_outpatient_1y",       sd_key = "cost_outpatient_1y_sd",       file_mean = "cost_outpatient_1y_mean_BEL",       file_lci = "cost_outpatient_1y_lci_BEL",       file_uci = "cost_outpatient_1y_uci_BEL",       label = "Outpatient cost, 1 year"),
  list(max_age_months = 35L,  mean_key = "cost_outpatient_2y",       sd_key = "cost_outpatient_2y_sd",       file_mean = "cost_outpatient_2y_mean_BEL",       file_lci = "cost_outpatient_2y_lci_BEL",       file_uci = "cost_outpatient_2y_uci_BEL",       label = "Outpatient cost, 2 years"),
  list(max_age_months = 47L,  mean_key = "cost_outpatient_3y",       sd_key = "cost_outpatient_3y_sd",       file_mean = "cost_outpatient_3y_mean_BEL",       file_lci = "cost_outpatient_3y_lci_BEL",       file_uci = "cost_outpatient_3y_uci_BEL",       label = "Outpatient cost, 3 years"),
  list(max_age_months = 59L,  mean_key = "cost_outpatient_4y",       sd_key = "cost_outpatient_4y_sd",       file_mean = "cost_outpatient_4y_mean_BEL",       file_lci = "cost_outpatient_4y_lci_BEL",       file_uci = "cost_outpatient_4y_uci_BEL",       label = "Outpatient cost, 4 years")
)
hospital_bands <- list(
  list(max_age_months = 11L,  mean_key = "cost_hospital_under1y", sd_key = "cost_hospital_under1y_sd", file_mean = "cost_hosp_under1y_mean_BEL", file_sd = "cost_hosp_under1y_sd_BEL", label = "Hospital cost, under 1 year"),
  list(max_age_months = 59L,  mean_key = "cost_hospital_1to4y",   sd_key = "cost_hospital_1to4y_sd",   file_mean = "cost_hosp_1to4y_mean_BEL",   file_sd = "cost_hosp_1to4y_sd_BEL",   label = "Hospital cost, 1 to 4 years")
)
icu_bands <- list(
  list(max_age_months = 11L,  mean_key = "cost_icu_under1y", sd_key = "cost_icu_under1y_sd", file_mean = "cost_icu_under1y_mean_BEL", file_sd = "cost_icu_under1y_sd_BEL", label = "ICU cost, under 1 year"),
  list(max_age_months = 59L,  mean_key = "cost_icu_1to4y",   sd_key = "cost_icu_1to4y_sd",   file_mean = "cost_icu_1to4y_mean_BEL",   file_sd = "cost_icu_1to4y_sd_BEL",   label = "ICU cost, 1 to 4 years")
)
non_ma_bands <- list(
  list(max_age_months = 59L,  mean_key = "cost_non_ma", sd_key = NULL, file_mean = "cost_non_ma_BEL", file_sd = NULL, label = "Non-medically-attended cost")
)

parameters <- list()

for (band in outpatient_bands) {
  parameters[[band$mean_key]] <- param(
    band$mean_key, band$label, db_cost[[band$file_mean]][1], "EUR per case", "synthetic",
    source = "input/BELX_cost_perspectives_regions.csv (payer, BEL)",
    notes = "Public-release perturbed Belgian outpatient unit cost for this age band."
  )
  parameters[[band$sd_key]] <- param(
    band$sd_key, paste0(band$label, ", SD"),
    se_from_ci(db_cost[[band$file_lci]][1], db_cost[[band$file_uci]][1]), "EUR per case", "synthetic",
    source = "input/BELX_cost_perspectives_regions.csv (payer, BEL)",
    notes = "Standard error derived from the published confidence interval as (uci - lci) / (2 x 1.96). The public-release perturbation can invert the bounds, so the absolute value is used."
  )
}
for (band in hospital_bands) {
  parameters[[band$mean_key]] <- param(
    band$mean_key, band$label, db_cost[[band$file_mean]][1], "EUR per admission", "synthetic",
    source = "input/BELX_cost_perspectives_regions.csv (payer, BEL)",
    notes = "Public-release perturbed Belgian hospital unit cost for this age band."
  )
  parameters[[band$sd_key]] <- param(
    band$sd_key, paste0(band$label, ", SD"), db_cost[[band$file_sd]][1], "EUR per admission", "synthetic",
    source = "input/BELX_cost_perspectives_regions.csv (payer, BEL)",
    notes = "Public-release perturbed Belgian hospital unit cost standard deviation."
  )
}
for (band in icu_bands) {
  parameters[[band$mean_key]] <- param(
    band$mean_key, band$label, db_cost[[band$file_mean]][1], "EUR per admission", "synthetic",
    source = "input/BELX_cost_perspectives_regions.csv (payer, BEL)",
    notes = "Public-release perturbed Belgian ICU unit cost for this age band."
  )
  parameters[[band$sd_key]] <- param(
    band$sd_key, paste0(band$label, ", SD"), db_cost[[band$file_sd]][1], "EUR per admission", "synthetic",
    source = "input/BELX_cost_perspectives_regions.csv (payer, BEL)",
    notes = "Public-release perturbed Belgian ICU unit cost standard deviation."
  )
}
parameters[["cost_non_ma"]] <- param(
  "cost_non_ma", "Non-medically-attended cost per case",
  db_cost$cost_non_ma_BEL[1], "EUR per case", "synthetic",
  source = "input/BELX_cost_perspectives_regions.csv (payer, BEL)",
  notes = "Public-release perturbed Belgian non-medically-attended unit cost. The Belgian inputs carry no uncertainty for this category."
)

# Scalar fallbacks (used only when a pack defines no age bands).
parameters[["cost_outpatient"]] <- param("cost_outpatient", "Outpatient cost per case", db_cost$cost_outpatient_under1y_mean_BEL[1], "EUR per case", "synthetic", source = "input/BELX_cost_perspectives_regions.csv", notes = "Fallback value; the age-banded values take precedence.")
parameters[["cost_outpatient_sd"]] <- param("cost_outpatient_sd", "Outpatient cost SD", se_from_ci(db_cost$cost_outpatient_under1y_lci_BEL[1], db_cost$cost_outpatient_under1y_uci_BEL[1]), "EUR per case", "synthetic", source = "input/BELX_cost_perspectives_regions.csv", notes = "Fallback value; the age-banded values take precedence.")
parameters[["cost_hospital"]] <- param("cost_hospital", "Hospital cost per admission", db_cost$cost_hosp_under1y_mean_BEL[1], "EUR per admission", "synthetic", source = "input/BELX_cost_perspectives_regions.csv", notes = "Fallback value; the age-banded values take precedence.")
parameters[["cost_hospital_sd"]] <- param("cost_hospital_sd", "Hospital cost SD", db_cost$cost_hosp_under1y_sd_BEL[1], "EUR per admission", "synthetic", source = "input/BELX_cost_perspectives_regions.csv", notes = "Fallback value; the age-banded values take precedence.")
parameters[["cost_icu"]] <- param("cost_icu", "ICU cost per admission", db_cost$cost_icu_under1y_mean_BEL[1], "EUR per admission", "synthetic", source = "input/BELX_cost_perspectives_regions.csv", notes = "Fallback value; the age-banded values take precedence.")
parameters[["cost_icu_sd"]] <- param("cost_icu_sd", "ICU cost SD", db_cost$cost_icu_under1y_sd_BEL[1], "EUR per admission", "synthetic", source = "input/BELX_cost_perspectives_regions.csv", notes = "Fallback value; the age-banded values take precedence.")
parameters[["cost_non_ma_sd"]] <- param("cost_non_ma_sd", "Non-MA cost SD", 0, "EUR per case", "convention", notes = "The Belgian reference inputs carry no uncertainty for non-medically-attended costs.")
parameters[["cost_wheezing"]] <- param("cost_wheezing", "Wheezing sequelae cost", 0, "EUR per case", "convention", notes = "The recurrent-wheezing module is switched off in the Belgian base case (wheezing_horizon = 0), so the value is zero.")
parameters[["cost_wheezing_sd"]] <- param("cost_wheezing_sd", "Wheezing cost SD", 0, "EUR per case", "convention", notes = "The recurrent-wheezing module is switched off in the Belgian base case.")
parameters[["qaly_loss_wheezing"]] <- param("qaly_loss_wheezing", "QALY loss, wheezing", 0, "QALY/case", "convention", notes = "The recurrent-wheezing module is switched off in the Belgian base case, so the value is zero.")
parameters[["qaly_loss_wheezing_sd"]] <- param("qaly_loss_wheezing_sd", "QALY loss SD, wheezing", 0, "QALY/case", "convention", notes = "The recurrent-wheezing module is switched off in the Belgian base case.")

## --------------------------------------------------------- QALY losses -----
# Values hard-coded in functions/RSV_config.R for the Belgian base case.
parameters[["qaly_loss_hospital"]] <- param("qaly_loss_hospital", "QALY loss, hospital", 0.00982, "QALY/case", "transfer", source = "functions/RSV_config.R (QALYloss_hosp_mean)", notes = "Mao et al., adapted for Belgium by the upstream model; carried over from the hospitalised-case literature.")
parameters[["qaly_loss_hospital_sd"]] <- param("qaly_loss_hospital_sd", "QALY loss SD, hospital", se_from_ci(0.0085, 0.0114), "QALY/case", "transfer", source = "functions/RSV_config.R", notes = "Derived from the published confidence interval (0.0085 - 0.0114).")
parameters[["qaly_loss_outpatient"]] <- param("qaly_loss_outpatient", "QALY loss, outpatient", 0.00589, "QALY/case", "transfer", source = "functions/RSV_config.R (QALYloss_outpatient_mean)", notes = "Mao et al., adapted for Belgium by the upstream model.")
parameters[["qaly_loss_outpatient_sd"]] <- param("qaly_loss_outpatient_sd", "QALY loss SD, outpatient", se_from_ci(0.0052, 0.0067), "QALY/case", "transfer", source = "functions/RSV_config.R", notes = "Derived from the published confidence interval (0.0052 - 0.0067).")
parameters[["qaly_loss_non_ma"]] <- param("qaly_loss_non_ma", "QALY loss, non-MA", 0.00337, "QALY/case", "transfer", source = "functions/RSV_config.R (QALY_non_ma_mean)", notes = "Belgian value set used by the upstream model.")
parameters[["qaly_loss_non_ma_sd"]] <- param("qaly_loss_non_ma_sd", "QALY loss SD, non-MA", se_from_ci(0.0028, 0.004), "QALY/case", "transfer", source = "functions/RSV_config.R", notes = "Derived from the published confidence interval (0.0028 - 0.004).")
parameters[["qol_mean"]] <- param("qol_mean", "Mean population utility", qol_mean, "utility", "ok", source = "input/eq5d/popnormINDEX.rds (EQ-5D Belgian general population norms, BE 2018)", notes = "Mean of the age-specific population means over the first 60 months; only used when a life table has to be generated.")

## ---------------------------------------------------------- demography -----
parameters[["births"]] <- param("births", "Annual births", target_population, "births/year", "ok", source = "input/20250106_BE_country_details.csv (BEL, 2024)", notes = "Target population of the 2024 Belgian birth cohort used by the reference run.")
parameters[["stillbirth_rate"]] <- param("stillbirth_rate", "Stillbirth rate", stillbirth_rate, "proportion", "ok", source = "input/20250106_BE_country_details.csv (BEL, 2024)", notes = "Official demographic input of the reference run.")
parameters[["pre_term_rate"]] <- param("pre_term_rate", "Pre-term birth rate", pre_term_rate, "proportion", "ok", source = "input/20250106_BE_country_details.csv (BEL, 2024)", notes = "Incomplete maternal transfer rate of the reference run.")

## ------------------------------------------------------------ analysis -----
parameters[["discount_rate_cost"]] <- param("discount_rate_cost", "Cost discount rate", 0.030, "annual", "convention", source = "functions/RSV_config.R (disc_rate_cost)", notes = "Convention of the upstream model.")
parameters[["discount_rate_effect"]] <- param("discount_rate_effect", "Effect discount rate", 0.015, "annual", "convention", source = "functions/RSV_config.R (disc_rate_effect)", notes = "Convention of the upstream model.")
parameters[["analysis_perspective"]] <- param("analysis_perspective", "Analysis perspective", "payer", "", "convention", source = "functions/RSV_config.R (perspective)", notes = "The Belgian base case uses the health-care payer perspective; societal productivity costs are not part of it.")
parameters[["currency"]] <- param("currency", "Currency", "EUR", "", "ok", source = "config/RSV_BELX_basecase.xlsx", notes = "All Belgian inputs are in euro.")
parameters[["gdp_per_capita"]] <- param("gdp_per_capita", "GDP per capita (local currency)", 52270, "EUR per capita", "ok", source = "Eurostat nama_10_pc, Belgium 2024, current prices (CP_EUR_HAB)", notes = "Used only to derive the willingness-to-pay thresholds (0.5x, 1x, 3x GDP per capita).")
parameters[["gdp_fx_usd"]] <- param("gdp_fx_usd", "Exchange rate, local currency per USD", 1, "EUR/USD", "convention", notes = "Not used by the engine; the Belgian analysis is entirely in euro.")
parameters[["wtp_multiplier_0_5"]] <- param("wtp_multiplier_0_5", "WTP threshold, 0.5x GDP", 0.5, "multiple", "convention", notes = "Decision convention, not an empirical estimate.")
parameters[["wtp_multiplier_1"]] <- param("wtp_multiplier_1", "WTP threshold, 1x GDP", 1.0, "multiple", "convention", notes = "Decision convention, not an empirical estimate.")
parameters[["wtp_multiplier_3"]] <- param("wtp_multiplier_3", "WTP threshold, 3x GDP", 3.0, "multiple", "convention", notes = "Decision convention, not an empirical estimate.")

## ------------------------------------------------------------ efficacy -----
ve_summary <- function(file_pattern) {
  raw <- as.matrix(read.table(find_input(file_pattern), sep = ",", header = FALSE))
  ages <- colMeans(raw)          # per age, over the bootstrap samples
  sds <- apply(raw, 2, sd)       # per age
  keep <- ages > 0
  list(mean = mean(ages[keep]), sd = mean(sds[keep]), months = sum(keep),
       per_age = ages, per_age_sd = sds)
}

## Age-specific profile: the full 60-month vector, padded with zeros where the
## efficacy file carries no protection (ages beyond the protection window).
ve_profile <- function(file_pattern) {
  raw <- as.matrix(read.table(find_input(file_pattern), sep = ",", header = FALSE))
  mean_age <- colMeans(raw)
  sd_age <- apply(raw, 2, sd)
  pad <- function(values) c(values, rep(0, 60 - length(values)))
  list(mean = pad(mean_age), sd = pad(sd_age))
}

ve_specs <- list(
  list(key = "ve_maternal_hospital",  file = "VE_maternal_CI_Hosp_MA", label = "Maternal VE, hospital"),
  list(key = "ve_maternal_outpatient", file = "VE_maternal_CI_MA",      label = "Maternal VE, outpatient"),
  list(key = "ve_maternal_icu",       file = "VE_maternal_CI_Seve_MA",  label = "Maternal VE, ICU"),
  list(key = "ve_maternal_non_ma",    file = "VE_maternal_CI_MA",      label = "Maternal VE, non-MA"),
  list(key = "ve_mab_hospital",       file = "VE_mab_CI_Hosp_MA",      label = "mAb VE, hospital"),
  list(key = "ve_mab_outpatient",     file = "VE_mab_CI_MA",           label = "mAb VE, outpatient"),
  list(key = "ve_mab_icu",            file = "VE_mab_CI_Hosp_MA",      label = "mAb VE, ICU"),
  list(key = "ve_mab_non_ma",         file = "VE_mab_CI_MA",           label = "mAb VE, non-MA")
)

for (spec in ve_specs) {
  summary <- ve_summary(spec$file)
  parameters[[spec$key]] <- param(
    spec$key, spec$label, summary$mean, "proportion", "ok",
    source = paste0("input/", spec$file),
    notes = sprintf("Age-specific efficacy for the first %d months, summarised over the %d bootstrap samples in the file.", summary$months, summary$months)
  )
  parameters[[paste0(spec$key, "_sd")]] <- param(
    paste0(spec$key, "_sd"), paste0(spec$label, " SD"), summary$sd, "proportion", "ok",
    source = paste0("input/", spec$file),
    notes = "Standard deviation of the bootstrap distribution in the efficacy file."
  )
}
# Mortality protection equals ICU protection in the upstream model.
parameters[["ve_maternal_death"]] <- parameters[["ve_maternal_icu"]]
parameters[["ve_maternal_death"]]$key <- "ve_maternal_death"
parameters[["ve_maternal_death"]]$label <- "Maternal VE, death"
parameters[["ve_maternal_death_sd"]] <- parameters[["ve_maternal_icu_sd"]]
parameters[["ve_maternal_death_sd"]]$key <- "ve_maternal_death_sd"
parameters[["ve_maternal_death_sd"]]$label <- "Maternal VE, death SD"
parameters[["ve_mab_death"]] <- parameters[["ve_mab_icu"]]
parameters[["ve_mab_death"]]$key <- "ve_mab_death"
parameters[["ve_mab_death"]]$label <- "mAb VE, death"
parameters[["ve_mab_death_sd"]] <- parameters[["ve_mab_icu_sd"]]
parameters[["ve_mab_death_sd"]]$key <- "ve_mab_death_sd"
parameters[["ve_mab_death_sd"]]$label <- "mAb VE, death SD"

## Age-specific efficacy profiles (the maternal vaccine wanes over the first
## months of life; the monoclonal antibody does not).
efficacy_profile <- list(
  maternal = list(
    hospital = ve_profile("VE_maternal_CI_Hosp_MA"),
    outpatient = ve_profile("VE_maternal_CI_MA"),
    icu = ve_profile("VE_maternal_CI_Seve_MA"),
    death = ve_profile("VE_maternal_CI_Seve_MA"),
    non_ma = ve_profile("VE_maternal_CI_MA")
  ),
  mab = list(
    hospital = ve_profile("VE_mab_CI_Hosp_MA"),
    outpatient = ve_profile("VE_mab_CI_MA"),
    icu = ve_profile("VE_mab_CI_Hosp_MA"),
    death = ve_profile("VE_mab_CI_Hosp_MA"),
    non_ma = ve_profile("VE_mab_CI_MA")
  )
)
efficacy_profile$maternal$hospital$mean <- as.numeric(efficacy_profile$maternal$hospital$mean)
efficacy_profile$maternal$hospital$sd <- as.numeric(efficacy_profile$maternal$hospital$sd)

## --------------------------------------------------- intervention inputs ---
parameters[["duration_maternal_years"]] <- param("duration_maternal_years", "Maternal protection duration", 0.5, "years", "ok", source = "input/VE_maternal_CI_*.csv", notes = "The efficacy files carry non-zero protection for the first six months only.")
parameters[["duration_mab_years"]] <- param("duration_mab_years", "mAb protection duration", 0.5, "years", "ok", source = "input/VE_mab_CI_*.csv", notes = "The efficacy files carry non-zero protection for the first six months only.")
parameters[["coverage_maternal"]] <- param("coverage_maternal", "Maternal vaccine coverage", 0.4, "proportion", "convention", source = "config/RSV_BELX_basecase.xlsx", notes = "Default coverage of the Belgian base case; individual scenarios override it.")
parameters[["coverage_mab"]] <- param("coverage_mab", "mAb coverage", 0.9, "proportion", "convention", source = "config/RSV_BELX_basecase.xlsx", notes = "Default coverage of the Belgian base case; individual scenarios override it.")
parameters[["price_maternal_min"]] <- param("price_maternal_min", "Maternal vaccine price, low", cfg$price_dose_maternal_min[1], "EUR per dose", "ok", source = "config/RSV_BELX_basecase.xlsx", notes = "Base-case price of the Belgian reference run.")
parameters[["price_maternal_max"]] <- param("price_maternal_max", "Maternal vaccine price, high", cfg$price_dose_maternal_max[1], "EUR per dose", "ok", source = "config/RSV_BELX_basecase.xlsx", notes = "Base-case price of the Belgian reference run.")
parameters[["price_mab_min"]] <- param("price_mab_min", "mAb price, low", cfg$price_dose_mAb_min[1], "EUR per dose", "ok", source = "config/RSV_BELX_basecase.xlsx", notes = "Base-case price of the Belgian reference run.")
parameters[["price_mab_max"]] <- param("price_mab_max", "mAb price, high", cfg$price_dose_mAb_max[1], "EUR per dose", "ok", source = "config/RSV_BELX_basecase.xlsx", notes = "Base-case price of the Belgian reference run.")
parameters[["admin_cost_maternal"]] <- param("admin_cost_maternal", "Maternal administration cost", 0, "EUR per dose", "convention", source = "functions/RSV_config.R", notes = "Switched off in the upstream Belgian base case.")
parameters[["admin_cost_mab"]] <- param("admin_cost_mab", "mAb administration cost", 0, "EUR per dose", "convention", source = "functions/RSV_config.R", notes = "Switched off in the upstream Belgian base case.")
parameters[["implementation_cost_maternal"]] <- param("implementation_cost_maternal", "Maternal implementation cost", 0, "EUR per dose", "convention", source = "functions/RSV_config.R", notes = "Switched off in the upstream Belgian base case.")
parameters[["implementation_cost_mab"]] <- param("implementation_cost_mab", "mAb implementation cost", 0, "EUR per dose", "convention", source = "functions/RSV_config.R", notes = "Switched off in the upstream Belgian base case.")
parameters[["wastage_maternal"]] <- param("wastage_maternal", "Maternal vaccine wastage", 0, "proportion", "convention", source = "functions/RSV_config.R", notes = "Switched off in the upstream Belgian base case.")
parameters[["wastage_mab"]] <- param("wastage_mab", "mAb wastage", 0, "proportion", "convention", source = "functions/RSV_config.R", notes = "Switched off in the upstream Belgian base case.")

## ------------------------------------------------------------- scenarios ---
season_label <- function(start, end, catch_up) {
  months <- month_id2name(c(start, end))
  paste0(months[1], months[2], if (catch_up > 0) "_CB" else "")
}

scenarios <- list(list(
  id = "none",
  name = "No intervention",
  is_reference = TRUE,
  coverage_maternal = 0,
  coverage_mab = 0,
  intervention_start = 1,
  intervention_end = 12,
  coverage_catch_up = 0,
  price_maternal = NULL,
  price_mab = NULL,
  notes = "Comparator arm of the Belgian reference run."
))

for (row in seq_len(nrow(cfg))) {
  is_maternal <- cfg$coverage_maternal[row] > 0
  is_mab <- cfg$coverage_mAb[row] > 0
  intervention <- if (is_maternal && is_mab) "maternal_mAb" else if (is_maternal) "maternal" else "mAb"
  season <- season_label(cfg$intervention_start[row], cfg$intervention_end[row], cfg$coverage_catch_up[row])
  scenarios[[length(scenarios) + 1]] <- list(
    id = sprintf("bel-%02d-%s-%s", row, tolower(cfg$scenario[row]), tolower(intervention)),
    name = sprintf("%s %s (%s)", intervention, season, cfg$scenario[row]),
    is_reference = FALSE,
    coverage_maternal = cfg$coverage_maternal[row],
    coverage_mab = cfg$coverage_mAb[row],
    intervention_start = cfg$intervention_start[row],
    intervention_end = cfg$intervention_end[row],
    coverage_catch_up = cfg$coverage_catch_up[row],
    price_maternal = if (is_maternal) cfg$price_dose_maternal_min[row] else NULL,
    price_mab = if (is_mab) cfg$price_dose_mAb_min[row] else NULL,
    notes = sprintf("Belgian reference scenario %d (%s, %s).", row, cfg$scenario[row], intervention)
  )
}

## ------------------------------------------------------------------ pack ---
pack <- list(
  schema_version = 1L,
  country = list(
    id = "bel-belx",
    name = "Belgium (BELX reference)",
    iso3 = "BEL",
    currency = "EUR",
    price_year = 2024L,
    locale = "en",
    perspective = "payer",
    region = "Europe",
    notes = paste(
      "Belgian reference case of the McMarcel framework, built from the public-release inputs of the repository.",
      "The country-specific input files (BELX, ESPX, RESCEUX) carry stochastic perturbation, so the values are not the observed Belgian values:",
      "every perturbed parameter is marked SYNTHETIC, production mode is blocked and every run is stamped.",
      "Use this pack to reproduce and validate the reference model, not to describe Belgium."
    )
  ),
  parameters = parameters,
  assets = list(
    hospital_age_calendar = matrix_json(hospital_matrix, 60, 12),
    icu_age_calendar = matrix_json(icu_matrix, 60, 12),
    outpatient_rate_age_sample = matrix_json(outpatient_matrix, 60, ncol(outpatient_matrix)),
    hcfr_age = matrix_json(hcfr_matrix, 60, 2),
    non_ma_sample_age = matrix_json(non_ma_matrix, nrow(non_ma_matrix), 60),
    life_table = matrix_json(life_matrix, 60, 5),
    cost_bands = list(
      outpatient = outpatient_bands,
      hospital = hospital_bands,
      icu = icu_bands,
      non_ma = non_ma_bands
    ),
    efficacy_profile = efficacy_profile
  ),
  scenarios = scenarios,
  notes = "Built by mcmarcel-rs/parity/build_belgium_pack.R from the R reference implementation's inputs."
)

dir.create(dirname(out_path), recursive = TRUE, showWarnings = FALSE)
write(toJSON(pack, auto_unbox = TRUE, digits = NA, null = "null", pretty = TRUE), out_path)
cat("wrote", out_path, "\n")
cat("parameters:", length(parameters), " scenarios:", length(scenarios), "\n")
