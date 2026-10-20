import { useEffect, useMemo, useState } from 'react';
import type { RunRecord } from './types';
import { formatCompact, formatDate, formatIcer, formatMoney, formatNumber, statusLabel } from './format';
import { Button, Card, EmptyState, Icon, IssueList, Metric, StatusPill } from './ui';
import { CEPlane } from './CEPlane';

export function Results({
  run,
  currency,
  onRunAgain,
  onGoCountries,
}: {
  run: RunRecord | null;
  currency: string;
  onRunAgain: () => void;
  onGoCountries: () => void;
}) {
  const result = run?.result ?? null;
  const interventions = result?.scenarios.filter((scenario) => !scenario.is_reference) ?? [];
  const [selectedId, setSelectedId] = useState<string | null>(null);

  useEffect(() => {
    if (interventions.length > 0) {
      setSelectedId((current) => current && interventions.some((scenario) => scenario.scenario_id === current) ? current : interventions[0].scenario_id);
    } else {
      setSelectedId(null);
    }
  }, [run?.id]);

  const selected = interventions.find((scenario) => scenario.scenario_id === selectedId) ?? interventions[0] ?? null;
  const icer = selected ? result?.analysis.icer_table.find((row) => row.strategy_id === selected.scenario_id) : null;
  const inmbRows = useMemo(() => {
    if (!result) return [];
    const thresholds = result.analysis.thresholds;
    return result.analysis.icer_table.map((row) => {
      const values = thresholds.map((threshold) => result.analysis.inmb_table.find((item) => item.strategy_id === row.strategy_id && Math.abs(item.threshold - threshold) < 1e-6)?.inmb ?? null);
      return { ...row, values };
    });
  }, [result]);

  if (!run || !result) {
    return (
      <Card>
        <EmptyState icon="chart" title="No result selected" action={<Button onClick={onRunAgain} icon="play">Run a scenario</Button>}>
          Run the model or open a completed run to explore the cost-effectiveness plane, ICER and price thresholds.
        </EmptyState>
      </Card>
    );
  }

  const fixture = result.fixture_stamp;
  const status = run.status;
  const reference = result.scenarios.find((scenario) => scenario.is_reference);
  const totalCasesAverted = selected?.mean_averted_cases ?? 0;

  return (
    <div className="stack">
      <Card className="results-head">
        <div className="results-title">
          <div>
            <span className="eyebrow">Run {run.id.slice(0, 8)} · {formatDate(run.created_at_ms)}</span>
            <h2>{result.pack_name}</h2>
            <div className="editor-badges">
              <StatusPill status={status} />
              <StatusPill status={result.mode} label={result.mode === 'production' ? 'Production' : 'Fixture'} />
              <span className="hash" title={result.pack_hash}>#{result.pack_hash.slice(0, 12)}</span>
              <span className="table-sub">{formatNumber(result.draws)} draws · seed {result.seed}</span>
            </div>
          </div>
          <div className="results-actions">
            <Button variant="ghost" onClick={() => downloadRun(run)} icon="save">Export JSON</Button>
            <Button variant="secondary" onClick={onRunAgain} icon="refresh">Run again</Button>
          </div>
        </div>
        {fixture && (
          <div className="banner banner-fixture">
            <Icon name="warning" />
            <div>
              <strong>{fixture}</strong>
              <p>These results are illustrative. They are not a country estimate and must not be used for policy, pricing or reimbursement decisions.</p>
            </div>
          </div>
        )}
      </Card>

      {selected && (
        <div className="metric-grid five">
          <Metric label="Cases averted" value={formatNumber(totalCasesAverted)} detail="All RSV outcomes" tone="good" />
          <Metric label="Hospitalisations averted" value={formatNumber(selected.mean_averted_hospital)} detail="Inpatient admissions" tone="good" />
          <Metric label="Deaths averted" value={formatNumber(selected.mean_averted_deaths, 2)} detail="RSV-attributable" tone="good" />
          <Metric label="QALYs averted" value={formatNumber(selected.mean_qaly_averted, 2)} detail="Discounted" tone="good" />
          <Metric label="Incremental cost" value={formatMoney(selected.mean_incremental_cost, currency)} detail={`ICER ${formatIcer(icer?.icer, currency)}`} tone={selected.mean_incremental_cost <= 0 ? 'good' : 'warn'} />
        </div>
      )}

      <div className="results-grid">
        <Card>
          <CEPlane run={result} currency={currency} />
        </Card>
        <Card>
          <div className="card-head">
            <div>
              <span className="eyebrow">Strategy comparison</span>
              <h3>Incremental results</h3>
            </div>
          </div>
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Strategy</th>
                  <th>Cost</th>
                  <th>QALYs</th>
                  <th>ICER</th>
                </tr>
              </thead>
              <tbody>
                {result.analysis.icer_table.map((row) => (
                  <tr key={row.strategy_id} className={row.strategy_id === selectedId ? 'selected-row' : ''}>
                    <td>
                      <strong>{row.strategy}</strong>
                      <span className="table-sub">{statusLabel(row.status)}</span>
                    </td>
                    <td className="mono">{formatMoney(row.cost, currency)}</td>
                    <td className="mono">{formatNumber(row.effect, 1)}</td>
                    <td className="mono">{row.icer === null || row.icer === undefined ? '—' : formatCompact(row.icer)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="scenario-switcher">
            {interventions.map((scenario) => (
              <button key={scenario.scenario_id} className={scenario.scenario_id === selectedId ? 'active' : ''} onClick={() => setSelectedId(scenario.scenario_id)}>
                {scenario.scenario_name}
              </button>
            ))}
          </div>
        </Card>
      </div>

      <div className="two-column">
        <Card>
          <div className="card-head">
            <div>
              <span className="eyebrow">Decision rule</span>
              <h3>Net monetary benefit</h3>
            </div>
          </div>
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Strategy</th>
                  {result.analysis.thresholds.map((threshold) => <th key={threshold}>{formatCompact(threshold)}</th>)}
                </tr>
              </thead>
              <tbody>
                {inmbRows.map((row) => (
                  <tr key={row.strategy_id}>
                    <td><strong>{row.strategy}</strong><span className="table-sub">{statusLabel(row.status)}</span></td>
                    {row.values.map((value, index) => (
                      <td key={index} className={`mono ${value !== null && value >= 0 ? 'positive' : 'negative'}`}>{value === null ? '—' : formatMoney(value, currency)}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="table-note">Thresholds are derived from GDP per capita and the pack’s WTP multipliers. INMB = WTP × QALYs averted − incremental cost.</p>
        </Card>

        <Card>
          <div className="card-head">
            <div>
              <span className="eyebrow">Price sensitivity</span>
              <h3>Threshold dose price</h3>
            </div>
          </div>
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Strategy</th>
                  <th>Current</th>
                  <th>P*</th>
                  <th>At WTP</th>
                </tr>
              </thead>
              <tbody>
                {result.analysis.pstar_table.map((row) => (
                  <tr key={row.strategy_id}>
                    <td><strong>{row.strategy}</strong><span className="table-sub">{formatNumber(row.doses)} doses</span></td>
                    <td className="mono">{formatMoney(row.current_price, currency)}</td>
                    <td className={`mono ${row.pstar !== null && row.pstar !== undefined && row.pstar > 0 ? 'positive' : 'negative'}`}>
                      {row.pstar === null || row.pstar === undefined ? '—' : formatMoney(row.pstar, currency)}
                    </td>
                    <td className="mono">{formatCompact(row.wtp)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="table-note">P* is the dose price at which INMB reaches zero at the central WTP threshold, holding other inputs at their sampled means.</p>
        </Card>
      </div>

      <Card>
        <div className="card-head">
          <div>
            <span className="eyebrow">Detailed summary</span>
            <h3>All scenarios</h3>
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Scenario</th>
                <th>Reference cases</th>
                <th>Averted cases</th>
                <th>Averted hospital</th>
                <th>Deaths averted</th>
                <th>Medical cost averted</th>
                <th>Intervention cost</th>
                <th>Incremental cost</th>
                <th>QALYs averted</th>
                <th>FVP</th>
              </tr>
            </thead>
            <tbody>
              {result.scenarios.map((scenario) => (
                <tr key={scenario.scenario_id} className={scenario.scenario_id === selectedId ? 'selected-row' : ''}>
                  <td><strong>{scenario.scenario_name}</strong><span className="table-sub">{scenario.is_reference ? 'Comparator' : `${formatNumber(Math.max(scenario.coverage_maternal, scenario.coverage_mab) * 100, 1)}% coverage`}</span></td>
                  <td className="mono">{formatNumber(scenario.mean_reference_cases)}</td>
                  <td className="mono">{formatNumber(scenario.mean_averted_cases)}</td>
                  <td className="mono">{formatNumber(scenario.mean_averted_hospital)}</td>
                  <td className="mono">{formatNumber(scenario.mean_averted_deaths, 2)}</td>
                  <td className="mono">{formatMoney(scenario.mean_medical_cost_averted, currency)}</td>
                  <td className="mono">{formatMoney(scenario.mean_intervention_cost, currency)}</td>
                  <td className="mono">{formatMoney(scenario.mean_incremental_cost, currency)}</td>
                  <td className="mono">{formatNumber(scenario.mean_qaly_averted, 2)}</td>
                  <td className="mono">{formatNumber(scenario.mean_fvp)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>

      <div className="two-column">
        <Card>
          <div className="card-head">
            <div>
              <span className="eyebrow">Audit</span>
              <h3>Validation and warnings</h3>
            </div>
            <StatusPill status={result.validation.production_ready ? 'ok' : 'synthetic'} label={result.validation.production_ready ? 'Ready' : 'Gated'} />
          </div>
          <div className="issue-list">
            <IssueList issues={result.validation.issues} limit={6} />
            {result.warnings.filter((warning) => !result.validation.issues.some((issue) => warning.includes(issue.code))).slice(0, 4).map((warning) => (
              <div key={warning} className="issue issue-warning">
                <StatusPill status="convention" label="warning" />
                <div><p>{warning}</p></div>
              </div>
            ))}
          </div>
        </Card>
        <Card>
          <div className="card-head">
            <div>
              <span className="eyebrow">Method</span>
              <h3>What this run includes</h3>
            </div>
          </div>
          <ul className="check-list">
            <li><Icon name="check" size={16} /> Static monthly birth-cohort RSV model over 0–59 months.</li>
            <li><Icon name="check" size={16} /> Paired probabilistic draws for efficacy, costs, QALYs and prices.</li>
            <li><Icon name="check" size={16} /> Discounted medical costs and health effects at pack rates.</li>
            <li><Icon name="check" size={16} /> Reference and intervention scenarios with explicit provenance gates.</li>
            <li className="muted"><span className="cross">×</span> Excluded: dynamic transmission, EVPI/EVPPI, budget impact, caregiver spillovers and the recurrent-wheezing module.</li>
          </ul>
          {reference && <p className="table-note">Reference burden: {formatNumber(reference.mean_reference_cases)} cases, {formatNumber(reference.mean_reference_qaly, 1)} discounted QALYs lost.</p>}
          <div className="run-foot-actions">
            <Button variant="secondary" onClick={onGoCountries} icon="globe">Edit country pack</Button>
            <Button onClick={onRunAgain} icon="play">New run</Button>
          </div>
        </Card>
      </div>
    </div>
  );
}

function downloadRun(run: RunRecord) {
  const blob = new Blob([JSON.stringify(run, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = `mcmarcel-run-${run.id.slice(0, 8)}.json`;
  anchor.click();
  URL.revokeObjectURL(url);
}
