export function formatNumber(value: number | null | undefined, digits = 0): string {
  if (value === null || value === undefined || Number.isNaN(value) || !Number.isFinite(value)) return '—';
  return new Intl.NumberFormat('en-US', {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  }).format(value);
}

export function formatCompact(value: number | null | undefined): string {
  if (value === null || value === undefined || Number.isNaN(value) || !Number.isFinite(value)) return '—';
  return new Intl.NumberFormat('en-US', { notation: 'compact', maximumFractionDigits: 1 }).format(value);
}

export function formatMoney(value: number | null | undefined, currency = 'TND', digits = 0): string {
  if (value === null || value === undefined || Number.isNaN(value) || !Number.isFinite(value)) return '—';
  try {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency,
      maximumFractionDigits: digits,
      minimumFractionDigits: digits,
    }).format(value);
  } catch {
    return `${formatNumber(value, digits)} ${currency}`;
  }
}

export function formatIcer(value: number | null | undefined, currency = 'TND'): string {
  if (value === null || value === undefined || !Number.isFinite(value)) return '—';
  return `${formatMoney(value, currency)} / QALY`;
}

export function formatDate(ms: number | null | undefined): string {
  if (!ms) return '—';
  return new Intl.DateTimeFormat('en-GB', {
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(ms));
}

export function titleCase(value: string): string {
  return value.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
}

export function statusLabel(status: string): string {
  switch (status) {
    case 'ok': return 'Sourced';
    case 'derived': return 'Derived';
    case 'transfer': return 'Transferred';
    case 'convention': return 'Convention';
    case 'missing': return 'Missing';
    case 'synthetic': return 'Synthetic';
    case 'reference': return 'Comparator';
    case 'frontier': return 'Frontier';
    case 'dominant': return 'Dominant';
    case 'dominated': return 'Dominated';
    case 'complete': return 'Complete';
    case 'failed': return 'Failed';
    case 'running': return 'Running';
    default: return titleCase(status);
  }
}

export function statusTone(status: string): string {
  if (['ok', 'complete', 'frontier', 'dominant'].includes(status)) return 'good';
  if (['derived', 'transfer', 'convention'].includes(status)) return 'info';
  if (['warning'].includes(status)) return 'warn';
  if (['missing', 'synthetic', 'failed', 'dominated', 'error'].includes(status)) return 'bad';
  return 'neutral';
}

export function slugify(value: string): string {
  return value
    .toLowerCase()
    .normalize('NFKD')
    .replace(/[^\w\s-]/g, '')
    .trim()
    .replace(/[\s_-]+/g, '-')
    .replace(/^-+|-+$/g, '');
}
