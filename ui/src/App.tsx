import { useCallback, useEffect, useMemo, useState } from 'react';
import { api } from './api';
import type {
  CountryListItem,
  CountryPack,
  CountryRecord,
  RunListItem,
  RunMode,
  RunRecord,
  Scenario,
  ValidationReport,
} from './types';
import { Layout, type Page } from './Layout';
import { Dashboard } from './Dashboard';
import { Countries } from './Countries';
import { Questionnaire } from './Questionnaire';
import { RunPanel } from './RunPanel';
import { Results } from './Results';
import { Methods } from './Methods';
import { ErrorBanner, Spinner, Toast } from './ui';

interface Meta {
  name: string;
  engine_version: string;
  database: string;
  models: string[];
  license: string;
}

export default function App() {
  const [page, setPage] = useState<Page>('dashboard');
  const [countries, setCountries] = useState<CountryListItem[]>([]);
  const [runs, setRuns] = useState<RunListItem[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [country, setCountry] = useState<CountryRecord | null>(null);
  const [draft, setDraft] = useState<CountryPack | null>(null);
  const [validation, setValidation] = useState<ValidationReport | null>(null);
  const [activeRun, setActiveRun] = useState<RunRecord | null>(null);
  const [activeCurrency, setActiveCurrency] = useState('TND');
  const [meta, setMeta] = useState<Meta | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  const notify = useCallback((message: string) => {
    setToast(message);
    window.setTimeout(() => setToast(null), 3200);
  }, []);

  const refreshLists = useCallback(async () => {
    const [nextCountries, nextRuns] = await Promise.all([api.listCountries(), api.listRuns()]);
    setCountries(nextCountries);
    setRuns(nextRuns);
    return nextCountries;
  }, []);

  const loadCountry = useCallback(async (id: string) => {
    setBusy('country');
    setError(null);
    try {
      const record = await api.getCountry(id);
      setSelectedId(record.id);
      setCountry(record);
      setDraft(structuredClone(record.pack));
      const report = await api.validateCountry(record.id, record.mode);
      setValidation(report);
    } catch (loadError) {
      setError(loadError instanceof Error ? loadError.message : String(loadError));
    } finally {
      setBusy(null);
    }
  }, []);

  useEffect(() => {
    let alive = true;
    (async () => {
      try {
        const [nextCountries, nextRuns, nextMeta] = await Promise.all([api.listCountries(), api.listRuns(), api.meta()]);
        if (!alive) return;
        setCountries(nextCountries);
        setRuns(nextRuns);
        setMeta(nextMeta);
        if (nextCountries.length > 0) {
          const record = await api.getCountry(nextCountries[0].id);
          if (!alive) return;
          setSelectedId(record.id);
          setCountry(record);
          setDraft(structuredClone(record.pack));
          const report = await api.validateCountry(record.id, record.mode);
          if (alive) setValidation(report);
        }
      } catch (loadError) {
        if (alive) setError(loadError instanceof Error ? loadError.message : String(loadError));
      } finally {
        if (alive) setLoading(false);
      }
    })();
    return () => { alive = false; };
  }, []);

  const dirty = useMemo(() => {
    if (!draft || !country) return false;
    return JSON.stringify(draft) !== JSON.stringify(country.pack);
  }, [draft, country]);

  const selectCountry = useCallback(async (id: string) => {
    if (!id) return;
    if (dirty && !window.confirm('The active pack has unsaved changes. Discard them and switch country?')) {
      return;
    }
    await loadCountry(id);
  }, [dirty, loadCountry]);

  const saveDraft = useCallback(async (silent = false) => {
    if (!draft) return null;
    setBusy('save');
    setError(null);
    try {
      const record = await api.updateCountry(draft);
      setCountry(record);
      setDraft(structuredClone(record.pack));
      await refreshLists();
      if (!silent) notify('Country pack saved');
      return record;
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : String(saveError));
      return null;
    } finally {
      setBusy(null);
    }
  }, [draft, notify, refreshLists]);

  const validateDraft = useCallback(async () => {
    if (!draft) return;
    setBusy('validate');
    setError(null);
    try {
      const record = await api.updateCountry(draft);
      setCountry(record);
      setDraft(structuredClone(record.pack));
      const report = await api.validateCountry(record.id, record.mode);
      setValidation(report);
      await refreshLists();
      notify(report.production_ready ? 'Validation passed — production-ready' : `Validation complete — ${report.issues.filter((issue) => issue.blocks_production).length} blockers`);
    } catch (validateError) {
      setError(validateError instanceof Error ? validateError.message : String(validateError));
    } finally {
      setBusy(null);
    }
  }, [draft, notify, refreshLists]);

  const updateParameter = useCallback((key: string, value: number | string | null) => {
    setDraft((current) => {
      if (!current) return current;
      const parameter = current.parameters[key];
      if (!parameter) return current;
      return {
        ...current,
        parameters: { ...current.parameters, [key]: { ...parameter, value } },
      };
    });
  }, []);

  const updateParameterMeta = useCallback((key: string, field: 'status' | 'source' | 'notes', value: string) => {
    setDraft((current) => {
      if (!current) return current;
      const parameter = current.parameters[key];
      if (!parameter) return current;
      return {
        ...current,
        parameters: {
          ...current.parameters,
          [key]: { ...parameter, [field]: field === 'status' ? (value as CountryPack['parameters'][string]['status']) : value },
        },
      };
    });
  }, []);

  const updateScenario = useCallback((index: number, patch: Partial<Scenario>) => {
    setDraft((current) => {
      if (!current) return current;
      return {
        ...current,
        scenarios: current.scenarios.map((scenario, scenarioIndex) => scenarioIndex === index ? { ...scenario, ...patch } : scenario),
      };
    });
  }, []);

  const addScenario = useCallback((kind: 'maternal' | 'mab') => {    setDraft((current) => {
      if (!current) return current;
      const baseId = kind === 'maternal' ? 'maternal' : 'mab';
      let id = baseId;
      let suffix = 2;
      while (current.scenarios.some((scenario) => scenario.id === id)) {
        id = `${baseId}-${suffix}`;
        suffix += 1;
      }
      const template: Scenario = {
        id,
        name: kind === 'maternal' ? `Maternal vaccine ${suffix > 2 ? suffix - 1 : ''}`.trim() : `Monoclonal antibody ${suffix > 2 ? suffix - 1 : ''}`.trim(),
        is_reference: false,
        coverage_maternal: kind === 'maternal' ? (current.parameters.coverage_maternal?.value as number ?? 0.97) : 0,
        coverage_mab: kind === 'mab' ? (current.parameters.coverage_mab?.value as number ?? 0.99) : 0,
        intervention_start: 1,
        intervention_end: 12,
        coverage_catch_up: 0,
        price_maternal: null,
        price_mab: null,
        notes: 'Added in McMarcel Studio.',
      };
      return { ...current, scenarios: [...current.scenarios, template] };
    });
  }, []);

  const removeScenario = useCallback((index: number) => {
    setDraft((current) => {
      if (!current) return current;
      const scenario = current.scenarios[index];
      if (!scenario || scenario.is_reference) return current;
      return { ...current, scenarios: current.scenarios.filter((_, scenarioIndex) => scenarioIndex !== index) };
    });
  }, []);

  const createCountry = useCallback(async (input: { name: string; iso3: string; template: 'demo' | 'blank' }) => {
    setBusy('create');
    setError(null);
    try {
      const record = await api.createCountry(input);
      await refreshLists();
      await loadCountry(record.id);
      setPage('countries');
      setValidation(await api.validateCountry(record.id, record.mode));
      notify('Country pack created');
    } catch (createError) {
      setError(createError instanceof Error ? createError.message : String(createError));
      throw createError;
    } finally {
      setBusy(null);
    }
  }, [loadCountry, notify, refreshLists]);

  const deleteCountry = useCallback(async (id: string) => {
    setBusy('delete');
    setError(null);
    try {
      await api.deleteCountry(id);
      const remaining = await refreshLists();
      if (remaining.length > 0) {
        await loadCountry(remaining[0].id);
      } else {
        setSelectedId(null);
        setCountry(null);
        setDraft(null);
        setValidation(null);
      }
      notify('Country pack deleted');
    } catch (deleteError) {
      setError(deleteError instanceof Error ? deleteError.message : String(deleteError));
      throw deleteError;
    } finally {
      setBusy(null);
    }
  }, [loadCountry, notify, refreshLists]);

  const openRunPanel = useCallback(async () => {
    if (dirty) {
      const saved = await saveDraft(true);
      if (!saved) return;
      notify('Unsaved changes saved before running');
    }
    setPage('run');
  }, [dirty, notify, saveDraft]);

  const runModel = useCallback(async (input: { draws: number; seed: number; mode: RunMode }) => {
    if (!country) return;
    setBusy('run');
    setError(null);
    try {
      const record = await api.createRun({ country_id: country.id, ...input });
      setActiveRun(record);
      setActiveCurrency(country.pack.country.currency);
      await refreshLists();
      setPage('results');
      notify('Run complete');
    } catch (runError) {
      setError(runError instanceof Error ? runError.message : String(runError));
    } finally {
      setBusy(null);
    }
  }, [country, notify, refreshLists]);

  const openRun = useCallback(async (id: string) => {
    setBusy('open');
    setError(null);
    try {
      const record = await api.getRun(id);
      setActiveRun(record);
      const runCountry = await api.getCountry(record.country_id);
      setActiveCurrency(runCountry.pack.country.currency);
      setPage('results');
    } catch (openError) {
      setError(openError instanceof Error ? openError.message : String(openError));
    } finally {
      setBusy(null);
    }
  }, []);

  const selectedCountry = countries.find((item) => item.id === selectedId) ?? null;

  if (loading) {
    return (
      <div className="boot-screen">
        <div className="boot-mark">M</div>
        <Spinner label="Opening McMarcel Studio…" />
      </div>
    );
  }

  return (
    <Layout
      page={page}
      onNavigate={setPage}
      countries={countries}
      selectedCountryId={selectedId}
      onSelectCountry={selectCountry}
      engineVersion={meta?.engine_version ?? '0.1.0'}
    >
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}
      {page === 'dashboard' && (
        <Dashboard
          countries={countries}
          runs={runs}
          selectedCountry={selectedCountry}
          engineVersion={meta?.engine_version ?? '0.1.0'}
          database={meta?.database ?? 'data/mcmarcel.db'}
          onGoCountries={() => setPage('countries')}
          onRun={() => setPage('run')}
          onOpenRun={openRun}
        />
      )}
      {page === 'countries' && (
        <Countries
          countries={countries}
          country={country}
          draft={draft}
          validation={validation}
          busy={busy}
          dirty={dirty}
          onSelect={selectCountry}
          onParameterChange={updateParameter}
          onParameterMetaChange={updateParameterMeta}
          onScenarioChange={updateScenario}
          onAddScenario={addScenario}
          onRemoveScenario={removeScenario}
          onSave={() => saveDraft()}
          onValidate={validateDraft}
          onCreate={createCountry}
          onDelete={deleteCountry}
          onRun={openRunPanel}
          onStartQuestionnaire={() => setPage('questionnaire')}
        />
      )}
      {page === 'questionnaire' && (
        <Questionnaire
          onCancel={() => setPage('countries')}
          onCreated={async (id) => {
            await refreshLists();
            await loadCountry(id);
            setPage('countries');
            notify('Country pack created from the questionnaire');
          }}
        />
      )}
      {page === 'run' && (
        <RunPanel
          countries={countries}
          country={country}
          validation={validation}
          busy={busy}
          onSelectCountry={selectCountry}
          onRun={runModel}
          onGoCountries={() => setPage('countries')}
        />
      )}
      {page === 'results' && (
        <Results
          run={activeRun}
          currency={activeCurrency}
          onRunAgain={() => setPage('run')}
          onGoCountries={() => setPage('countries')}
        />
      )}
      {page === 'methods' && (
        <Methods
          engineVersion={meta?.engine_version ?? '0.1.0'}
          database={meta?.database ?? 'data/mcmarcel.db'}
        />
      )}
      {busy && busy !== 'run' && busy !== 'save' && busy !== 'validate' && (
        <div className="busy-corner"><Spinner label="Working…" /></div>
      )}
      {toast && <Toast message={toast} />}
    </Layout>
  );
}
