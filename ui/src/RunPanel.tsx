import { useEffect, useState } from 'react';
import type { CountryListItem, CountryRecord, RunMode, ValidationReport } from './types';
import { formatNumber } from './format';
import { Button, Card, EmptyState, Field, Icon, StatusPill } from './ui';

export function RunPanel({
  countries,
  country,
  validation,
  busy,
  onSelectCountry,
  onRun,
  onGoCountries,
}: {
  countries: CountryListItem[];
  country: CountryRecord | null;
  validation: ValidationReport | null;
  busy: string | null;
  onSelectCountry: (id: string) => void;
  onRun: (input: { draws: number; seed: number; mode: RunMode }) => void;
  onGoCountries: () => void;
}) {
  const [draws, setDraws] = useState(500);
  const [seed, setSeed] = useState(20260120);
  const [mode, setMode] = useState<RunMode>('fixture');

  useEffect(() => {
    if (country) setMode(country.mode);
  }, [country]);

  if (!country) {
    return (
      <Card>
        <EmptyState icon="play" title="Choose a country pack" action={<Button onClick={onGoCountries}>Open countries</Button>}>
          Select a country and at least one intervention scenario before running the engine.
        </EmptyState>
      </Card>
    );
  }

  const blockers = validation?.issues.filter((issue) => issue.blocks_production) ?? [];
  const productionBlocked = mode === 'production' && !(validation?.production_ready ?? false);
  const isBusy = busy === 'run';
  const interventionScenarios = country.pack.scenarios.filter((scenario) => !scenario.is_reference);

  return (
    <div className="run-layout">
      <div className="stack">
        <Card className="run-country">
          <div className="run-country-head">
            <div className="country-avatar large">{country.pack.country.iso3}</div>
            <div>
              <span className="eyebrow">Selected pack</span>
              <h2>{country.pack.country.name}</h2>
              <p>{country.pack.country.currency} · price year {country.pack.country.price_year} · {country.pack.country.perspective} perspective</p>
            </div>
            <select value={country.id} onChange={(event) => onSelectCountry(event.target.value)}>
              {countries.map((item) => <option key={item.id} value={item.id}>{item.name} · {item.iso3}</option>)}
            </select>
          </div>
          <div className="run-scenario-strip">
            <div>
              <span className="mini-label">Reference</span>
              <strong>No intervention</strong>
            </div>
            {interventionScenarios.map((scenario) => (
              <div key={scenario.id}>
                <span className="mini-label">Intervention</span>
                <strong>{scenario.name}</strong>
                <span className="table-sub">{formatNumber(Math.max(scenario.coverage_maternal, scenario.coverage_mab) * 100, 1)}% coverage</span>
              </div>
            ))}
          </div>
        </Card>

        <Card>
          <div className="card-head">
            <div>
              <span className="eyebrow">Run configuration</span>
              <h3>Probabilistic settings</h3>
            </div>
            <StatusPill status={mode} label={mode === 'production' ? 'Production' : 'Fixture'} />
          </div>
          <div className="form-grid two">
            <Field label="Draws" hint="Paired Monte Carlo draws. 500 is a fast preview.">
              <input type="number" min={1} max={20000} value={draws} onChange={(event) => setDraws(Math.max(1, Math.min(20000, Number(event.target.value))))} />
            </Field>
            <Field label="Seed" hint="Reproducible integer seed.">
              <input type="number" value={seed} onChange={(event) => setSeed(Number(event.target.value))} />
            </Field>
            <Field label="Run mode" hint="Production requires all gates to be sourced." className="span-2">
              <div className="segmented">
                <button className={mode === 'fixture' ? 'active' : ''} onClick={() => setMode('fixture')}>Fixture</button>
                <button className={mode === 'production' ? 'active' : ''} onClick={() => setMode('production')}>Production</button>
              </div>
            </Field>
          </div>

          {productionBlocked && (
            <div className="banner banner-warn">
              <Icon name="warning" />
              <div>
                <strong>Production run blocked.</strong>
                <p>Resolve the {blockers.length} evidence gates in the country editor, or switch back to fixture.</p>
              </div>
            </div>
          )}

          <div className="run-actions">
            <Button icon="play" onClick={() => onRun({ draws, seed, mode })} disabled={isBusy || productionBlocked}>
              {isBusy ? 'Running engine…' : `Run ${formatNumber(draws)} draws`}
            </Button>
            <span className="run-hint">Results persist to the local SQLite store.</span>
          </div>
        </Card>
      </div>

      <div className="stack">
        <Card className="run-summary-card">
          <span className="eyebrow">What the engine will do</span>
          <h3>Model contract</h3>
          <ul className="check-list">
            <li><Icon name="check" size={16} /> Build 60-month cohort incidence from the country pack.</li>
            <li><Icon name="check" size={16} /> Sample efficacy, costs, utilities and prices with paired draws.</li>
            <li><Icon name="check" size={16} /> Apply maternal and mAb protection by birth cohort.</li>
            <li><Icon name="check" size={16} /> Compute averted cases, deaths, costs and discounted QALYs.</li>
            <li><Icon name="check" size={16} /> Produce ICER, INMB and price-threshold tables.</li>
          </ul>
        </Card>

        <Card className={validation?.production_ready ? 'gate-card ready' : 'gate-card'}>
          <div className="gate-head">
            <div className="empty-icon small"><Icon name={validation?.production_ready ? 'shield' : 'warning'} size={20} /></div>
            <div>
              <span className="eyebrow">Evidence gate</span>
              <h3>{validation?.production_ready ? 'Production-ready' : 'Fixture-only pack'}</h3>
            </div>
          </div>
          <p>
            {validation?.production_ready
              ? 'All gated parameters carry an accepted evidence status. The run can be labelled production.'
              : 'Some parameters are missing, synthetic or unsupported. Fixture runs are stamped SYNTHETIC FIXTURE — NOT FOR POLICY USE.'}
          </p>
          <div className="gate-stats">
            <div><strong>{validation?.issues.length ?? 0}</strong><span>checks</span></div>
            <div><strong>{blockers.length}</strong><span>blockers</span></div>
            <div><strong>{validation?.fixture_stamp ? 'Yes' : 'No'}</strong><span>stamp</span></div>
          </div>
          <Button variant="secondary" icon="shield" onClick={onGoCountries}>Review evidence</Button>
        </Card>

        <Card className="note-card">
          <span className="eyebrow">Scope note</span>
          <p>
            This first engine release covers the static monthly cohort core and probabilistic CEA. Dynamic transmission,
            EVPI/EVPPI, budget-impact, caregiver and recurrent-wheezing modules are intentionally deferred.
          </p>
        </Card>
      </div>
    </div>
  );
}
