import { useMemo, useState } from 'react';
import type {
  CountryListItem,
  CountryPack,
  CountryRecord,
  EvidenceStatus,
  Parameter,
  Scenario,
  ValidationReport,
} from './types';
import { formatDate, formatNumber, titleCase } from './format';
import { Button, Card, EmptyState, Field, Icon, IssueList, Modal, StatusPill } from './ui';

type EditorTab = 'parameters' | 'scenarios' | 'provenance';

const statuses: EvidenceStatus[] = ['ok', 'derived', 'transfer', 'convention', 'missing', 'synthetic'];

const groupOrder: Array<{ id: string; label: string; hint: string; match: (key: string) => boolean }> = [
  {
    id: 'epi',
    label: 'Epidemiology & demography',
    hint: 'Births, incidence, severity and age structure',
    match: (key) => /^(births|stillbirth|pre_term|incidence|ratio_|proportion_|icu_|cfr_|age_decay|seasonality|life_expectancy|qol_mean)/.test(key),
  },
  {
    id: 'cost',
    label: 'Health system costs',
    hint: 'Local currency per case or admission',
    match: (key) => key.startsWith('cost_'),
  },
  {
    id: 'qaly',
    label: 'QALY losses',
    hint: 'Utility loss per case and sequelae',
    match: (key) => key.startsWith('qaly_'),
  },
  {
    id: 'maternal',
    label: 'Maternal vaccine',
    hint: 'Efficacy, duration, coverage and price',
    match: (key) => key.includes('maternal'),
  },
  {
    id: 'mab',
    label: 'Monoclonal antibody',
    hint: 'Efficacy, duration, coverage and price',
    match: (key) => key.includes('mab') || key.includes('mAb'),
  },
  {
    id: 'analysis',
    label: 'Analysis conventions',
    hint: 'Discounting, WTP thresholds and perspective',
    match: (key) => /^(discount|gdp_|wtp_|analysis_perspective|currency)/.test(key),
  },
];

function groupFor(key: string): string {
  return groupOrder.find((group) => group.match(key))?.id ?? 'other';
}

export function Countries({
  countries,
  country,
  draft,
  validation,
  busy,
  dirty,
  onSelect,
  onParameterChange,
  onParameterMetaChange,
  onScenarioChange,
  onAddScenario,
  onRemoveScenario,
  onSave,
  onValidate,
  onCreate,
  onDelete,
  onRun,
}: {
  countries: CountryListItem[];
  country: CountryRecord | null;
  draft: CountryPack | null;
  validation: ValidationReport | null;
  busy: string | null;
  dirty: boolean;
  onSelect: (id: string) => void;
  onParameterChange: (key: string, value: number | string | null) => void;
  onParameterMetaChange: (key: string, field: 'status' | 'source' | 'notes', value: string) => void;
  onScenarioChange: (index: number, patch: Partial<Scenario>) => void;
  onAddScenario: (kind: 'maternal' | 'mab') => void;
  onRemoveScenario: (index: number) => void;
  onSave: () => void;
  onValidate: () => void;
  onCreate: (input: { name: string; iso3: string; template: 'demo' | 'blank' }) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onRun: () => void;
}) {
  const [tab, setTab] = useState<EditorTab>('parameters');
  const [newOpen, setNewOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [newName, setNewName] = useState('');
  const [newIso3, setNewIso3] = useState('');
  const [newTemplate, setNewTemplate] = useState<'demo' | 'blank'>('demo');
  const [creating, setCreating] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);

  const grouped = useMemo(() => {
    if (!draft) return [];
    const groups = new Map<string, Parameter[]>();
    for (const parameter of Object.values(draft.parameters)) {
      const id = groupFor(parameter.key);
      const list = groups.get(id) ?? [];
      list.push(parameter);
      groups.set(id, list);
    }
    const order = [...groupOrder.map((group) => ({ id: group.id, label: group.label, hint: group.hint })), { id: 'other', label: 'Other', hint: 'Additional configuration' }];
    return order
      .map((group) => ({ ...group, parameters: (groups.get(group.id) ?? []).sort((a, b) => a.key.localeCompare(b.key)) }))
      .filter((group) => group.parameters.length > 0);
  }, [draft]);

  const issueCount = validation?.issues.filter((issue) => issue.severity !== 'info').length ?? 0;
  const blockingCount = validation?.issues.filter((issue) => issue.blocks_production).length ?? 0;

  async function submitNew() {
    if (!newName.trim() || newIso3.trim().length !== 3) {
      setFormError('Enter a country name and a three-letter ISO3 code.');
      return;
    }
    setCreating(true);
    setFormError(null);
    try {
      await onCreate({ name: newName.trim(), iso3: newIso3.trim().toUpperCase(), template: newTemplate });
      setNewOpen(false);
      setNewName('');
      setNewIso3('');
    } catch (error) {
      setFormError(error instanceof Error ? error.message : String(error));
    } finally {
      setCreating(false);
    }
  }

  async function confirmDelete() {
    if (!country) return;
    setCreating(true);
    try {
      await onDelete(country.id);
      setDeleteOpen(false);
    } finally {
      setCreating(false);
    }
  }

  return (
    <div className="countries-layout">
      <aside className="country-list">
        <div className="card-head compact">
          <div>
            <span className="eyebrow">Local library</span>
            <h3>{countries.length} packs</h3>
          </div>
          <Button icon="plus" onClick={() => setNewOpen(true)} variant="secondary">New</Button>
        </div>
        <div className="country-cards">
          {countries.map((item) => (
            <button
              key={item.id}
              className={`country-card ${item.id === country?.id ? 'active' : ''}`}
              onClick={() => onSelect(item.id)}
            >
              <div className="country-card-top">
                <span className="country-code">{item.iso3}</span>
                <StatusPill status={item.production_ready ? 'ok' : 'synthetic'} label={item.production_ready ? 'Ready' : 'Fixture'} />
              </div>
              <strong>{item.name}</strong>
              <span className="country-meta">{item.scenario_count} scenarios · {item.parameter_count} parameters</span>
              <span className="country-meta">Updated {formatDate(item.updated_at_ms)}</span>
            </button>
          ))}
          {countries.length === 0 && (
            <EmptyState icon="globe" title="No country packs" action={<Button onClick={() => setNewOpen(true)}>Create one</Button>}>
              Start from the synthetic demo or a blank evidence template.
            </EmptyState>
          )}
        </div>
      </aside>

      <div className="editor">
        {!draft || !country ? (
          <Card>
            <EmptyState icon="globe" title="Select a country pack" action={<Button onClick={() => setNewOpen(true)}>Create a pack</Button>}>
              A pack contains the country metadata, evidence parameters, model inputs and scenarios.
            </EmptyState>
          </Card>
        ) : (
          <>
            <Card className="editor-head">
              <div className="editor-title">
                <div className="country-avatar">{draft.country.iso3}</div>
                <div>
                  <span className="eyebrow">{draft.country.region || 'Country pack'} · {draft.country.currency}</span>
                  <h2>{draft.country.name}</h2>
                  <div className="editor-badges">
                    <StatusPill status={country.mode} label={country.mode === 'production' ? 'Production mode' : 'Fixture mode'} />
                    <StatusPill status={validation?.production_ready ? 'ok' : 'synthetic'} label={validation?.production_ready ? 'Evidence complete' : `${blockingCount} blockers`} />
                    {dirty && <StatusPill status="warning" label="Unsaved changes" />}
                    <span className="hash" title={validation?.pack_hash}>#{validation?.pack_hash.slice(0, 12) ?? 'unvalidated'}</span>
                  </div>
                </div>
              </div>
              <div className="editor-actions">
                <Button variant="ghost" icon="trash" onClick={() => setDeleteOpen(true)}>Delete</Button>
                <Button variant="secondary" icon="shield" onClick={onValidate} disabled={busy === 'validate'}>
                  {busy === 'validate' ? 'Validating…' : 'Validate'}
                </Button>
                <Button variant="secondary" icon="save" onClick={onSave} disabled={busy === 'save'}>
                  {busy === 'save' ? 'Saving…' : 'Save pack'}
                </Button>
                <Button icon="play" onClick={onRun}>Run</Button>
              </div>
            </Card>

            {validation && (
              <div className={`validation-strip ${validation.production_ready ? 'good' : 'warn'}`}>
                <div className="validation-icon"><Icon name={validation.production_ready ? 'check' : 'warning'} size={20} /></div>
                <div>
                  <strong>{validation.production_ready ? 'This pack can run in production mode.' : 'Production mode is blocked for this pack.'}</strong>
                  <p>
                    {validation.production_ready
                      ? 'All gated parameters are sourced. Fixture runs are still stamped when selected.'
                      : `${blockingCount} parameter or model gates need attention. Fixture runs remain available.`}
                  </p>
                </div>
                <span className="validation-count">{issueCount} issues</span>
              </div>
            )}

            <div className="tabs">
              <button className={tab === 'parameters' ? 'active' : ''} onClick={() => setTab('parameters')}>Parameters</button>
              <button className={tab === 'scenarios' ? 'active' : ''} onClick={() => setTab('scenarios')}>Scenarios</button>
              <button className={tab === 'provenance' ? 'active' : ''} onClick={() => setTab('provenance')}>Provenance</button>
            </div>

            {tab === 'parameters' && (
              <div className="param-groups">
                {grouped.map((group, index) => (
                  <details key={group.id} className="param-group" open={index < 2}>
                    <summary>
                      <div>
                        <strong>{group.label}</strong>
                        <span>{group.hint}</span>
                      </div>
                      <span className="param-count">{group.parameters.length}</span>
                    </summary>
                    <div className="param-grid">
                      {group.parameters.map((parameter) => (
                        <ParameterField
                          key={parameter.key}
                          parameter={parameter}
                          onChange={(value) => onParameterChange(parameter.key, value)}
                        />
                      ))}
                    </div>
                  </details>
                ))}
              </div>
            )}

            {tab === 'scenarios' && (
              <div className="stack">
                <Card className="scenario-intro">
                  <div>
                    <span className="eyebrow">Intervention design</span>
                    <h3>Scenarios</h3>
                    <p>Coverage is applied by birth cohort. A year-round programme uses months 1–12. Catch-up is optional and is applied to cohorts outside the programme window.</p>
                  </div>
                  <div className="scenario-add">
                    <Button variant="secondary" icon="plus" onClick={() => onAddScenario('maternal')}>Maternal</Button>
                    <Button variant="secondary" icon="plus" onClick={() => onAddScenario('mab')}>mAb</Button>
                  </div>
                </Card>
                <div className="scenario-grid">
                  {draft.scenarios.map((scenario, index) => (
                    <ScenarioCard
                      key={scenario.id}
                      scenario={scenario}
                      onChange={(patch) => onScenarioChange(index, patch)}
                      onRemove={() => onRemoveScenario(index)}
                    />
                  ))}
                </div>
              </div>
            )}

            {tab === 'provenance' && (
              <Card>
                <div className="card-head">
                  <div>
                    <span className="eyebrow">Audit trail</span>
                    <h3>Parameter provenance</h3>
                  </div>
                  <span className="table-sub">Changing a status does not create a citation.</span>
                </div>
                <div className="table-wrap">
                  <table className="provenance-table">
                    <thead>
                      <tr>
                        <th>Parameter</th>
                        <th>Value</th>
                        <th>Status</th>
                        <th>Source</th>
                        <th>Notes</th>
                      </tr>
                    </thead>
                    <tbody>
                      {Object.values(draft.parameters).sort((a, b) => a.key.localeCompare(b.key)).map((parameter) => (
                        <tr key={parameter.key}>
                          <td>
                            <strong>{parameter.label}</strong>
                            <span className="table-sub mono">{parameter.key}</span>
                          </td>
                          <td className="mono">{formatValue(parameter)}</td>
                          <td>
                            <select value={parameter.status} onChange={(event) => onParameterMetaChange(parameter.key, 'status', event.target.value)}>
                              {statuses.map((status) => <option key={status} value={status}>{titleCase(status)}</option>)}
                            </select>
                          </td>
                          <td>
                            <input
                              value={parameter.source ?? ''}
                              placeholder="Citation or source"
                              onChange={(event) => onParameterMetaChange(parameter.key, 'source', event.target.value)}
                            />
                          </td>
                          <td>
                            <input
                              value={parameter.notes ?? ''}
                              placeholder="Reviewer note"
                              onChange={(event) => onParameterMetaChange(parameter.key, 'notes', event.target.value)}
                            />
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </Card>
            )}

            {validation && validation.issues.length > 0 && (
              <Card>
                <div className="card-head">
                  <div>
                    <span className="eyebrow">Validation</span>
                    <h3>Evidence and model checks</h3>
                  </div>
                  <Button variant="ghost" onClick={onValidate}>Re-run checks</Button>
                </div>
                <IssueList issues={validation.issues} />
              </Card>
            )}
          </>
        )}
      </div>

      {newOpen && (
        <Modal
          title="Create country pack"
          onClose={() => setNewOpen(false)}
          footer={
            <>
              <Button variant="ghost" onClick={() => setNewOpen(false)}>Cancel</Button>
              <Button onClick={submitNew} disabled={creating}>{creating ? 'Creating…' : 'Create pack'}</Button>
            </>
          }
        >
          <div className="form-grid">
            <Field label="Country name">
              <input value={newName} onChange={(event) => setNewName(event.target.value)} placeholder="e.g. Ghana" autoFocus />
            </Field>
            <Field label="ISO3 code">
              <input value={newIso3} maxLength={3} onChange={(event) => setNewIso3(event.target.value.toUpperCase())} placeholder="GHA" />
            </Field>
            <Field label="Starting point" hint="The demo is synthetic. The blank template contains no values." className="span-2">
              <select value={newTemplate} onChange={(event) => setNewTemplate(event.target.value as 'demo' | 'blank')}>
                <option value="demo">Synthetic demo pack (runnable fixture)</option>
                <option value="blank">Blank evidence template (production gate closed)</option>
              </select>
            </Field>
          </div>
          {formError && <div className="form-error">{formError}</div>}
        </Modal>
      )}

      {deleteOpen && country && (
        <Modal
          title="Delete country pack"
          onClose={() => setDeleteOpen(false)}
          footer={
            <>
              <Button variant="ghost" onClick={() => setDeleteOpen(false)}>Cancel</Button>
              <Button variant="danger" onClick={confirmDelete} disabled={creating}>{creating ? 'Deleting…' : 'Delete permanently'}</Button>
            </>
          }
        >
          <p>Delete <strong>{country.name}</strong> and all of its runs? This cannot be undone.</p>
        </Modal>
      )}
    </div>
  );
}

function ParameterField({ parameter, onChange }: { parameter: Parameter; onChange: (value: number | string | null) => void }) {
  const isNumber = typeof parameter.value === 'number' || (parameter.value === null && parameter.unit !== '');
  const displayValue = parameter.value === null ? '' : String(parameter.value);
  return (
    <div className="param-field">
      <div className="param-label">
        <span>{parameter.label}</span>
        <StatusPill status={parameter.status} />
      </div>
      <div className="param-input">
        <input
          type={isNumber ? 'number' : 'text'}
          step="any"
          value={displayValue}
          onChange={(event) => {
            if (isNumber) {
              onChange(event.target.value === '' ? null : Number(event.target.value));
            } else {
              onChange(event.target.value);
            }
          }}
        />
        {parameter.unit && <span>{parameter.unit}</span>}
      </div>
      <div className="param-foot">
        <span className="mono">{parameter.key}</span>
        {parameter.source && <span title={parameter.source}>Source: {parameter.source}</span>}
      </div>
    </div>
  );
}

function ScenarioCard({
  scenario,
  onChange,
  onRemove,
}: {
  scenario: Scenario;
  onChange: (patch: Partial<Scenario>) => void;
  onRemove: () => void;
}) {
  return (
    <Card className={`scenario-card ${scenario.is_reference ? 'reference' : ''}`}>
      <div className="card-head">
        <div>
          <span className="eyebrow">{scenario.is_reference ? 'Comparator' : 'Intervention'}</span>
          <h3>{scenario.name}</h3>
        </div>
        <div className="card-head-actions">
          <StatusPill status={scenario.is_reference ? 'reference' : scenario.coverage_maternal > 0 || scenario.coverage_mab > 0 ? 'frontier' : 'missing'} />
          {!scenario.is_reference && (
            <button className="icon-button danger" onClick={onRemove} aria-label={`Remove ${scenario.name}`} title="Remove scenario">
              ×
            </button>
          )}
        </div>
      </div>
      <div className="form-grid two">
        <Field label="Scenario name" className="span-2">
          <input value={scenario.name} disabled={scenario.is_reference} onChange={(event) => onChange({ name: event.target.value })} />
        </Field>
        <Field label="Maternal coverage">
          <PercentInput value={scenario.coverage_maternal} disabled={scenario.is_reference} onChange={(value) => onChange({ coverage_maternal: value })} />
        </Field>
        <Field label="mAb coverage">
          <PercentInput value={scenario.coverage_mab} disabled={scenario.is_reference} onChange={(value) => onChange({ coverage_mab: value })} />
        </Field>
        <Field label="Start month">
          <select value={scenario.intervention_start} disabled={scenario.is_reference} onChange={(event) => onChange({ intervention_start: Number(event.target.value) })}>
            {Array.from({ length: 12 }, (_, i) => <option key={i + 1} value={i + 1}>{i + 1}</option>)}
          </select>
        </Field>
        <Field label="End month">
          <select value={scenario.intervention_end} disabled={scenario.is_reference} onChange={(event) => onChange({ intervention_end: Number(event.target.value) })}>
            {Array.from({ length: 12 }, (_, i) => <option key={i + 1} value={i + 1}>{i + 1}</option>)}
          </select>
        </Field>
        <Field label="Catch-up coverage">
          <PercentInput value={scenario.coverage_catch_up} disabled={scenario.is_reference} onChange={(value) => onChange({ coverage_catch_up: value })} />
        </Field>
        <Field label="Price override, maternal" hint="Optional; blank samples the pack range.">
          <input
            type="number"
            step="any"
            disabled={scenario.is_reference}
            value={scenario.price_maternal ?? ''}
            onChange={(event) => onChange({ price_maternal: event.target.value === '' ? null : Number(event.target.value) })}
          />
        </Field>
        <Field label="Price override, mAb">
          <input
            type="number"
            step="any"
            disabled={scenario.is_reference}
            value={scenario.price_mab ?? ''}
            onChange={(event) => onChange({ price_mab: event.target.value === '' ? null : Number(event.target.value) })}
          />
        </Field>
        <Field label="Notes" className="span-2">
          <input value={scenario.notes} disabled={scenario.is_reference} onChange={(event) => onChange({ notes: event.target.value })} />
        </Field>
      </div>
      <div className="scenario-foot">
        <span>Programme months: <strong>{scenario.intervention_start}–{scenario.intervention_end}</strong></span>
        <span>Target share: <strong>{formatNumber((scenario.coverage_maternal + scenario.coverage_mab) * 100, 1)}%</strong></span>
      </div>
    </Card>
  );
}

function PercentInput({ value, onChange, disabled }: { value: number; onChange: (value: number) => void; disabled?: boolean }) {
  return (
    <div className="percent-input">
      <input
        type="number"
        min={0}
        max={100}
        step="0.1"
        disabled={disabled}
        value={formatNumber(value * 100, 1)}
        onChange={(event) => onChange(Math.max(0, Math.min(100, Number(event.target.value))) / 100)}
      />
      <span>%</span>
    </div>
  );
}

function formatValue(parameter: Parameter): string {
  if (parameter.value === null) return '—';
  if (typeof parameter.value === 'number') return formatNumber(parameter.value, 4);
  return String(parameter.value);
}
