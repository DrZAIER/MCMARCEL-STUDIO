import { Card, Icon, StatusPill } from './ui';

export function Methods({
  engineVersion,
  database,
}: {
  engineVersion: string;
  database: string;
}) {
  return (
    <div className="stack">
      <Card className="methods-hero">
        <div>
          <span className="eyebrow">Scientific contract</span>
          <h2>Speed does not excuse silent assumptions.</h2>
          <p>
            McMarcel Studio separates the model engine, country evidence, and decision analysis. A run is only as
            production-ready as the weakest gated parameter in its pack.
          </p>
        </div>
        <div className="methods-badges">
          <StatusPill status="ok" label="GPL-3.0-or-later" />
          <StatusPill status="transfer" label="R oracle preserved" />
          <StatusPill status="convention" label="No invented values" />
        </div>
      </Card>

      <div className="two-column">
        <Card>
          <div className="card-head">
            <div><span className="eyebrow">Evidence states</span><h3>Provenance is first-class</h3></div>
          </div>
          <div className="evidence-status-list">
            <div><StatusPill status="ok" /><p><strong>OK</strong> — direct, reviewed country evidence with a citation.</p></div>
            <div><StatusPill status="derived" /><p><strong>DERIVED</strong> — calculated from documented country inputs; the derivation is recorded.</p></div>
            <div><StatusPill status="transfer" /><p><strong>TRANSFER</strong> — borrowed from another setting and explicitly flagged.</p></div>
            <div><StatusPill status="convention" /><p><strong>CONVENTION</strong> — a modelling choice, not an empirical estimate (e.g. discount rate).</p></div>
            <div><StatusPill status="missing" /><p><strong>MISSING</strong> — no defensible value. Production is blocked.</p></div>
            <div><StatusPill status="synthetic" /><p><strong>SYNTHETIC</strong> — demo or fixture only. Production is blocked and the run is stamped.</p></div>
          </div>
        </Card>

        <Card>
          <div className="card-head">
            <div><span className="eyebrow">Engine</span><h3>Implementation contract</h3></div>
          </div>
          <div className="spec-list">
            <div><span>Engine</span><strong>mcmarcel-core {engineVersion}</strong></div>
            <div><span>Time step</span><strong>Monthly, 60 age months × 12 calendar months</strong></div>
            <div><span>RNG</span><strong>ChaCha8, seeded per draw, paired across scenarios</strong></div>
            <div><span>Storage</span><strong>SQLite STRICT + WAL, local-only</strong></div>
            <div><span>Database</span><strong className="mono">{database}</strong></div>
            <div><span>Licence</span><strong>GPL-3.0-or-later</strong></div>
          </div>
        </Card>
      </div>

      <Card>
        <div className="card-head">
          <div><span className="eyebrow">Architecture</span><h3>Why the Rust rewrite is fast and auditable</h3></div>
        </div>
        <div className="architecture">
          <div className="arch-node"><Icon name="globe" /><strong>Country pack</strong><span>JSON evidence + matrices</span></div>
          <div className="arch-arrow">→</div>
          <div className="arch-node"><Icon name="shield" /><strong>Validation gates</strong><span>Shape, ranges, provenance</span></div>
          <div className="arch-arrow">→</div>
          <div className="arch-node accent"><Icon name="play" /><strong>Rust engine</strong><span>Rayon paired draws</span></div>
          <div className="arch-arrow">→</div>
          <div className="arch-node"><Icon name="database" /><strong>SQLite store</strong><span>Immutable run artefacts</span></div>
          <div className="arch-arrow">→</div>
          <div className="arch-node"><Icon name="chart" /><strong>CEA layer</strong><span>ICER, INMB, P*</span></div>
        </div>
      </Card>

      <div className="two-column">
        <Card>
          <div className="card-head">
            <div><span className="eyebrow">Scope</span><h3>v1 model boundary</h3></div>
          </div>
          <ul className="check-list">
            <li><Icon name="check" size={16} /> Static monthly cohort RSV burden and prevention.</li>
            <li><Icon name="check" size={16} /> Maternal vaccine and monoclonal antibody scenarios.</li>
            <li><Icon name="check" size={16} /> Probabilistic CEA with paired draws and CE plane.</li>
            <li><Icon name="check" size={16} /> Multi-country pack comparison and local audit trail.</li>
            <li className="muted"><span className="cross">×</span> Dynamic transmission, EVPI/EVPPI, budget impact.</li>
            <li className="muted"><span className="cross">×</span> Recurrent wheezing and asthma, caregiver QALY losses, productivity costs.</li>
            <li className="muted"><span className="cross">×</span> Multi-user server deployment (reserved for a later module).</li>
          </ul>
        </Card>

        <Card>
          <div className="card-head">
            <div><span className="eyebrow">Validation</span><h3>R oracle, not runtime dependency</h3></div>
          </div>
          <p className="prose">
            The existing R implementation remains the reference oracle. Rust parity is checked against fixed R fixtures,
            including the documented C1–C16 defect fixes. The desktop runtime does not call R.
          </p>
          <p className="prose">
            The parity fixture calls the R implementation's own functions — cohort conversion, protection by cohort, the
            shared unit-price formula, the gamma parameterisation, the discounting vector and the net monetary benefit —
            and the Rust test suite recomputes the same values. The deliberate deviations of the rewrite are listed in{' '}
            <code>docs/model.md</code>.
          </p>
          <p className="prose">
            The current demo pack is deliberately synthetic. Replace every value, matrix and source before using production
            mode. The production gate is designed to fail closed.
          </p>
          <div className="callout">
            <Icon name="info" size={18} />
            <span>Need a new country? Start from the blank template, import documented inputs, then let validation drive the gap list.</span>
          </div>
        </Card>
      </div>
    </div>
  );
}
