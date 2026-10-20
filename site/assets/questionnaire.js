/* The interactive questionnaire: it reads the catalogue that the Rust engine
   exports, keeps the answers in the browser and lets the analyst download a
   filled questionnaire (JSON for McMarcel Studio, CSV for a spreadsheet, and a
   printable Markdown document). Nothing is sent anywhere. */
(function () {
  const root = document.getElementById('questionnaire-app');
  if (!root) return;

  const dataNode = document.getElementById('questionnaire-data');
  const catalogue = JSON.parse(dataNode.textContent);
  const groups = [...catalogue.groups].sort((a, b) => a.order - b.order);
  const questions = catalogue.questions;
  const answers = {};
  let activeGroup = groups[0].id;

  const STATUSES = ['ok', 'derived', 'transfer', 'convention', 'missing', 'synthetic'];
  const STATUS_LABEL = {
    ok: 'OK — sourced country evidence',
    derived: 'DERIVED — calculated from country inputs',
    transfer: 'TRANSFER — taken from another setting',
    convention: 'CONVENTION — modelling choice',
    missing: 'MISSING — no defensible value',
    synthetic: 'SYNTHETIC — demo or fixture data',
  };

  const escapeHtml = (value) =>
    String(value === undefined || value === null ? '' : value)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');

  const answerFor = (key) => {
    if (!answers[key]) answers[key] = { value: null, status: 'missing', source: null, notes: null };
    return answers[key];
  };

  const isAnswered = (question) => {
    const answer = answers[question.key];
    if (!answer) return false;
    return answer.value !== null && answer.value !== undefined && answer.value !== '';
  };

  const progress = () => {
    let answered = 0;
    let blocking = 0;
    const perGroup = {};
    questions.forEach((question) => {
      perGroup[question.group] = perGroup[question.group] || { answered: 0, total: 0, blocking: 0 };
      perGroup[question.group].total += 1;
      if (isAnswered(question)) {
        answered += 1;
        perGroup[question.group].answered += 1;
      }
      const status = answers[question.key] ? answers[question.key].status : 'missing';
      if (status === 'missing' || status === 'synthetic' || (!isAnswered(question) && !question.optional)) {
        blocking += 1;
        perGroup[question.group].blocking += 1;
      }
    });
    return { answered, blocking, total: questions.length, perGroup };
  };

  function render() {
    const state = progress();
    const group = groups.find((entry) => entry.id === activeGroup) || groups[0];
    const groupQuestions = questions.filter((question) => question.group === group.id);

    root.innerHTML = `
      <div class="q-layout">
        <aside class="q-rail">
          <div class="q-progress">
            <div class="q-bar"><span style="width:${state.total ? (state.answered / state.total) * 100 : 0}%"></span></div>
            <div class="q-progress-text">
              <span><strong>${state.answered}</strong> of ${state.total} answered</span>
              <span class="${state.blocking ? 'warn' : 'good'}"><strong>${state.blocking}</strong> blocking</span>
            </div>
          </div>
          <ol class="q-steps">
            ${groups
              .map(
                (entry) => `<li><button data-group="${entry.id}" class="${entry.id === activeGroup ? 'active' : ''}">
                  <span class="q-step-num">${entry.order}</span>
                  <span class="q-step-body"><strong>${escapeHtml(entry.title)}</strong>
                  <span>${state.perGroup[entry.id].answered}/${state.perGroup[entry.id].total} answered
                  ${state.perGroup[entry.id].blocking ? ' · ' + state.perGroup[entry.id].blocking + ' blocking' : ''}</span></span>
                </button></li>`,
              )
              .join('')}
          </ol>
          <div class="q-export">
            <h3>Export the questionnaire</h3>
            <p class="muted">Your answers stay in this browser. Download them and import the JSON into
            McMarcel Studio.</p>
            <div class="q-export-buttons">
              <button data-export="json" class="button">JSON</button>
              <button data-export="csv" class="button ghost">CSV</button>
              <button data-export="markdown" class="button ghost">Markdown</button>
              <button data-export="clear" class="button ghost">Clear</button>
            </div>
            <p class="muted" id="q-export-note"></p>
          </div>
        </aside>
        <section class="q-main">
          <div class="q-group-head">
            <p class="eyebrow">Section ${group.order} of ${groups.length}</p>
            <h2>${escapeHtml(group.title)}</h2>
            <p>${escapeHtml(group.description)}</p>
          </div>
          ${groupQuestions.map(questionCard).join('')}
        </section>
      </div>`;

    root.querySelectorAll('[data-group]').forEach((button) => {
      button.addEventListener('click', () => {
        activeGroup = button.getAttribute('data-group');
        render();
      });
    });
    root.querySelectorAll('[data-key]').forEach((input) => {
      const key = input.getAttribute('data-key');
      const field = input.getAttribute('data-field');
      input.addEventListener('input', () => {
        const answer = answerFor(key);
        if (field === 'value') {
          const question = questions.find((entry) => entry.key === key);
          if (question && question.kind !== 'text') {
            answer.value = input.value === '' ? null : Number(input.value);
          } else {
            answer.value = input.value === '' ? null : input.value;
          }
        } else if (field === 'status') {
          answer.status = input.value;
        } else {
          answer[field] = input.value === '' ? null : input.value;
        }
        save();
        updateProgressOnly();
        updateCardState(key);
      });
    });
    root.querySelectorAll('[data-toggle]').forEach((button) => {
      button.addEventListener('click', () => {
        const list = button.nextElementSibling;
        if (list) list.hidden = !list.hidden;
      });
    });
    root.querySelectorAll('[data-export]').forEach((button) => {
      button.addEventListener('click', () => exportQuestionnaire(button.getAttribute('data-export')));
    });
  }

  function updateCardState(key) {
    const answer = answerFor(key);
    root.querySelectorAll('.q-card').forEach((card) => {
      const code = card.querySelector('.q-meta code');
      if (!code || code.textContent !== key) return;
      const pill = card.querySelector('.q-status');
      if (pill) {
        pill.textContent = answer.status.toUpperCase();
        pill.className = `q-status q-status-${answer.status}`;
      }
      card.classList.toggle('answered', isAnswered(questions.find((entry) => entry.key === key)));
    });
  }

  function updateProgressOnly() {
    const state = progress();
    const bar = root.querySelector('.q-bar span');
    if (bar) bar.style.width = `${state.total ? (state.answered / state.total) * 100 : 0}%`;
    const text = root.querySelector('.q-progress-text');
    if (text) {
      text.innerHTML = `<span><strong>${state.answered}</strong> of ${state.total} answered</span>
        <span class="${state.blocking ? 'warn' : 'good'}"><strong>${state.blocking}</strong> blocking</span>`;
    }
    root.querySelectorAll('.q-steps button').forEach((button) => {
      const id = button.getAttribute('data-group');
      const entry = state.perGroup[id];
      const span = button.querySelector('.q-step-body span');
      if (span && entry) {
        span.textContent = `${entry.answered}/${entry.total} answered${entry.blocking ? ' · ' + entry.blocking + ' blocking' : ''}`;
      }
    });
  }

  function questionCard(question) {
    const answer = answerFor(question.key);
    const numeric = question.kind !== 'text';
    const sources = (question.reference_sources || [])
      .map(
        (source) =>
          `<li><strong>${escapeHtml(source.citation)}</strong><span>${escapeHtml(source.note)}</span></li>`,
      )
      .join('');
    return `<article class="q-card${isAnswered(question) ? ' answered' : ''}">
      <header>
        <div>
          <h3>${escapeHtml(question.label)}</h3>
          <div class="q-meta"><code>${escapeHtml(question.key)}</code>${question.unit ? `<span>${escapeHtml(question.unit)}</span>` : ''}${
            question.optional ? '<span class="q-optional">optional</span>' : ''
          }</div>
        </div>
        <span class="q-status q-status-${answer.status}">${escapeHtml(answer.status.toUpperCase())}</span>
      </header>
      <p class="q-definition">${escapeHtml(question.definition)}</p>
      <div class="q-guidance">
        <div><span class="q-label">Why it matters</span><p>${escapeHtml(question.why_it_matters)}</p></div>
        <div><span class="q-label">How to source it</span><p>${escapeHtml(question.how_to_source)}</p></div>
      </div>
      ${
        sources
          ? `<div class="q-sources">
              <button data-toggle="sources" class="q-toggle">${(question.reference_sources || []).length} known source${
                (question.reference_sources || []).length === 1 ? '' : 's'
              }</button>
              <ul hidden>${sources}</ul>
            </div>`
          : ''
      }
      <div class="q-inputs">
        <label><span>Value${question.range ? ` <em>${question.range[0]} to ${question.range[1]}</em>` : ''}</span>
          ${
            numeric
              ? `<input type="number" step="any" data-key="${question.key}" data-field="value" value="${answer.value ?? ''}">`
              : `<input type="text" data-key="${question.key}" data-field="value" value="${escapeHtml(answer.value ?? '')}">`
          }
        </label>
        <label><span>Evidence status</span>
          <select data-key="${question.key}" data-field="status">
            ${STATUSES.map(
              (status) => `<option value="${status}"${status === answer.status ? ' selected' : ''}>${STATUS_LABEL[status]}</option>`,
            ).join('')}
          </select>
          <em>${escapeHtml(question.evidence_guidance)}</em>
        </label>
        <label><span>Source</span>
          <input type="text" data-key="${question.key}" data-field="source" value="${escapeHtml(answer.source ?? '')}" placeholder="Citation, file or database">
        </label>
        <label><span>Reviewer note</span>
          <input type="text" data-key="${question.key}" data-field="notes" value="${escapeHtml(answer.notes ?? '')}">
        </label>
      </div>
    </article>`;
  }

  function save() {
    try {
      localStorage.setItem('mcmarcel-questionnaire', JSON.stringify(answers));
    } catch (error) {
      /* storage may be unavailable; the answers simply stay in memory */
    }
  }

  function restore() {
    try {
      const stored = localStorage.getItem('mcmarcel-questionnaire');
      if (stored) Object.assign(answers, JSON.parse(stored));
    } catch (error) {
      /* ignore unreadable storage */
    }
  }

  function download(name, content, type) {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = name;
    anchor.click();
    URL.revokeObjectURL(url);
    const note = document.getElementById('q-export-note');
    if (note) note.textContent = `downloaded ${name}`;
  }

  function exportQuestionnaire(kind) {
    if (kind === 'clear') {
      Object.keys(answers).forEach((key) => delete answers[key]);
      save();
      render();
      return;
    }
    if (kind === 'json') {
      const payload = {
        country: {
          name: '',
          iso3: '',
          currency: '',
          price_year: new Date().getFullYear(),
          perspective: 'healthcare',
          region: '',
          locale: 'en',
          notes: 'Completed with the McMarcel Studio questionnaire.',
        },
        answers: Object.fromEntries(
          questions.map((question) => {
            const answer = answerFor(question.key);
            return [
              question.key,
              {
                value: answer.value,
                status: answer.status,
                source: answer.source,
                notes: answer.notes,
              },
            ];
          }),
        ),
      };
      download('questionnaire-answers.json', JSON.stringify(payload, null, 2), 'application/json');
      return;
    }
    if (kind === 'csv') {
      const header = [
        'group',
        'key',
        'label',
        'unit',
        'definition',
        'value',
        'evidence_status',
        'source',
        'reviewer_note',
      ];
      const escape = (value) => `"${String(value ?? '').replace(/"/g, '""')}"`;
      const lines = [header.join(',')];
      questions.forEach((question) => {
        const answer = answerFor(question.key);
        lines.push(
          [
            question.group,
            question.key,
            question.label,
            question.unit,
            question.definition,
            answer.value ?? '',
            answer.status,
            answer.source ?? '',
            answer.notes ?? '',
          ]
            .map(escape)
            .join(','),
        );
      });
      download('questionnaire-answers.csv', lines.join('\n'), 'text/csv');
      return;
    }
    const lines = [
      '# McMarcel Studio — completed country-pack questionnaire',
      '',
      '| Parameter | Value | Evidence status | Source | Reviewer note |',
      '|---|---|---|---|---|',
    ];
    questions.forEach((question) => {
      const answer = answerFor(question.key);
      lines.push(
        `| ${question.key} | ${answer.value ?? ''} | ${answer.status} | ${answer.source ?? ''} | ${answer.notes ?? ''} |`,
      );
    });
    download('questionnaire-answers.md', lines.join('\n'), 'text/markdown');
  }

  restore();
  render();
})();
