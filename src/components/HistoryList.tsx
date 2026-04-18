import type { HistoryEntry } from '../types/commands';

interface HistoryListProps {
  entries: HistoryEntry[];
  /** Index of the latest entry that can be undone (must have inverse_action). */
  undoableIndex: number | null;
  onUndo: () => void;
}

const OUTCOME_ICON: Record<string, string> = {
  success: '✓',
  recoverable_failure: '✗',
  blocked: '⊘',
  timed_out: '⌛',
  partial_success: '◑',
};

function formatTimestamp(iso: string): string {
  const d = new Date(iso);
  if (isNaN(d.getTime())) return '';
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffMin = Math.floor(diffMs / 60_000);
  if (diffMin < 1) return 'just now';
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffH = Math.floor(diffMin / 60);
  if (diffH < 24) return `${diffH}h ago`;
  return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
}

export function HistoryList({ entries, undoableIndex, onUndo }: HistoryListProps) {
  if (entries.length === 0) {
    return (
      <div className="history-list history-list--empty">
        <span className="history-list__empty-msg">No recent commands</span>
      </div>
    );
  }

  // Show newest first, capped at 5 entries.
  const visible = [...entries].reverse().slice(0, 5);

  return (
    <ol className="history-list" aria-label="Recent commands">
      {visible.map((entry, i) => {
        const icon = OUTCOME_ICON[entry.outcome] ?? '·';
        const isUndoable = undoableIndex !== null && i === 0 && !!entry.inverse_action;

        return (
          <li
            key={`${entry.timestamp}-${i}`}
            className={`history-list__item history-list__item--${entry.outcome}`}
          >
            <span className="history-list__outcome" aria-hidden="true">
              {icon}
            </span>
            <span className="history-list__input" title={entry.command.raw_input}>
              {entry.command.raw_input}
            </span>
            <span className="history-list__time">{formatTimestamp(entry.timestamp)}</span>
            {isUndoable && (
              <button
                className="history-list__undo"
                onClick={onUndo}
                title="Undo this action"
                aria-label={`Undo: ${entry.command.raw_input}`}
              >
                ↩
              </button>
            )}
          </li>
        );
      })}
    </ol>
  );
}
