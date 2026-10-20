export type EvidenceStatus = 'ok' | 'derived' | 'transfer' | 'convention' | 'missing' | 'synthetic';
export type RunMode = 'fixture' | 'production';

export interface Parameter {
  key: string;
  label: string;
  unit: string;
  value: number | string | boolean | null;
  low?: number | null;
  high?: number | null;
  status: EvidenceStatus;
  source?: string | null;
  source_ref?: string | null;
  url?: string | null;
  notes?: string | null;
  blocks_outputs?: string[];
}

export interface Matrix {
  rows: number;
  cols: number;
  data: number[];
}

export interface Assets {
  hospital_age_calendar?: Matrix | null;
  icu_age_calendar?: Matrix | null;
  outpatient_rate_age_sample?: Matrix | null;
  hcfr_age?: Matrix | null;
  non_ma_sample_age?: Matrix | null;
  life_table?: Matrix | null;
}

export interface Scenario {
  id: string;
  name: string;
  is_reference: boolean;
  coverage_maternal: number;
  coverage_mab: number;
  intervention_start: number;
  intervention_end: number;
  coverage_catch_up: number;
  price_maternal?: number | null;
  price_mab?: number | null;
  notes: string;
}

export interface CountryMeta {
  id: string;
  name: string;
  iso3: string;
  currency: string;
  price_year: number;
  locale: string;
  perspective: string;
  region: string;
  notes: string;
}

export interface CountryPack {
  schema_version: number;
  country: CountryMeta;
  parameters: Record<string, Parameter>;
  assets: Assets;
  scenarios: Scenario[];
  notes: string;
}

export interface CountryRecord {
  id: string;
  name: string;
  iso3: string;
  mode: RunMode;
  pack: CountryPack;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CountryListItem {
  id: string;
  name: string;
  iso3: string;
  mode: RunMode;
  scenario_count: number;
  parameter_count: number;
  updated_at_ms: number;
  production_ready: boolean;
}

export interface ValidationIssue {
  severity: 'info' | 'warning' | 'error';
  code: string;
  message: string;
  parameter?: string | null;
  blocks_production: boolean;
}

export interface ValidationReport {
  production_ready: boolean;
  issues: ValidationIssue[];
  fixture_stamp?: string | null;
  pack_hash: string;
}

export interface ScenarioSummary {
  scenario_id: string;
  scenario_name: string;
  is_reference: boolean;
  coverage_maternal: number;
  coverage_mab: number;
  mean_reference_cases: number;
  mean_program_cases: number;
  mean_averted_cases: number;
  mean_averted_hospital: number;
  mean_averted_deaths: number;
  mean_reference_cost: number;
  mean_program_cost: number;
  mean_medical_cost_averted: number;
  mean_intervention_cost: number;
  mean_incremental_cost: number;
  mean_reference_qaly: number;
  mean_program_qaly: number;
  mean_qaly_averted: number;
  mean_fvp: number;
  mean_doses: number;
  mean_price: number;
  mean_wastage: number;
  intervention_months: number;
}

export interface DrawPoint {
  draw: number;
  scenario_id: string;
  reference_cost: number;
  reference_effect: number;
  program_cost: number;
  program_effect: number;
  incremental_cost: number;
  incremental_effect: number;
}

export interface IcerRow {
  strategy_id: string;
  strategy: string;
  cost: number;
  effect: number;
  incremental_cost?: number | null;
  incremental_effect?: number | null;
  icer?: number | null;
  status: string;
  note: string;
}

export interface InmbRow {
  threshold: number;
  strategy_id: string;
  strategy: string;
  inmb: number;
}

export interface PstarRow {
  strategy_id: string;
  strategy: string;
  wtp: number;
  pstar?: number | null;
  current_price: number;
  price_floor: number;
  slope: number;
  doses: number;
  fvp: number;
  note: string;
}

export interface AnalysisResult {
  thresholds: number[];
  icer_table: IcerRow[];
  inmb_table: InmbRow[];
  pstar_table: PstarRow[];
  frontier: string[];
}

export interface RunResult {
  pack_id: string;
  pack_name: string;
  pack_hash: string;
  mode: RunMode;
  draws: number;
  seed: number;
  created_at_ms: number;
  fixture_stamp?: string | null;
  scenarios: ScenarioSummary[];
  points: DrawPoint[];
  analysis: AnalysisResult;
  validation: ValidationReport;
  warnings: string[];
}

export interface RunRecord {
  id: string;
  country_id: string;
  country_name: string;
  status: 'running' | 'complete' | 'failed';
  draws: number;
  seed: number;
  mode: RunMode;
  created_at_ms: number;
  completed_at_ms?: number | null;
  error?: string | null;
  result?: RunResult | null;
}

export interface RunListItem {
  id: string;
  country_id: string;
  country_name: string;
  status: string;
  draws: number;
  seed: number;
  mode: RunMode;
  created_at_ms: number;
  completed_at_ms?: number | null;
  error?: string | null;
  pack_name?: string | null;
  fixture_stamp?: string | null;
}

export type QuestionKind = 'number' | 'quantity' | 'text' | 'proportion';

export interface ReferenceSource {
  citation: string;
  note: string;
  reference_model: boolean;
}

export interface QuestionnaireGroup {
  id: string;
  title: string;
  description: string;
  order: number;
  required_for_run: boolean;
}

export interface QuestionnaireQuestion {
  key: string;
  label: string;
  group: string;
  unit: string;
  kind: QuestionKind;
  definition: string;
  why_it_matters: string;
  how_to_source: string;
  reference_sources: ReferenceSource[];
  range: [number, number] | null;
  evidence_guidance: string;
  blocks: string[];
  optional: boolean;
}

export interface QuestionnaireCatalogue {
  groups: QuestionnaireGroup[];
  questions: QuestionnaireQuestion[];
}

export interface QuestionnaireAnswer {
  value: number | string | null;
  status: EvidenceStatus;
  source: string | null;
  notes: string | null;
}

export interface QuestionnaireIdentity {
  name: string;
  iso3: string;
  currency: string;
  price_year: number;
  perspective: string;
  region: string;
  locale: string;
  notes: string;
}

export interface QuestionnaireSubmission {
  country: QuestionnaireIdentity;
  answers: Record<string, QuestionnaireAnswer>;
}
