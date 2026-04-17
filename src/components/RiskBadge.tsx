import type { RiskLevel } from '../types/commands';

interface RiskBadgeProps {
  risk: RiskLevel;
}

const RISK_META: Record<RiskLevel, { label: string; colorVar: string; title: string }> = {
  R0: {
    label: 'R0',
    colorVar: 'var(--accent-r0)',
    title: 'Read-only / no side effects',
  },
  R1: {
    label: 'R1',
    colorVar: 'var(--accent-r1)',
    title: 'Reversible action',
  },
  R2: {
    label: 'R2',
    colorVar: 'var(--accent-r2)',
    title: 'Medium risk — review before confirming',
  },
  R3: {
    label: 'R3',
    colorVar: 'var(--accent-r3)',
    title: 'High risk — destructive, use caution',
  },
};

export function RiskBadge({ risk }: RiskBadgeProps) {
  const meta = RISK_META[risk];
  return (
    <span
      className="risk-badge"
      style={{ '--risk-color': meta.colorVar } as React.CSSProperties}
      title={meta.title}
      aria-label={`Risk level ${meta.label}: ${meta.title}`}
    >
      {meta.label}
    </span>
  );
}
