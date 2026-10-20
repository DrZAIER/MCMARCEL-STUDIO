import type { CountryListItem, RunListItem } from './types';
import { formatDate, formatNumber, statusLabel } from './format';
import { Button, Card, EmptyState, Icon, Metric, StatusPill } from './ui';

export function Dashboard({
  countries,
  runs,
  selectedCountry,
  engineVersion,
  database,
  onGoCountries,
  onRun,
  onOpenRun,
}: {
  countries: CountryListItem[];
  runs: RunListItem[];
  selectedCountry: CountryListItem | null;
  engineVersion: string;
  database: string;
  onGoCountries: () => void;
  onRun: () => void;
  onOpenRun: (id: string) => void;
}) {
  const complete = runs.filter((run) => run.status === 'complete').length;
  const countriesWithData = countries.filter((country) => country.production_ready).length;
  const warnings = selectedCountry && !selectedCountry.production_ready ? 1 : 0;

  return (
    <div className="stack">
      <section className="hero">
        <div className="hero-copy">
          <span className="hero-kicker">Country-configurable RSV cohort modelling</span>
          <h2>From evidence to a defensible RSV decision — without spreadsheets.</h2>
          <p>
            McMarcel Studio runs a probabilistic monthly birth-cohort model, keeps every parameter’s
            provenance visible, and refuses production claims until the evidence is sourced.
          </p>
          <div className="hero-actions">
            <Button onClick={onRun} icon="play">Run a scenario</Button>
            <Button onClick={onGoCountries} variant="secondary" icon="globe">Configure a country</Button>
          </div>
        </div>
        <div className="hero-panel">
          <div className="hero-panel-head">
            <span>Engine snapshot</span>
            <span className="dot dot-live" />
          </div>
          <div className="hero-panel-grid">
            <div><strong>{engineVersion}</strong><span>Rust core</span></div>
            <div><strong>{countries.length}</strong><span>Country packs</span></div>
            <div><strong>{complete}</strong><span>Completed runs</span></div>
            <div><strong>WAL</strong><span>SQLite local store</span></div>
          </div>
          <div className="hero-database" title={database}>
            <Icon name="database" size={16} />
            <span>{database}</span>
          </div>
        </div>
      </section>

      <div className="metric-grid four">
        <Metric label="Countries" value={formatNumber(countries.length)} detail="Local packs" />
        <Metric label="Production-ready" value={formatNumber(countriesWithData)} detail="All evidence sourced" tone={countriesWithData ? 'good' : 'warn'} />
        <Metric label="Runs" value={formatNumber(runs.length)} detail={`${complete} complete`} />
        <Metric label="Open evidence flags" value={warnings ? 'Yes' : 'No'} detail={selectedCountry ? selectedCountry.name : 'Select a country'} tone={warnings ? 'warn' : 'good'} />
      </div>

      <div className="two-column">
        <Card className="recent-runs">
          <div className="card-head">
            <div>
              <span className="eyebrow">Recent activity</span>
              <h3>Runs</h3>
            </div>
            <Button variant="ghost" onClick={onRun} icon="play">New run</Button>
          </div>
          {runs.length === 0 ? (
            <EmptyState icon="chart" title="No runs yet" action={<Button onClick={onRun}>Run the demo</Button>}>
              The synthetic demo country is ready. Run it to see the full CEA output and evidence stamp.
            </EmptyState>
          ) : (
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>Country</th>
                    <th>Status</th>
                    <th>Draws</th>
                    <th>Created</th>
                    <th />
                  </tr>
                </thead>
                <tbody>
                  {runs.slice(0, 6).map((run) => (
                    <tr key={run.id} className="clickable" onClick={() => onOpenRun(run.id)}>
                      <td>
                        <strong>{run.country_name}</strong>
                        <span className="table-sub">{run.mode} · seed {run.seed}</span>
                      </td>
                      <td><StatusPill status={run.status} /></td>
                      <td className="mono">{formatNumber(run.draws)}</td>
                      <td className="muted">{formatDate(run.created_at_ms)}</td>
                      <td className="right"><Icon name="chevron" size={16} /></td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </Card>

        <Card className="side-card">
          <div className="card-head">
            <div>
              <span className="eyebrow">Evidence posture</span>
              <h3>{selectedCountry ? selectedCountry.name : 'No country selected'}</h3>
            </div>
            {selectedCountry && <StatusPill status={selectedCountry.production_ready ? 'ok' : 'synthetic'} label={selectedCountry.production_ready ? 'Production-ready' : 'Fixture only'} />}
          </div>
          {selectedCountry ? (
            <div className="evidence-list">
              <div><span>Parameters</span><strong>{formatNumber(selectedCountry.parameter_count)}</strong></div>
              <div><span>Scenarios</span><strong>{formatNumber(selectedCountry.scenario_count)}</strong></div>
              <div><span>Last updated</span><strong>{formatDate(selectedCountry.updated_at_ms)}</strong></div>
              <div className="evidence-callout">
                <Icon name="shield" size={18} />
                <p>
                  Production mode blocks any pack containing <code>MISSING</code> or <code>SYNTHETIC</code> evidence.
                  Demo runs are always stamped.
                </p>
              </div>
            </div>
          ) : (
            <EmptyState icon="globe" title="Create a country pack" action={<Button onClick={onGoCountries}>Open countries</Button>}>
              Start from the synthetic demo or a blank template, then replace every value with a documented source.
            </EmptyState>
          )}
        </Card>
      </div>

      <Card>
        <div className="card-head">
          <div>
            <span className="eyebrow">Workflow</span>
            <h3>From pack to policy question</h3>
          </div>
        </div>
        <div className="steps">
          <div className="step">
            <span className="step-num">01</span>
            <div><strong>Assemble the country pack</strong><p>Epidemiology, costs, utilities, life table and intervention prices.</p></div>
          </div>
          <div className="step">
            <span className="step-num">02</span>
            <div><strong>Review provenance</strong><p>Every parameter carries a status, source and production gate.</p></div>
          </div>
          <div className="step">
            <span className="step-num">03</span>
            <div><strong>Run probabilistic scenarios</strong><p>Paired draws, deterministic seeds and draw-level CE plane.</p></div>
          </div>
          <div className="step">
            <span className="step-num">04</span>
            <div><strong>Read the decision</strong><p>ICER, INMB, price thresholds and explicit uncertainty flags.</p></div>
          </div>
        </div>
      </Card>

      <Card className="about-mcmarcel">
        <div className="card-head">
          <div>
            <span className="eyebrow">The model behind this application</span>
            <h3>McMarcel</h3>
          </div>
          <StatusPill status="ok" label="CHERMID · University of Antwerp" />
        </div>
        <p className="prose">
          <strong>McMarcel</strong> — <em>Multi-Country Model Application for RSV Cost-Effectiveness
          Policy</em> — is the static monthly birth-cohort model of respiratory syncytial virus
          prevention built at CHERMID. It weighs maternal vaccination against monoclonal antibodies
          in a given country, and it was first applied to 72 Gavi-eligible countries.
          <strong>McMarcel Studio</strong> is the rewrite of that model: the same structure, a Rust
          engine, a guided interface and a production gate that fails closed.
        </p>
        <div className="about-releases">
          <div>
            <span className="mini-label">The original study</span>
            <strong>McMarcel for 72 Gavi countries</strong>
            <a href="https://doi.org/10.5281/zenodo.3663447" target="_blank" rel="noreferrer">
              doi:10.5281/zenodo.3663447
            </a>
            <span className="table-sub">
              12 February 2020 · CC BY 4.0 · Li et al., BMC Medicine 2020;18(1):82
            </span>
          </div>
          <div>
            <span className="mini-label">The version used here</span>
            <strong>MCMARCEL — Belgium</strong>
            <a href="https://doi.org/10.5281/zenodo.17992864" target="_blank" rel="noreferrer">
              doi:10.5281/zenodo.17992864
            </a>
            <span className="table-sub">
              version export_zenodo1 · 19 December 2025 · GPL-3.0 · Getaneh et al., Vaccine 2023;41(9):1623–1631
            </span>
          </div>
        </div>
        <div className="about-aims">
          <div>
            <span className="step-num">01</span>
            <p>
              <strong>Belgium</strong> — the reference case. The R implementation reproduces it
              byte-identically; the engine has to match it within a documented tolerance.
            </p>
          </div>
          <div>
            <span className="step-num">02</span>
            <p>
              <strong>Gavi</strong> — the 58 countries eligible in 2020 each get a pack with the life
              table, births, life expectancy and GDP derived, and everything else waiting for
              country evidence.
            </p>
          </div>
          <div>
            <span className="step-num">03</span>
            <p>
              <strong>New packs</strong> — the guided questionnaire turns a documented country into a
              pack with full provenance.
            </p>
          </div>
        </div>
      </Card>
    </div>
  );
}
