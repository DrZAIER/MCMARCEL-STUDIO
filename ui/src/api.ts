import type {
  CountryListItem,
  CountryPack,
  CountryRecord,
  RunListItem,
  RunMode,
  RunRecord,
  ValidationReport,
} from './types';

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const headers = new Headers(options.headers);
  if (options.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }
  const response = await fetch(path, { ...options, headers });
  if (!response.ok) {
    let message = `Request failed (${response.status})`;
    try {
      const body = await response.json();
      if (body?.error) message = body.error;
    } catch {
      // Keep the status message when the body is not JSON.
    }
    throw new Error(message);
  }
  if (response.status === 204) {
    return undefined as T;
  }
  return response.json() as Promise<T>;
}

export const api = {
  health: () => request<{ status: string; engine_version: string; database: string }>('/api/health'),
  meta: () => request<{ name: string; engine_version: string; database: string; models: string[]; license: string }>('/api/meta'),
  listCountries: () => request<CountryListItem[]>('/api/countries'),
  getCountry: (id: string) => request<CountryRecord>(`/api/countries/${encodeURIComponent(id)}`),
  createCountry: (input: { name: string; iso3: string; template?: 'demo' | 'blank'; mode?: RunMode }) =>
    request<CountryRecord>('/api/countries', { method: 'POST', body: JSON.stringify(input) }),
  updateCountry: (pack: CountryPack) =>
    request<CountryRecord>(`/api/countries/${encodeURIComponent(pack.country.id)}`, {
      method: 'PUT',
      body: JSON.stringify(pack),
    }),
  deleteCountry: (id: string) =>
    request<void>(`/api/countries/${encodeURIComponent(id)}`, { method: 'DELETE' }),
  validateCountry: (id: string, mode: RunMode) =>
    request<ValidationReport>(`/api/countries/${encodeURIComponent(id)}/validate`, {
      method: 'POST',
      body: JSON.stringify({ mode }),
    }),
  listRuns: () => request<RunListItem[]>('/api/runs'),
  getRun: (id: string) => request<RunRecord>(`/api/runs/${encodeURIComponent(id)}`),
  createRun: (input: { country_id: string; draws: number; seed: number; mode: RunMode }) =>
    request<RunRecord>('/api/runs', { method: 'POST', body: JSON.stringify(input) }),
};
