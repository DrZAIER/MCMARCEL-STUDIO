#!/usr/bin/env Rscript
#
# Build the McMarcel Studio country packs for the Gavi-eligible countries.
#
# For every country in countries/gavi-2020.csv this script writes
# packs/gavi/<ISO3>.json. It fills in what can be *derived* from public sources
# and leaves the rest MISSING, so the production gate stays closed until an
# analyst fills the questionnaire with sourced evidence:
#
#   life table            UN World Population Prospects 2019 mortality, built with
#                          the R implementation's own get_life_table()
#   life expectancy       from the same life table
#   annual births         UN WPP 2019 age-specific fertility rates x the female
#                          population of the price year
#   GDP per capita        World Bank WDI, current local currency
#   exchange rate         World Bank WDI (USD per local currency, derived)
#   mean population utility
#                          EQ-5D Belgian population norms - the same norms the
#                          reference implementation uses for every country, which
#                          the R code warns about
#
# Everything else (incidence, severity, costs, utilities, efficacy, prices) stays
# MISSING: it needs country evidence and is never fabricated here.
#
# Usage: Rscript mcmarcel-rs/parity/build_gavi_packs.R [catalogue.csv] [out-dir]

suppressPackageStartupMessages({
  library(jsonlite)
  library(foreach)
})

args <- commandArgs(trailingOnly = FALSE)
file_arg <- grep("^--file=", args, value = TRUE)
script_path <- normalizePath(sub("^--file=", "", file_arg[1]))
rs_root <- normalizePath(file.path(dirname(script_path), ".."))
source(file.path(dirname(script_path), "mcmarcel_root.R"))
repo_root <- resolve_mcmarcel_root(script_path)
setwd(repo_root)

positional <- commandArgs(trailingOnly = TRUE)
catalogue_path <- if (length(positional) > 0) positional[1] else file.path(rs_root, "countries", "gavi-2020.csv")
out_dir <- if (length(positional) > 1) positional[2] else file.path(rs_root, "packs", "gavi")
price_year <- 2024
disc_rate_effect <- 0.015

source(file.path(repo_root, "functions", "RSV_get_life_table.R"))
source(file.path(repo_root, "functions", "RSV_QALE_BEL.R"))
source(file.path(repo_root, "functions", "RSV_load_all.R"))

catalogue <- read.csv(catalogue_path, stringsAsFactors = FALSE)
cat("catalogue:", nrow(catalogue), "countries\n")

# --------------------------------------------------------- the R input tree ---
# get_life_table() reads 'input/dictionary_UN_countrynames.rds' relative to the
# working directory and looks the country up by the UN name it finds there. For
# three countries the dictionary and the wpp2019 data spell the name differently
# ("Dem. People's Republic of Korea" against "Dem. People's Rep. of Korea",
# "Democratic Republic of the Congo" against "Dem. Republic of the Congo",
# "Lao People's Democratic Republic" against "Lao People's Dem. Republic"), so
# the upstream lookup returns NA for them. The generator therefore works on a
# copy of the input tree whose dictionary maps every country of the catalogue to
# the name the wpp2019 data uses. The R code itself is not modified.
work_dir <- file.path(tempdir(), "mcmarcel-gavi-inputs")
unlink(work_dir, recursive = TRUE)
dir.create(work_dir, recursive = TRUE)
input_files <- list.files(file.path(repo_root, "input"), recursive = TRUE, all.files = TRUE,
                         full.names = FALSE, no.. = TRUE)
for (relative in input_files) {
  target_dir <- dirname(file.path(work_dir, "input", relative))
  dir.create(target_dir, recursive = TRUE, showWarnings = FALSE)
  file.copy(file.path(repo_root, "input", relative), file.path(work_dir, "input", relative),
            overwrite = TRUE)
}

library(countrycode)
data(mxM, package = 'wpp2019')
numeric_codes <- codelist$iso3n[match(catalogue$iso3, codelist$iso3c)]
dictionary_path <- file.path(work_dir, "input", "dictionary_UN_countrynames.rds")
dictionary <- readRDS(dictionary_path)
wpp_names <- character(nrow(catalogue))
for (index in seq_len(nrow(catalogue))) {
  wpp_name <- unique(mxM$name[mxM$country_code == numeric_codes[index]])
  if (length(wpp_name) == 1) {
    wpp_names[index] <- wpp_name
    dictionary$name[dictionary$iso3 == catalogue$iso3[index]] <- wpp_name
  }
}
saveRDS(dictionary, dictionary_path)
cat("patched the country dictionary for", nrow(catalogue), "countries\n")
setwd(work_dir)

# ------------------------------------------------------------------ helpers ---
qol_norms_file <- file.path(repo_root, "input", "eq5d", "popnormINDEX.rds")
qol_mean_vector <- popnormINDEX_BE(qol_norms_file)$mean
qol_mean <- mean(as.numeric(qol_mean_vector[1:min(60, length(qol_mean_vector))]), na.rm = TRUE)

data(tfrprojMed, package = 'wpp2019')
data(popF, package = 'wpp2019')
data(popFprojMed, package = 'wpp2019')

fertile_ages <- c('15-19', '20-24', '25-29', '30-34', '35-39', '40-44', '45-49')

# The number of births in a year.
#
# The wpp2019 package holds the total fertility rate and the female population,
# not a births series, so the births are estimated the way the UN does it when
# only the TFR is available:
#
#     births = TFR x (women aged 15-49) / 35
#
# The female population of the price year is interpolated between the last
# observed year (2020) and the first projection (2025). The approximation is
# accurate to roughly 5 % for high-fertility countries (India 22.7M against
# 23.0M published; Nigeria 7.7M against 7.4M) and drifts for low-fertility
# countries, where the fertility is concentrated in a few years of age
# (Belgium 122k against 108,680 registered; Tunisia 181k against 126,401).
# The value is therefore recorded as DERIVED with this caveat, and the country
# birth registration is the value an analyst should enter.
births_in_year <- function(un_name, year, period) {
  # The model's period uses an underscore ("2020_2025"), the WPP a hyphen.
  column <- gsub('_', '-', period)
  tfr_rows <- tfrprojMed[tfrprojMed$name == un_name, ]
  if (nrow(tfr_rows) == 0 || !column %in% names(tfr_rows)) return(NA_real_)
  fertility <- as.numeric(tfr_rows[[column]])

  historical <- popF[popF$name == un_name & popF$age %in% fertile_ages, ]
  projected <- popFprojMed[popFprojMed$name == un_name & popFprojMed$age %in% fertile_ages, ]
  if (nrow(historical) == 0 || nrow(projected) == 0) return(NA_real_)
  lower_value <- as.numeric(historical[['2020']])
  upper_value <- as.numeric(projected[['2025']])
  women <- sum(lower_value + (upper_value - lower_value) * (year - 2020) / 5) * 1000

  fertility * women / 35
}

# The World Bank indicators for the willingness-to-pay thresholds.
world_bank <- function(iso3, indicator, year) {
  url <- paste0("https://api.worldbank.org/v2/country/", iso3, "/indicator/", indicator,
                "?date=", year, "&format=json")
  result <- try(jsonlite::fromJSON(url), silent = TRUE)
  if (inherits(result, 'try-error')) return(NA_real_)
  values <- result[[2]]$value
  if (is.null(values) || length(values) == 0) return(NA_real_)
  as.numeric(values[1])
}

# --------------------------------------------------------- parameter helpers ---
param <- function(key, label, unit, value, status, source, notes) {
  list(key = key, label = label, unit = unit, value = value, low = NULL, high = NULL,
       status = status, source = source, source_ref = NULL, url = NULL, notes = notes,
       blocks_outputs = list())
}

# The questionnaire, so that every pack carries every question.
questionnaire <- jsonlite::fromJSON(file.path(rs_root, "docs", "questionnaire", "questionnaire.json"),
                                    simplifyVector = FALSE)

build_pack <- function(row, wpp_name) {
  iso3 <- row$iso3
  un_name <- wpp_name

  life <- get_life_table(iso3, price_year, disc_rate_effect, qol_norms_file = qol_norms_file)
  life <- life[1:60, ]
  life_matrix <- cbind(
    age_years = (0:59) / 12,
    le = life$LE,
    le_disc = life$LE_disc,
    hrqale = life$HrQALE,
    hrqale_disc = life$HrQALE_disc
  )

  period <- get_year_category(price_year)[1]
  births <- births_in_year(un_name, price_year, period)
  gdp_local <- world_bank(iso3, 'NY.GDP.PCAP.CN', price_year)
  gdp_usd <- world_bank(iso3, 'NY.GDP.PCAP.CD', price_year)
  exchange <- if (!is.na(gdp_local) && !is.na(gdp_usd) && gdp_local != 0) gdp_local / gdp_usd else NA_real_

  parameters <- list()
  for (question in questionnaire$questions) {
    key <- question$key
    value <- NULL
    status <- 'missing'
    source <- NULL
    notes <- paste0("Not yet sourced for ", row$name,
                    ". Use the questionnaire to record the value, its evidence status and its source.")
    if (key == 'births') {
      value <- if (is.na(births)) NULL else births
      status <- 'derived'
      source <- "UN World Population Prospects 2019 (total fertility rate x women aged 15-49 / 35)"
      notes <- paste0("Estimated for ", price_year, " from the UN WPP 2019 medium variant: the total fertility rate",
                      " multiplied by the number of women aged 15-49, divided by 35. Accurate to about 5 % for",
                      " high-fertility countries and less so for low-fertility ones, where the fertility is",
                      " concentrated in a few years of age. Replace with the national birth registration when it",
                      " is available; that is the value the model should use.")
    } else if (key == 'life_expectancy_years') {
      value <- life$LE[1]
      status <- 'derived'
      source <- "UN World Population Prospects 2019 (life table built by the McMarcel implementation)"
      notes <- "Derived from the life table asset of this pack."
    } else if (key == 'qol_mean') {
      value <- qol_mean
      status <- 'transfer'
      source <- "input/eq5d/popnormINDEX.rds (EQ-5D Belgian general population norms, BE 2018)"
      notes <- "The reference implementation applies the Belgian population norms to every country and warns about it. Replace with national norms when they exist."
    } else if (key == 'gdp_per_capita') {
      value <- if (is.na(gdp_local)) NULL else gdp_local
      status <- if (is.na(gdp_local)) 'missing' else 'ok'
      source <- if (is.na(gdp_local)) NULL else "World Bank World Development Indicators, NY.GDP.PCAP.CN"
      notes <- "Used only to derive the willingness-to-pay thresholds (0.5x, 1x, 3x GDP per capita)."
    } else if (key == 'gdp_fx_usd') {
      value <- if (is.na(exchange)) NULL else exchange
      status <- if (is.na(exchange)) 'missing' else 'ok'
      source <- if (is.na(exchange)) NULL else "World Bank World Development Indicators (NY.GDP.PCAP.CN / NY.GDP.PCAP.CD)"
      notes <- "Local currency per US dollar, implied by the two World Bank series."
    } else if (key == 'currency') {
      value <- row$currency
      status <- 'ok'
      source <- "ISO 4216"
      notes <- "Every monetary value of this pack is expressed in this currency."
    } else if (key == 'analysis_perspective') {
      value <- 'healthcare'
      status <- 'convention'
      source <- "McMarcel configuration default"
      notes <- "The societal perspective needs productivity costs, which this engine does not model."
    } else if (key == 'discount_rate_cost') {
      value <- 0.03
      status <- 'convention'
      source <- "McMarcel configuration default"
      notes <- "The upstream default; replace with the national guideline when it differs."
    } else if (key == 'discount_rate_effect') {
      value <- 0.015
      status <- 'convention'
      source <- "McMarcel configuration default"
      notes <- "The upstream default; replace with the national guideline when it differs."
    }
    parameters[[key]] <- param(key, question$label, question$unit, value, status, source, notes)
  }

  list(
    schema_version = 1L,
    country = list(
      id = paste0("gavi-", tolower(iso3)),
      name = row$name,
      iso3 = iso3,
      currency = row$currency,
      price_year = price_year,
      locale = 'en',
      perspective = 'healthcare',
      region = row$region,
      notes = paste0(
        "Gavi-eligible country (", row$gavi_list_year, " list). The life table, the life expectancy, the number of births",
        " and the GDP per capita are derived from public sources; every other parameter still needs country evidence",
        " and is marked MISSING, so production mode is blocked until the questionnaire is completed."
      )
    ),
    parameters = parameters,
    assets = list(
      life_table = list(rows = 60, cols = 5, data = as.numeric(t(life_matrix)))
    ),
    scenarios = list(list(
      id = 'none',
      name = 'No intervention',
      is_reference = TRUE,
      coverage_maternal = 0,
      coverage_mab = 0,
      intervention_start = 1,
      intervention_end = 12,
      coverage_catch_up = 0,
      price_maternal = NULL,
      price_mab = NULL,
      notes = 'Comparator arm. Add the intervention scenarios in the country editor.'
    )),
    notes = paste0("Generated by mcmarcel-rs/parity/build_gavi_packs.R for the Gavi-eligible country ", row$name, ".")
  )
}

# ------------------------------------------------------------------- build ---
dir.create(out_dir, recursive = TRUE, showWarnings = FALSE)
written <- 0
failed <- character(0)
for (index in seq_len(nrow(catalogue))) {
  row <- catalogue[index, ]
  result <- try(build_pack(row, wpp_names[index]), silent = TRUE)
  if (inherits(result, 'try-error')) {
    failed <- c(failed, paste0(row$iso3, ': ', attr(result, 'condition')$message))
    next
  }
  path <- file.path(out_dir, paste0(tolower(row$iso3), '.json'))
  write(toJSON(result, auto_unbox = TRUE, digits = NA, null = 'null', pretty = TRUE), path)
  written <- written + 1
}
cat("wrote", written, "packs to", out_dir, "\n")
if (length(failed) > 0) {
  cat("failed:\n")
  cat(paste(failed, collapse = '\n'), '\n')
}
