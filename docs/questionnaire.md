# The country-pack questionnaire

McMarcel Studio does not ask an analyst to guess which numbers a country needs. The
**questionnaire** is the explicit contract: every parameter of a country pack, with its
definition, its unit, where a defensible value comes from and how its evidence has to be
classified.

The questionnaire is **independent of any country**. It describes the questions, not the
answers. The Belgian reference pack and the synthetic demo are examples of completed
questionnaires.

## Two ways to use it

### 1. The guided wizard in the interface

`Build a pack` in the sidebar opens the questionnaire. It walks through seven sections —
demography, epidemiology, costs, QALY losses, the maternal vaccine, the monoclonal antibody
and the analysis conventions — and for every question it shows:

* the **definition** of the parameter, in plain language;
* **why it matters**: what the model does with it, and what changes if it is wrong;
* **how to source it**: the kind of data set that carries the value;
* the **known sources** of the reference implementation, when they exist;
* the **evidence guidance**: which status fits which kind of source;
* the input for the value, the evidence status, the source and a reviewer note.

The rail on the left shows the progress per section, the number of answered questions and the
number of blocking ones. When the pack is created, every parameter that was not answered stays
`MISSING`, so nothing is silently invented: production mode stays blocked until each parameter
carries a value, an evidence status and a source.

### 2. The standalone toolkit

The same content is exported as a printable Markdown document and a spreadsheet-friendly CSV:

```bash
cd mcmarcel-rs
cargo run --release -p mcmarcel-cli -- questionnaire --out docs/questionnaire
```

| File | Use |
|------|-----|
| `docs/questionnaire/questionnaire.md` | Print or share: every question with a definition, its unit, its range, the sourcing guidance and a table to fill in |
| `docs/questionnaire/questionnaire.csv` | Fill in in a spreadsheet; the columns `value`, `evidence_status`, `source` and `reviewer_note` are meant to be filled |
| `docs/questionnaire/questionnaire.json` | The machine-readable catalogue, identical to the one the interface uses |

The three files are generated from the same catalogue as the wizard
(`crates/mcmarcel-core/src/questionnaire.rs`), so they can never drift apart. The test suite
checks that the catalogue still covers every parameter of the demo pack.

## The sections

| Section | Questions | What it decides |
|---------|----------|-----------------|
| 1 Demography | 5 | The size and shape of the birth cohort |
| 2 Epidemiology | 11 | Incidence, severity, seasonality and age profile |
| 3 Health system costs | 10 | The cost of one case of each type, with uncertainty |
| 4 QALY losses | 8 | The quality of life lost per case, with uncertainty |
| 5 Maternal RSV vaccine | 17 | Efficacy, duration, coverage, price and delivery |
| 6 Monoclonal antibody | 17 | Efficacy, duration, coverage, price and delivery |
| 7 Analysis conventions | 9 | Perspective, discounting and the willingness-to-pay thresholds |

## How to fill it in

1. **Start from a blank pack, not the demo.** The synthetic demo exists to show the interface;
   the Belgian reference pack exists to validate the model. Neither is a country.
2. **Work through the sections in order.** Demography and epidemiology decide whether the
   burden is right; costs and utilities decide whether the money is right.
3. **Give every value a source.** A number without a source is `MISSING` as far as the engine is
   concerned, and it blocks production mode.
4. **Classify the evidence honestly.** `TRANSFER` is not a failure: a value borrowed from
   another country, explicitly flagged, is a legitimate answer for a parameter that is not
   available locally. What is not legitimate is presenting a borrowed or synthetic value as if
   it were observed.
5. **Record the derivation.** A `DERIVED` value needs its derivation written down in the
   reviewer note, not just in your head.
6. **Leave the optional parameters alone** unless you have a value: they are only used when the
   engine has to generate an asset.

## Two parameters that need a word of warning

* **`cost_wheezing` and `qaly_loss_wheezing`** belong to the recurrent-wheezing module of the
  original framework. That module is **not implemented** in this engine version, so these
  values are not applied to the results. The validation report says so explicitly
  (`model.wheezing.not_implemented`) whenever they are not zero. Set them to zero to record
  that the omission is deliberate.
* **The summary scalars** (`incidence_hospital_0_5m`, `ratio_outpatient_hospital`,
  `cfr_hospital`, ...) are what the engine needs when a pack does not ship the corresponding
  age-by-month matrix. If you *do* have country matrices, add them as pack assets; the scalar
  then becomes a summary of the matrix rather than the source of the data. The validator only
  asks for the scalar when the asset is absent.

## The evidence statuses

| Status | Meaning | Blocks production |
|--------|---------|--------------------|
| `OK` | Direct, reviewed country evidence with a citation | No |
| `DERIVED` | Calculated from documented country inputs; the derivation is recorded | No |
| `TRANSFER` | Borrowed from another setting and explicitly flagged | No |
| `CONVENTION` | A modelling choice rather than an empirical estimate | No |
| `MISSING` | No defensible value available | **Yes** |
| `SYNTHETIC` | Demo or fixture data | **Yes** |

A pack with `MISSING` or `SYNTHETIC` evidence cannot run in production mode, and every run
made with one is stamped `SYNTHETIC FIXTURE — NOT FOR POLICY USE`. That rule is enforced by
the engine and by the API, not by the interface.
