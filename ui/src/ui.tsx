import type { ReactNode } from 'react';
import { statusLabel, statusTone } from './format';

const paths: Record<string, ReactNode> = {
  dashboard: <><rect x="3" y="3" width="7" height="7" rx="1.5" /><rect x="14" y="3" width="7" height="7" rx="1.5" /><rect x="3" y="14" width="7" height="7" rx="1.5" /><rect x="14" y="14" width="7" height="7" rx="1.5" /></>,
  globe: <><circle cx="12" cy="12" r="9" /><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18" /></>,
  play: <><path d="m8 5 11 7-11 7Z" /></>,
  chart: <><path d="M4 19V5M4 19h16" /><path d="m7 15 3-4 3 2 5-7" /></>,
  book: <><path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H20v18H6.5A2.5 2.5 0 0 1 4 18.5Z" /><path d="M4 18.5A2.5 2.5 0 0 1 6.5 16H20" /></>,
  plus: <><path d="M12 5v14M5 12h14" /></>,
  save: <><path d="M5 3h12l4 4v14H3V3Z" /><path d="M7 3v6h9V3M7 21v-7h10v7" /></>,
  check: <><path d="m4 12 5 5L20 6" /></>,
  warning: <><path d="M12 3 2.5 20h19Z" /><path d="M12 9v5M12 17.5v.1" /></>,
  database: <><ellipse cx="12" cy="5" rx="8" ry="3" /><path d="M4 5v7c0 1.7 3.6 3 8 3s8-1.3 8-3V5M4 12v7c0 1.7 3.6 3 8 3s8-1.3 8-3v-7" /></>,
  shield: <><path d="M12 3 4 6v6c0 5 3.5 7.8 8 9 4.5-1.2 8-4 8-9V6Z" /><path d="m8.5 12 2.5 2.5 4.5-5" /></>,
  trash: <><path d="M4 7h16M9 7V4h6v3M6 7l1 14h10l1-14M10 11v6M14 11v6" /></>,
  refresh: <><path d="M20 7v5h-5M4 17v-5h5" /><path d="M6.1 8A8 8 0 0 1 20 12M4 12a8 8 0 0 1 13.9-4" /></>,
  chevron: <><path d="m9 6 6 6-6 6" /></>,
  copy: <><rect x="8" y="8" width="12" height="12" rx="2" /><path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3" /></>,
  info: <><circle cx="12" cy="12" r="9" /><path d="M12 11v5M12 7.5v.1" /></>,
  arrow: <><path d="M5 12h14M13 6l6 6-6 6" /></>,
};

export function Icon({ name, size = 18 }: { name: keyof typeof paths; size?: number }) {
  return (
    <svg
      className="icon"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {paths[name]}
    </svg>
  );
}

export function StatusPill({ status, label }: { status: string; label?: string }) {
  return <span className={`pill pill-${statusTone(status)}`}>{label ?? statusLabel(status)}</span>;
}

export function Badge({ children, tone = 'neutral' }: { children: ReactNode; tone?: string }) {
  return <span className={`pill pill-${tone}`}>{children}</span>;
}

export function Button({
  children,
  onClick,
  variant = 'primary',
  disabled,
  type = 'button',
  icon,
  className = '',
}: {
  children: ReactNode;
  onClick?: () => void;
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
  disabled?: boolean;
  type?: 'button' | 'submit';
  icon?: keyof typeof paths;
  className?: string;
}) {
  return (
    <button
      type={type}
      className={`button button-${variant} ${className}`}
      onClick={onClick}
      disabled={disabled}
    >
      {icon && <Icon name={icon} size={16} />}
      {children}
    </button>
  );
}

export function Card({ children, className = '' }: { children: ReactNode; className?: string }) {
  return <section className={`card ${className}`}>{children}</section>;
}

export function Metric({
  label,
  value,
  detail,
  tone = '',
}: {
  label: string;
  value: ReactNode;
  detail?: ReactNode;
  tone?: string;
}) {
  return (
    <div className={`metric ${tone}`}>
      <span className="metric-label">{label}</span>
      <strong className="metric-value">{value}</strong>
      {detail && <span className="metric-detail">{detail}</span>}
    </div>
  );
}

export function Spinner({ label = 'Working…' }: { label?: string }) {
  return (
    <span className="spinner-wrap">
      <span className="spinner" />
      {label}
    </span>
  );
}

export function ErrorBanner({ message, onDismiss }: { message: string; onDismiss?: () => void }) {
  return (
    <div className="banner banner-error" role="alert">
      <Icon name="warning" />
      <span>{message}</span>
      {onDismiss && (
        <button className="banner-close" onClick={onDismiss} aria-label="Dismiss">
          ×
        </button>
      )}
    </div>
  );
}

export function Toast({ message }: { message: string }) {
  return (
    <div className="toast" role="status">
      <Icon name="check" size={16} />
      {message}
    </div>
  );
}

export function EmptyState({
  icon = 'chart',
  title,
  children,
  action,
}: {
  icon?: keyof typeof paths;
  title: string;
  children: ReactNode;
  action?: ReactNode;
}) {
  return (
    <div className="empty-state">
      <div className="empty-icon"><Icon name={icon} size={30} /></div>
      <h3>{title}</h3>
      <p>{children}</p>
      {action}
    </div>
  );
}

export function Modal({
  title,
  children,
  onClose,
  footer,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
  footer?: ReactNode;
}) {
  return (
    <div className="modal-backdrop" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>
      <div className="modal" role="dialog" aria-modal="true" aria-label={title}>
        <div className="modal-head">
          <h2>{title}</h2>
          <button className="icon-button" onClick={onClose} aria-label="Close">×</button>
        </div>
        <div className="modal-body">{children}</div>
        {footer && <div className="modal-foot">{footer}</div>}
      </div>
    </div>
  );
}

export function Field({
  label,
  hint,
  children,
  className = '',
}: {
  label: string;
  hint?: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <label className={`field ${className}`}>
      <span className="field-label">{label}</span>
      {children}
      {hint && <span className="field-hint">{hint}</span>}
    </label>
  );
}

export interface IssueLike {
  severity: string;
  code: string;
  message: string;
  parameter?: string | null;
  blocks_production?: boolean;
}

/**
 * Group validation issues by code so that a long list of per-parameter evidence
 * issues stays readable: every distinct issue is visible, with the number of
 * parameters it affects.
 */
export function IssueList({ issues, limit = 12 }: { issues: IssueLike[]; limit?: number }) {
  if (issues.length === 0) {
    return <p className="table-note">No issues were reported.</p>;
  }
  const groups = new Map<string, { issue: IssueLike; count: number; parameters: string[] }>();
  for (const issue of issues) {
    const entry = groups.get(issue.code);
    if (entry) {
      entry.count += 1;
      if (issue.parameter) entry.parameters.push(issue.parameter);
    } else {
      groups.set(issue.code, { issue, count: 1, parameters: issue.parameter ? [issue.parameter] : [] });
    }
  }
  const ordered = [...groups.values()].sort((a, b) => {
    const rank = (severity: string) => (severity === 'error' ? 0 : severity === 'warning' ? 1 : 2);
    return rank(a.issue.severity) - rank(b.issue.severity) || b.count - a.count;
  });
  return (
    <div className="issue-list">
      {ordered.slice(0, limit).map(({ issue, count, parameters }) => (
        <div key={issue.code} className={`issue issue-${issue.severity}`}>
          <StatusPill
            status={issue.severity === 'error' ? 'missing' : issue.severity === 'warning' ? 'convention' : 'ok'}
            label={issue.severity}
          />
          <div>
            <strong>
              {issue.code}
              {count > 1 && <span className="issue-count">× {count}</span>}
            </strong>
            <p>{issue.message}</p>
            {parameters.length > 0 && (
              <p className="issue-parameters mono">{parameters.slice(0, 8).join(', ')}{parameters.length > 8 ? `, +${parameters.length - 8} more` : ''}</p>
            )}
          </div>
        </div>
      ))}
      {ordered.length > limit && <p className="table-note">{ordered.length - limit} further issue groups are not shown.</p>}
    </div>
  );
}
