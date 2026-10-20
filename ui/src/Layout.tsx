import type { ReactNode } from 'react';
import type { CountryListItem } from './types';
import { Icon, StatusPill } from './ui';

export type Page = 'dashboard' | 'countries' | 'run' | 'results' | 'methods';

const navItems: Array<{ id: Page; label: string; icon: 'dashboard' | 'globe' | 'play' | 'chart' | 'book' }> = [
  { id: 'dashboard', label: 'Overview', icon: 'dashboard' },
  { id: 'countries', label: 'Countries', icon: 'globe' },
  { id: 'run', label: 'Run model', icon: 'play' },
  { id: 'results', label: 'Results', icon: 'chart' },
  { id: 'methods', label: 'Methods & evidence', icon: 'book' },
];

const titles: Record<Page, { eyebrow: string; title: string }> = {
  dashboard: { eyebrow: 'Decision support', title: 'Overview' },
  countries: { eyebrow: 'Country configuration', title: 'Countries & scenarios' },
  run: { eyebrow: 'Probabilistic engine', title: 'Run model' },
  results: { eyebrow: 'Cost-effectiveness', title: 'Results' },
  methods: { eyebrow: 'Scientific integrity', title: 'Methods & evidence' },
};

export function Layout({
  page,
  onNavigate,
  countries,
  selectedCountryId,
  onSelectCountry,
  engineVersion,
  children,
}: {
  page: Page;
  onNavigate: (page: Page) => void;
  countries: CountryListItem[];
  selectedCountryId: string | null;
  onSelectCountry: (id: string) => void;
  engineVersion: string;
  children: ReactNode;
}) {
  const heading = titles[page];
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark" aria-hidden="true">
            <svg viewBox="0 0 48 48" width="38" height="38">
              <path d="M8 35 18 12l7 15 5-9 10 17" fill="none" stroke="currentColor" strokeWidth="3.4" strokeLinecap="round" strokeLinejoin="round" />
              <circle cx="37" cy="12" r="4" fill="currentColor" />
            </svg>
          </div>
          <div>
            <div className="brand-name">McMarcel</div>
            <div className="brand-sub">Studio</div>
          </div>
        </div>

        <nav className="nav" aria-label="Primary">
          {navItems.map((item) => (
            <button
              key={item.id}
              className={`nav-item ${page === item.id ? 'active' : ''}`}
              onClick={() => onNavigate(item.id)}
              aria-label={item.label}
              aria-current={page === item.id ? 'page' : undefined}
              title={item.label}
            >
              <Icon name={item.icon} />
              <span>{item.label}</span>
            </button>
          ))}
        </nav>

        <div className="sidebar-country">
          <label htmlFor="country-switcher">Active country</label>
          <select
            id="country-switcher"
            value={selectedCountryId ?? ''}
            onChange={(event) => onSelectCountry(event.target.value)}
          >
            {countries.length === 0 && <option value="">No countries yet</option>}
            {countries.map((country) => (
              <option key={country.id} value={country.id}>
                {country.name} · {country.iso3}
              </option>
            ))}
          </select>
        </div>

        <div className="sidebar-foot">
          <div className="local-status">
            <span className="dot dot-live" />
            Local engine
          </div>
          <span>v{engineVersion} · GPL-3.0</span>
        </div>
      </aside>

      <main className="main">
        <header className="topbar">
          <div>
            <span className="eyebrow">{heading.eyebrow}</span>
            <h1>{heading.title}</h1>
          </div>
          <div className="topbar-right">
            <StatusPill status="ok" label="Local-only" />
            <StatusPill status="convention" label="Fixture by default" />
          </div>
        </header>
        <div className="content">{children}</div>
      </main>
    </div>
  );
}
