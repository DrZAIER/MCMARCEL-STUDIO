import type { RunResult } from './types';
import { formatCompact, formatMoney } from './format';

const palette = ['#147d72', '#c46a2b', '#4e67b1', '#8a4f9e', '#b28a1f', '#4d8f3a'];

export function CEPlane({ run, currency }: { run: RunResult; currency: string }) {
  const reference = run.scenarios.find((scenario) => scenario.is_reference);
  const interventions = run.scenarios.filter((scenario) => !scenario.is_reference);
  if (!reference || interventions.length === 0) {
    return <div className="empty-chart">No intervention scenarios to plot.</div>;
  }

  const referenceId = reference.scenario_id;
  const points = run.points.filter((point) => point.scenario_id !== referenceId);
  const allX = [0, ...points.map((point) => point.incremental_effect)];
  const allY = [0, ...points.map((point) => point.incremental_cost)];
  const xDomain = paddedDomain(allX);
  const yDomain = paddedDomain(allY);

  const width = 720;
  const height = 400;
  const margin = { top: 26, right: 28, bottom: 62, left: 88 };
  const plotW = width - margin.left - margin.right;
  const plotH = height - margin.top - margin.bottom;
  const xScale = (value: number) => margin.left + ((value - xDomain[0]) / (xDomain[1] - xDomain[0])) * plotW;
  const yScale = (value: number) => margin.top + plotH - ((value - yDomain[0]) / (yDomain[1] - yDomain[0])) * plotH;
  const xTicks = ticks(xDomain);
  const yTicks = ticks(yDomain);
  const centralWtp = run.analysis.thresholds[1] ?? run.analysis.thresholds[0] ?? 0;

  const frontier = run.analysis.frontier
    .map((id) => run.scenarios.find((scenario) => scenario.scenario_id === id))
    .filter((scenario): scenario is NonNullable<typeof scenario> => Boolean(scenario))
    .map((scenario) => ({
      x: scenario.is_reference ? 0 : scenario.mean_qaly_averted,
      y: scenario.is_reference ? 0 : scenario.mean_incremental_cost,
      id: scenario.scenario_id,
    }));

  const colorFor = (id: string) => {
    const index = interventions.findIndex((scenario) => scenario.scenario_id === id);
    return palette[Math.max(0, index) % palette.length];
  };

  return (
    <div className="ce-plane">
      <div className="ce-plane-head">
        <div>
          <span className="eyebrow">Incremental cost-effectiveness plane</span>
          <h3>Draw-level uncertainty</h3>
        </div>
        <div className="ce-legend">
          {interventions.map((scenario) => (
            <span key={scenario.scenario_id}>
              <i style={{ backgroundColor: colorFor(scenario.scenario_id) }} />
              {scenario.scenario_name}
            </span>
          ))}
        </div>
      </div>
      <div className="chart-scroll">
        <svg viewBox={`0 0 ${width} ${height}`} role="img" aria-label="Cost-effectiveness plane">
          <defs>
            <clipPath id="plot-clip">
              <rect x={margin.left} y={margin.top} width={plotW} height={plotH} />
            </clipPath>
          </defs>
          <rect x={margin.left} y={margin.top} width={plotW} height={plotH} className="plot-bg" />
          {xTicks.map((tick) => (
            <g key={`x-${tick}`}>
              <line x1={xScale(tick)} y1={margin.top} x2={xScale(tick)} y2={margin.top + plotH} className="grid-line" />
              <text x={xScale(tick)} y={margin.top + plotH + 22} textAnchor="middle" className="axis-label">{formatCompact(tick)}</text>
            </g>
          ))}
          {yTicks.map((tick) => (
            <g key={`y-${tick}`}>
              <line x1={margin.left} y1={yScale(tick)} x2={margin.left + plotW} y2={yScale(tick)} className="grid-line" />
              <text x={margin.left - 12} y={yScale(tick) + 4} textAnchor="end" className="axis-label">{formatCompact(tick)}</text>
            </g>
          ))}
          <line x1={margin.left} y1={yScale(0)} x2={margin.left + plotW} y2={yScale(0)} className="axis-line" />
          <line x1={xScale(0)} y1={margin.top} x2={xScale(0)} y2={margin.top + plotH} className="axis-line" />

          {centralWtp > 0 && (
            <line
              x1={xScale(0)}
              y1={yScale(0)}
              x2={xScale(xDomain[1])}
              y2={yScale(centralWtp * xDomain[1])}
              className="wtp-line"
              clipPath="url(#plot-clip)"
            />
          )}

          <g clipPath="url(#plot-clip)">
            {points.map((point) => (
              <circle
                key={`${point.draw}-${point.scenario_id}`}
                cx={xScale(point.incremental_effect)}
                cy={yScale(point.incremental_cost)}
                r={2.1}
                fill={colorFor(point.scenario_id)}
                opacity={0.42}
              >
                <title>{`Draw ${point.draw}: ${formatMoney(point.incremental_cost, currency)} incremental cost, ${formatCompact(point.incremental_effect)} QALYs averted`}</title>
              </circle>
            ))}
            {frontier.length > 1 && (
              <polyline
                points={frontier.map((point) => `${xScale(point.x)},${yScale(point.y)}`).join(' ')}
                className="frontier-line"
              />
            )}
            {frontier.map((point) => (
              <circle key={`frontier-${point.id}`} cx={xScale(point.x)} cy={yScale(point.y)} r={5.2} className="frontier-point">
                <title>{`${point.id}: ${formatMoney(point.y, currency)} incremental cost, ${formatCompact(point.x)} QALYs averted`}</title>
              </circle>
            ))}
          </g>

          <text x={margin.left + plotW / 2} y={height - 10} textAnchor="middle" className="axis-title">Incremental QALYs averted</text>
          <text x={18} y={margin.top + plotH / 2} textAnchor="middle" transform={`rotate(-90 18 ${margin.top + plotH / 2})`} className="axis-title">Incremental cost ({currency})</text>
        </svg>
      </div>
      <div className="chart-foot">
        <span><i className="line-key wtp" /> Central WTP: {formatMoney(centralWtp, currency)} / QALY</span>
        <span><i className="line-key frontier" /> Frontier</span>
        <span className="muted">Each dot is one paired draw. Points below the WTP line are cost-effective at that threshold.</span>
      </div>
    </div>
  );
}

function paddedDomain(values: number[]): [number, number] {
  const finite = values.filter((value) => Number.isFinite(value));
  let min = Math.min(...finite);
  let max = Math.max(...finite);
  if (min === max) {
    min -= 1;
    max += 1;
  }
  const pad = (max - min) * 0.08;
  return [min - pad, max + pad];
}

function ticks(domain: [number, number]): number[] {
  const [min, max] = domain;
  const span = max - min;
  const rough = span / 5;
  const magnitude = 10 ** Math.floor(Math.log10(Math.max(rough, 1e-9)));
  const normalized = rough / magnitude;
  const step = (normalized >= 5 ? 5 : normalized >= 2 ? 2 : 1) * magnitude;
  const start = Math.ceil(min / step) * step;
  const values: number[] = [];
  for (let value = start; value <= max + step * 0.01; value += step) {
    values.push(Number(value.toFixed(10)));
  }
  return values;
}
