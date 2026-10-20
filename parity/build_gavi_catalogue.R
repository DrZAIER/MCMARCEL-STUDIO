#!/usr/bin/env Rscript
#
# Build the Gavi country catalogue for McMarcel Studio.
#
# The list is the set of countries eligible for Gavi support in 2020, taken from
# the Gavi eligibility page (archived by the Internet Archive on 15 August 2020,
# https://www.gavi.org/types-support/sustainability/eligibility). The McMarcel
# README refers to the 73 countries Gavi supported in 2018; the eligibility list
# is the set that could still apply for support at the time of the McMarcel
# analyses, and it is the set a country pack should be built for.
#
# Every name is resolved to an ISO3 code, an ISO 4217 currency code and the UN
# name used by the wpp2019 data, which is what the R implementation uses to
# build a life table.
#
# Usage: Rscript mcmarcel-rs/parity/build_gavi_catalogue.R [out.csv]

suppressPackageStartupMessages(library(countrycode))

out_path <- commandArgs(trailingOnly = TRUE)[1]
if (is.na(out_path)) out_path <- "countries/gavi-2020.csv"

# The eligibility list, in the order of the source page.
names_gavi <- c(
  "Afghanistan", "Bangladesh", "Benin", "Burkina Faso", "Burundi", "Cambodia", "Cameroon",
  "Central African Republic", "Chad", "Comoros", "Cote d'Ivoire", "Democratic Republic of Korea",
  "Democratic Republic of the Congo", "Djibouti", "Eritrea", "Ethiopia", "Gambia", "Ghana",
  "Guinea", "Guinea Bissau", "Haiti", "India", "Kenya", "Kyrgyzstan",
  "Lao People's Democratic Republic", "Lesotho", "Liberia", "Madagascar", "Malawi", "Mali",
  "Mauritania", "Mozambique", "Myanmar", "Nepal", "Nicaragua", "Niger", "Nigeria", "Pakistan",
  "Papua New Guinea", "Republic of Congo", "Republic of Sudan", "Rwanda", "Sao Tome and Principe",
  "Senegal", "Sierra Leone", "Solomon Islands", "Somalia", "South Sudan", "Syria", "Tajikistan",
  "Tanzania", "Togo", "Uganda", "Uzbekistan", "Vietnam", "Yemen", "Zambia", "Zimbabwe"
)

# Names the automatic resolution cannot handle, or that the Gavi page spells
# differently.
overrides <- c(
  "Cote d'Ivoire" = "CIV",
  "Democratic Republic of Korea" = "PRK",
  "Democratic Republic of the Congo" = "COD",
  "Guinea Bissau" = "GNB",
  "Lao People's Democratic Republic" = "LAO",
  "Republic of Congo" = "COG",
  "Republic of Sudan" = "SDN",
  "Sao Tome and Principe" = "STP",
  "Myanmar" = "MMR",
  "Tanzania" = "TZA",
  "Guinea" = "GIN"
)

iso3 <- vapply(names_gavi, function(name) {
  if (name %in% names(overrides)) return(unname(overrides[name]))
  code <- countrycode(name, 'country.name', 'iso3c', warn = FALSE)
  if (is.na(code)) NA_character_ else code
}, character(1), USE.NAMES = FALSE)

unresolved <- names_gavi[is.na(iso3)]
if (length(unresolved) > 0) {
  stop("could not resolve: ", paste(unresolved, collapse = ', '))
}

lookup <- function(column) {
  value <- codelist[[column]][match(iso3, codelist$iso3c)]
  value[is.na(value)] <- ""
  value
}

rows <- data.frame(
  iso3 = iso3,
  name = names_gavi,
  un_name = lookup("un.name.en"),
  currency = lookup("iso4217c"),
  region = lookup("region23"),
  gavi_status = "eligible",
  gavi_list_year = 2020L,
  stringsAsFactors = FALSE
)

dir.create(dirname(out_path), recursive = TRUE, showWarnings = FALSE)
write.csv(rows, out_path, row.names = FALSE, fileEncoding = "UTF-8")
cat("wrote", out_path, "with", nrow(rows), "countries\n")
print(rows, row.names = FALSE)
