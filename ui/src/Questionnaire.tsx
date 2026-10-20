import { useCallback, useEffect, useMemo, useState } from 'react';
import { api } from './api';
import type {
  EvidenceStatus,
  QuestionnaireAnswer,
  QuestionnaireCatalogue,
  QuestionnaireIdentity,
  QuestionnaireQuestion,
  QuestionnaireSubmission,
} from './types';
import { Button, Card, Field, Icon, Spinner, StatusPill } from './ui';

const statuses: EvidenceStatus[] = ['ok', 'derived', 'transfer', 'convention', 'missing', 'synthetic'];

const emptyAnswer = (): QuestionnaireAnswer => ({
  value: null,
  status: 'missing',
  source: null,
  notes: null,
});

export function Questionnaire({
  onCreated,
  onCancel,
}: {
  onCreated: (id: string) => void;
  onCancel: () => void;
}) {
  const [catalogue, setCatalogue] = useState<QuestionnaireCatalogue | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [groupIndex, setGroupIndex] = useState(0);
  const [identity, setIdentity] = useState<QuestionnaireIdentity>({
    name: '',
    iso3: '',
    currency: '',
    price_year: new Date().getFullYear(),
    perspective: 'healthcare',
    region: '',
    locale: 'en',
    notes: '',
  });
  const [answers, setAnswers] = useState<Record<string, QuestionnaireAnswer>>({});

  useEffect(() => {
    let alive = true;
    api
      .questionnaire()
      .then((loaded) => {
        if (!alive) return;
        setCatalogue(loaded);
        setAnswers(Object.fromEntries(loaded.questions.map((question) => [question.key, emptyAnswer()])));
      })
      .catch((loadError) => {
        if (alive) setError(loadError instanceof Error ? loadError.message : String(loadError));
      });
    return () => {
      alive = false;
    };
  }, []);

  const groups = useMemo(
    () => (catalogue ? [...catalogue.groups].sort((a, b) => a.order - b.order) : []),
    [catalogue],
  );
  const group = groups[groupIndex];
  const questions = useMemo(
    () => (catalogue && group ? catalogue.questions.filter((question) => question.group === group.id) : []),
    [catalogue, group],
  );

  const progress = useMemo(() => {
    const all = catalogue?.questions ?? [];
    let answered = 0;
    let blocking = 0;
    const perGroup: Record<string, { answered: number; total: number; blocking: number }> = {};
    for (const question of all) {
      const answer = answers[question.key];
      const entry = (perGroup[question.group] ??= { answered: 0, total: 0, blocking: 0 });
      entry.total += 1;
      const hasValue = answer?.value !== null && answer?.value !== undefined && answer?.value !== '';
      if (hasValue) {
        answered += 1;
        entry.answered += 1;
      }
      const statusBlocks = answer?.status === 'missing' || answer?.status === 'synthetic';
      if (statusBlocks || (!hasValue && !question.optional)) {
        blocking += 1;
        entry.blocking += 1;
      }
    }
    return { answered, total: all.length, blocking, perGroup };
  }, [catalogue, answers]);

  const setAnswer = useCallback((key: string, patch: Partial<QuestionnaireAnswer>) => {
    setAnswers((current) => ({ ...current, [key]: { ...(current[key] ?? emptyAnswer()), ...patch } }));
  }, []);

  const identityValid = identity.name.trim().length > 0 && identity.iso3.trim().length === 3;

  const submit = useCallback(async () => {
    if (!identityValid) return;
    setBusy(true);
    setError(null);
    try {
      const payload: QuestionnaireSubmission = {
        country: identity,
        answers: Object.fromEntries(
          Object.entries(answers).map(([key, answer]) => [
            key,
            {
              value: answer.value === '' ? null : answer.value,
              status: answer.status,
              source: answer.source && answer.source.trim() ? answer.source : null,
              notes: answer.notes && answer.notes.trim() ? answer.notes : null,
            },
          ]),
        ),
      };
      const record = await api.createFromQuestionnaire(payload);
      onCreated(record.id);
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : String(submitError));
    } finally {
      setBusy(false);
    }
  }, [answers, identity, identityValid, onCreated]);

  if (error && !catalogue) {
    return (
      <Card>
        <div className="banner banner-error">
          <Icon name="warning" />
          <span>{error}</span>
        </div>
      </Card>
    );
  }

  if (!catalogue || !group) {
    return (
      <Card>
        <Spinner label="Loading the country-pack questionnaire…" />
      </Card>
    );
  }

  const isLast = groupIndex === groups.length - 1;

  return (
    <div className="questionnaire">
      <aside className="questionnaire-rail">
        <div className="questionnaire-rail-head">
          <span className="eyebrow">Guided questionnaire</span>
          <h3>Build a country pack</h3>
          <p>
            Every parameter of a McMarcel Studio country pack, with its definition, its unit and where a
            defensible value comes from.
          </p>
        </div>
        <div className="questionnaire-progress">
          <div className="questionnaire-progress-bar">
            <span
              style={{
                width: `${progress.total ? (progress.answered / progress.total) * 100 : 0}%`,
              }}
            />
          </div>
          <div className="questionnaire-progress-text">
            <span>
              <strong>{progress.answered}</strong> of {progress.total} answered
            </span>
            <span className={progress.blocking ? 'warn' : 'good'}>
              <strong>{progress.blocking}</strong> blocking
            </span>
          </div>
        </div>
        <ol className="questionnaire-steps">
          {groups.map((entry, index) => {
            const entryProgress = progress.perGroup[entry.id] ?? { answered: 0, total: 0, blocking: 0 };
            return (
              <li key={entry.id}>
                <button
                  className={index === groupIndex ? 'active' : ''}
                  onClick={() => setGroupIndex(index)}
                >
                  <span className="step-num">{entry.order}</span>
                  <span className="questionnaire-step-body">
                    <strong>{entry.title}</strong>
                    <span>
                      {entryProgress.answered}/{entryProgress.total} answered
                      {entryProgress.blocking ? ` · ${entryProgress.blocking} blocking` : ''}
                    </span>
                  </span>
                </button>
              </li>
            );
          })}
        </ol>
        <div className="questionnaire-rail-foot">
          <Button variant="ghost" onClick={onCancel}>
            Cancel
          </Button>
        </div>
      </aside>

      <div className="questionnaire-main">
        <Card className="questionnaire-identity">
          <div className="card-head">
            <div>
              <span className="eyebrow">Step 0 · Country identity</span>
              <h3>Which country is this pack for?</h3>
              <p className="prose">
                The identity is stored with every run, so a result can always be traced back to the
                country, the currency and the price year it was produced for.
              </p>
            </div>
          </div>
          <div className="form-grid two">
            <Field label="Country name" hint="For example: Ghana">
              <input
                value={identity.name}
                onChange={(event) => setIdentity({ ...identity, name: event.target.value })}
                placeholder="Ghana"
              />
            </Field>
            <Field label="ISO3 code" hint="Three letters, e.g. GHA">
              <input
                value={identity.iso3}
                maxLength={3}
                onChange={(event) =>
                  setIdentity({ ...identity, iso3: event.target.value.toUpperCase() })
                }
                placeholder="GHA"
              />
            </Field>
            <Field label="Currency" hint="ISO 4217 code; every cost uses it">
              <input
                value={identity.currency}
                onChange={(event) =>
                  setIdentity({ ...identity, currency: event.target.value.toUpperCase() })
                }
                placeholder="GHS"
              />
            </Field>
            <Field label="Price year" hint="The year the costs and prices refer to">
              <input
                type="number"
                value={identity.price_year}
                onChange={(event) =>
                  setIdentity({ ...identity, price_year: Number(event.target.value) })
                }
              />
            </Field>
            <Field label="Perspective">
              <select
                value={identity.perspective}
                onChange={(event) => setIdentity({ ...identity, perspective: event.target.value })}
              >
                <option value="healthcare">Health-care perspective</option>
                <option value="payer">Health-care payer perspective</option>
                <option value="societal">Societal perspective (productivity costs not modelled)</option>
              </select>
            </Field>
            <Field label="Region" hint="Optional">
              <input
                value={identity.region}
                onChange={(event) => setIdentity({ ...identity, region: event.target.value })}
                placeholder="Africa"
              />
            </Field>
          </div>
          {!identityValid && (
            <p className="table-note">
              A country name and a three-letter ISO3 code are needed before the pack can be created.
            </p>
          )}
        </Card>

        <div className="questionnaire-group-head">
          <div>
            <span className="eyebrow">
              Section {group.order} of {groups.length}
            </span>
            <h2>{group.title}</h2>
            <p>{group.description}</p>
          </div>
        </div>

        <div className="questionnaire-questions">
          {questions.map((question) => (
            <QuestionCard
              key={question.key}
              question={question}
              answer={answers[question.key] ?? emptyAnswer()}
              onChange={(patch) => setAnswer(question.key, patch)}
            />
          ))}
        </div>

        {error && (
          <div className="banner banner-error">
            <Icon name="warning" />
            <span>{error}</span>
          </div>
        )}

        <div className="questionnaire-actions">
          <Button
            variant="secondary"
            icon="arrow"
            onClick={() => setGroupIndex((index) => Math.max(0, index - 1))}
            disabled={groupIndex === 0}
          >
            Previous section
          </Button>
          {isLast ? (
            <Button icon="check" onClick={submit} disabled={!identityValid || busy}>
              {busy ? 'Creating the pack…' : 'Create country pack'}
            </Button>
          ) : (
            <Button
              icon="arrow"
              onClick={() => setGroupIndex((index) => Math.min(groups.length - 1, index + 1))}
            >
              Next section
            </Button>
          )}
        </div>
        {isLast && (
          <p className="table-note">
            The pack is created with every unanswered parameter marked <code>MISSING</code>. Production
            mode stays blocked until each of them carries a value, an evidence status and a source.
          </p>
        )}
      </div>
    </div>
  );
}

function QuestionCard({
  question,
  answer,
  onChange,
}: {
  question: QuestionnaireQuestion;
  answer: QuestionnaireAnswer;
  onChange: (patch: Partial<QuestionnaireAnswer>) => void;
}) {
  const [open, setOpen] = useState(false);
  const numeric = question.kind !== 'text';
  const hasValue = answer.value !== null && answer.value !== undefined && answer.value !== '';

  return (
    <Card className={`question-card ${hasValue ? 'answered' : ''}`}>
      <div className="question-card-head">
        <div>
          <h3>{question.label}</h3>
          <div className="question-meta">
            <span className="mono">{question.key}</span>
            {question.unit && <span className="table-sub">{question.unit}</span>}
            {question.optional && <StatusPill status="convention" label="optional" />}
          </div>
        </div>
        <StatusPill status={answer.status} />
      </div>

      <p className="question-definition">{question.definition}</p>

      <div className="question-guidance">
        <div>
          <span className="mini-label">Why it matters</span>
          <p>{question.why_it_matters}</p>
        </div>
        <div>
          <span className="mini-label">How to source it</span>
          <p>{question.how_to_source}</p>
        </div>
      </div>

      {question.reference_sources.length > 0 && (
        <div className="question-sources">
          <button className="question-sources-toggle" onClick={() => setOpen((value) => !value)}>
            <Icon name="book" size={14} />
            {open ? 'Hide' : 'Show'} {question.reference_sources.length} known source
            {question.reference_sources.length === 1 ? '' : 's'}
          </button>
          {open && (
            <ul>
              {question.reference_sources.map((source) => (
                <li key={source.citation}>
                  <strong>{source.citation}</strong>
                  <span>{source.note}</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}

      <div className="question-inputs">
        <Field label="Value" hint={question.range ? `${question.range[0]} to ${question.range[1]}` : undefined}>
          {numeric ? (
            <input
              type="number"
              step="any"
              value={answer.value ?? ''}
              onChange={(event) =>
                onChange({
                  value: event.target.value === '' ? null : Number(event.target.value),
                })
              }
            />
          ) : (
            <input
              type="text"
              value={(answer.value as string) ?? ''}
              onChange={(event) =>
                onChange({ value: event.target.value === '' ? null : event.target.value })
              }
            />
          )}
        </Field>
        <Field label="Evidence status" hint={question.evidence_guidance}>
          <select
            value={answer.status}
            onChange={(event) => onChange({ status: event.target.value as EvidenceStatus })}
          >
            {statuses.map((status) => (
              <option key={status} value={status}>
                {status === 'ok'
                  ? 'OK — sourced country evidence'
                  : status === 'derived'
                    ? 'DERIVED — calculated from country inputs'
                    : status === 'transfer'
                      ? 'TRANSFER — taken from another setting'
                      : status === 'convention'
                        ? 'CONVENTION — modelling choice'
                        : status === 'missing'
                          ? 'MISSING — no defensible value'
                          : 'SYNTHETIC — demo or fixture data'}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Source" hint="Citation, file or database the value comes from">
          <input
            type="text"
            value={answer.source ?? ''}
            placeholder="e.g. National statistics office, 2024"
            onChange={(event) => onChange({ source: event.target.value })}
          />
        </Field>
        <Field label="Reviewer note" hint="Optional">
          <input
            type="text"
            value={answer.notes ?? ''}
            onChange={(event) => onChange({ notes: event.target.value })}
          />
        </Field>
      </div>
    </Card>
  );
}
