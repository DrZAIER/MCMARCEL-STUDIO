#!/usr/bin/env node
/**
 * Build the McMarcel Studio static site.
 *
 * The site is what can be hosted on GitHub Pages: the documentation, the
 * country-pack questionnaire (interactive, with a client-side export) and the
 * reference results of the Belgian case. The application itself is local-first
 * and needs the Rust engine, so it is downloaded or built, not served from here.
 *
 * Usage:
 *   node ui/scripts/build-site.mjs [--out site/dist]
 *
 * Inputs (produced by the CI workflow or by hand):
 *   docs/*.md                 the documentation
 *   docs/questionnaire/       the exported questionnaire toolkit
 *   site/content/belgium/     the engine run of the Belgian reference pack
 */
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync, copyFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { marked } from 'marked';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '..', '..');
const args = process.argv.slice(2);
const outIndex = args.indexOf('--out');
const outDir = resolve(root, outIndex >= 0 ? args[outIndex + 1] : 'site/dist');
const contentDir = resolve(root, 'site', 'content');

const read = (path) => readFileSync(resolve(root, path), 'utf8');
const readJson = (path) => JSON.parse(read(path));

console.log(`building the site into ${outDir}`);
mkdirSync(outDir, { recursive: true });

const markedOptions = { gfm: true, breaks: false };
const render = (markdown) => marked.parse(markdown, markedOptions);

/* ------------------------------------------------------------------ shell --- */
const nav = [
  ['index.html', 'Overview'],
  ['questionnaire.html', 'Questionnaire'],
  ['reference.html', 'Reference case'],
  ['docs/architecture.html', 'Architecture'],
  ['docs/model.html', 'Model'],
  ['docs/evidence-model.html', 'Evidence'],
  ['docs/pack-schema.html', 'Pack schema'],
  ['docs/questionnaire.html', 'Questionnaire guide'],
];

function shell({ title, eyebrow, body, depth = 0, extra = '' }) {
  const prefix = depth === 0 ? '' : '../';
  const links = nav
    .map(([href, label]) => {
      const base = depth === 0 ? href : href.replace(/^docs\//, '');
      return `<a href="${prefix}${base}">${label}</a>`;
    })
    .join('');
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${title} · McMarcel Studio</title>
<meta name="description" content="McMarcel Studio: a local-first application for country-configurable RSV burden and cost-effectiveness modelling.">
<link rel="stylesheet" href="${prefix}assets/site.css">
</head>
<body>
<header class="site-head">
  <a class="brand" href="${prefix}index.html"><span class="mark">M</span><span>McMarcel <em>Studio</em></span></a>
  <nav>${links}</nav>
</header>
<main>
${eyebrow ? `<p class="eyebrow">${eyebrow}</p>` : ''}
${body}
${extra}
</main>
<footer>
  <p>McMarcel Studio is free software under the GNU General Public License v3.0 or later. The R implementation of McMarcel is the validation oracle and is not part of this site.</p>
</footer>
</body>
</html>
`;
}

/* ------------------------------------------------------------------- docs --- */
const docsDir = resolve(root, 'docs');
const docFiles = readdirSync(docsDir).filter((name) => name.endsWith('.md'));
for (const name of docFiles) {
  const slug = name.replace(/\.md$/, '');
  const html = render(read(join('docs', name)));
  const target = slug === 'questionnaire' ? 'docs/questionnaire.html' : `docs/${slug}.html`;
  mkdirSync(join(outDir, 'docs'), { recursive: true });
  writeFileSync(join(outDir, target), shell({
    title: slug.replace(/-/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase()),
    eyebrow: 'Documentation',
    body: `<article class="prose">${html}</article>`,
    depth: 1,
  }));
  console.log(`  docs/${slug}.html`);
}

/* ------------------------------------------------------- questionnaire page --- */
const catalogue = readJson('docs/questionnaire/questionnaire.json');
const questionnaireScript = read('site/assets/questionnaire.js');
const questionnaireStyle = read('site/assets/questionnaire.css');
mkdirSync(join(outDir, 'assets'), { recursive: true });
writeFileSync(
  join(outDir, 'questionnaire.html'),
  shell({
    title: 'Country-pack questionnaire',
    eyebrow: 'Guided questionnaire',
    body: `<div id="questionnaire-app" data-catalogue-url="assets/questionnaire.json"></div>
<script id="questionnaire-data" type="application/json">${JSON.stringify(catalogue)}</script>
<script src="assets/questionnaire.js"></script>`,
    extra: `<style>${questionnaireStyle}</style>`,
  }),
);
writeFileSync(join(outDir, 'assets', 'questionnaire.js'), questionnaireScript);
console.log('  questionnaire.html');

/* --------------------------------------------------------- reference case --- */
const belgium = existsSync(join(contentDir, 'belgium', 'results.json'))
  ? readJson(join(contentDir, 'belgium', 'results.json'))
  : null;
const upstream = existsSync(resolve(root, 'parity', 'expected', 'belgium_reference.json'))
  ? readJson(resolve(root, 'parity', 'expected', 'belgium_reference.json'))
  : null;

function referenceBody() {
  if (!belgium) {
    return '<p>The reference run is not available in this build. Run <code>cargo run --release -p mcmarcel-cli -- run --pack packs/belgium-belx.json --out site/content/belgium --draws 500</code> to generate it.</p>';
  }
  const rows = belgium.scenarios
    .filter((scenario) => !scenario.is_reference)
    .map((scenario) => {
      const analysis = belgium.analysis.icer_table.find((row) => row.strategy_id === scenario.scenario_id);
      return `<tr>
        <td>${scenario.scenario_name}</td>
        <td class="num">${Math.round(scenario.mean_averted_cases).toLocaleString('en-US')}</td>
        <td class="num">${Math.round(scenario.mean_averted_hospital).toLocaleString('en-US')}</td>
        <td class="num">${scenario.mean_averted_deaths.toFixed(2)}</td>
        <td class="num">${scenario.mean_qaly_averted.toFixed(1)}</td>
        <td class="num">${Math.round(scenario.mean_incremental_cost).toLocaleString('en-US')}</td>
        <td>${analysis ? analysis.status : '—'}</td>
      </tr>`;
    })
    .join('');

  let comparison = '';
  if (upstream) {
    // The upstream fixture labels the interventions; the pack names the scenarios.
    const upstreamToScenario = {
      maternal: 'maternal JanDec',
      maternal_SepMar: 'maternal SepMar',
      mAb: 'mAb JanDec',
      mAb_OctMar: 'mAb OctMar',
      mAb_OctMar_CB: 'mAb OctMar_CB',
    };
    const byScenario = new Map(
      belgium.scenarios
        .filter((scenario) => !scenario.is_reference && scenario.scenario_name.endsWith('(basecase)'))
        .map((scenario) => [scenario.scenario_name.replace(/ \(.*\)$/, ''), scenario]),
    );
    const lines = upstream.scenarios
      .filter((entry) => entry.scenario === 'basecase')
      .map((entry) => {
        const actual = byScenario.get(upstreamToScenario[entry.intervention]);
        if (!actual) return '';
        const rust = actual.mean_qaly_averted;
        const diff = ((rust - entry.total_QALY_disc_averted) / entry.total_QALY_disc_averted) * 100;
        const cost = actual.mean_incremental_cost;
        const costDiff = ((cost - entry.incremental_cost_disc_program) / entry.incremental_cost_disc_program) * 100;
        return `<tr>
          <td>${entry.intervention}</td>
          <td class="num">${entry.total_QALY_disc_averted.toFixed(1)}</td>
          <td class="num">${rust.toFixed(1)}</td>
          <td class="num ${Math.abs(diff) < 5 ? 'good' : 'warn'}">${diff >= 0 ? '+' : ''}${diff.toFixed(1)} %</td>
          <td class="num">${Math.round(entry.incremental_cost_disc_program).toLocaleString('en-US')}</td>
          <td class="num">${Math.round(cost).toLocaleString('en-US')}</td>
          <td class="num ${Math.abs(costDiff) < 5 ? 'good' : 'warn'}">${costDiff >= 0 ? '+' : ''}${costDiff.toFixed(1)} %</td>
        </tr>`;
      })
      .filter(Boolean)
      .join('');
    comparison = `<h2>Against the upstream reference</h2>
      <p>The R implementation reproduces the Belgian base case byte-identically. This table compares the engine with
      the committed upstream fixture. The two use different random streams, so a small difference is expected; the
      regression test in the repository holds the engine to a documented tolerance.</p>
      <table class="data">
        <thead><tr><th>Intervention</th><th class="num">QALYs (R)</th><th class="num">QALYs (engine)</th><th class="num">Δ</th><th class="num">Cost (R)</th><th class="num">Cost (engine)</th><th class="num">Δ</th></tr></thead>
        <tbody>${lines}</tbody>
      </table>`;
  }

  return `<p class="lead">The Belgian base case is the reference of the McMarcel framework. The pack is built from the
  repository's Belgian input files and reproduces the upstream results within the tolerance of the regression test.</p>
  <div class="notice">${belgium.fixture_stamp ? `<strong>${belgium.fixture_stamp}</strong><br>` : ''}The public-release
  inputs carry stochastic perturbation, so these numbers validate the model; they do not describe Belgium.</div>
  <h2>The run</h2>
  <table class="data">
    <thead><tr><th>Scenario</th><th class="num">Cases averted</th><th class="num">Admissions averted</th><th class="num">Deaths averted</th><th class="num">QALYs averted</th><th class="num">Incremental cost</th><th>Status</th></tr></thead>
    <tbody>${rows}</tbody>
  </table>
  ${comparison}`;
}

writeFileSync(
  join(outDir, 'reference.html'),
  shell({
    title: 'Belgian reference case',
    eyebrow: 'Reference case',
    body: referenceBody(),
  }),
);
console.log('  reference.html');

/* --------------------------------------------------------------- index --- */
const indexBody = `
<p class="lead">McMarcel Studio is a local-first application for country-configurable RSV burden and
cost-effectiveness modelling: a Rust engine, a React interface, a local SQLite store and a
Tauri desktop shell. Every parameter carries its evidence, and the model refuses to present
synthetic numbers as a country estimate.</p>

<div class="grid">
  <div class="card">
    <h3>Use the application</h3>
    <p>The engine runs on your machine; the data never leaves it.</p>
    <pre><code>git clone https://github.com/DrZAIER/MCMARCEL-STUDIO.git
cd MCMARCEL-STUDIO
cargo build --release
cd ui &amp;&amp; npm install &amp;&amp; npm run build
cargo run --release -p mcmarcel-server</code></pre>
    <p class="muted">Then open <code>http://127.0.0.1:8787</code>, or build the desktop shell with
    <code>cargo build --release --manifest-path src-tauri/Cargo.toml</code>.</p>
  </div>
  <div class="card">
    <h3>Fill the questionnaire</h3>
    <p>Seventy-seven parameters in seven sections, each with its definition, its unit, how to source it and the
    evidence status that fits.</p>
    <p><a class="button" href="questionnaire.html">Open the questionnaire</a></p>
    <p class="muted">It works offline as well: <code>mcmarcel-cli questionnaire</code> writes a printable
    document and a spreadsheet.</p>
  </div>
  <div class="card">
    <h3>Read the methods</h3>
    <p>The cohort model, the evidence model, the pack schema and the known deviations from the R
    implementation.</p>
    <p><a class="button ghost" href="docs/model.html">Model</a>
       <a class="button ghost" href="docs/evidence-model.html">Evidence</a>
       <a class="button ghost" href="docs/architecture.html">Architecture</a></p>
  </div>
  <div class="card">
    <h3>Check the reference case</h3>
    <p>The Belgian base case, produced by the engine and compared with the upstream fixture.</p>
    <p><a class="button ghost" href="reference.html">Reference case</a></p>
  </div>
</div>

<h2>What this site is, and what it is not</h2>
<p>This site is the static part of McMarcel Studio: the documentation, the questionnaire toolkit and the
published reference results. The application itself is a local-first desktop program — the Rust engine, the
SQLite store and the production gate run on your machine, which is what keeps a country pack under the
analyst's control. That is also why a hosted copy of the engine is not offered here: a model whose inputs
leave the machine is a different product with different obligations.</p>
`;
writeFileSync(join(outDir, 'index.html'), shell({ title: 'Overview', body: indexBody }));
console.log('  index.html');

/* --------------------------------------------------------------- assets --- */
mkdirSync(join(outDir, 'assets'), { recursive: true });
copyFileSync(resolve(root, 'site', 'assets', 'site.css'), join(outDir, 'assets', 'site.css'));
if (existsSync(join(contentDir, 'belgium', 'results.json'))) {
  // Only the summaries are published: the draw-level points are large and the
  // site shows the aggregate results.
  const full = readJson(join(contentDir, 'belgium', 'results.json'));
  const trimmed = {
    pack_name: full.pack_name,
    pack_hash: full.pack_hash,
    mode: full.mode,
    draws: full.draws,
    seed: full.seed,
    fixture_stamp: full.fixture_stamp,
    scenarios: full.scenarios,
    analysis: full.analysis,
    validation: { production_ready: full.validation.production_ready, issue_count: full.validation.issues.length },
  };
  writeFileSync(join(outDir, 'assets', 'belgium-results.json'), JSON.stringify(trimmed));
}
if (existsSync(join(contentDir, 'questionnaire', 'questionnaire.json'))) {
  copyFileSync(join(contentDir, 'questionnaire', 'questionnaire.json'), join(outDir, 'assets', 'questionnaire.json'));
}
console.log('  assets/');
console.log('done');
