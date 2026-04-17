import type { ExecutionEvent } from '../types/events';

interface EventTimelineProps {
  events: ExecutionEvent[];
}

const KIND_ICON: Record<string, string> = {
  started: '▶',
  progress: '·',
  completed: '✓',
  failed: '✗',
  cancelled: '○',
};

export function EventTimeline({ events }: EventTimelineProps) {
  if (events.length === 0) return null;

  return (
    <ol className="event-timeline" aria-label="Execution log">
      {events.map((ev) => (
        <li
          key={ev.id}
          className={`event-timeline__item event-timeline__item--${ev.kind}`}
        >
          <span className="event-timeline__icon" aria-hidden="true">
            {KIND_ICON[ev.kind] ?? '·'}
          </span>
          <span className="event-timeline__message">{ev.message}</span>
        </li>
      ))}
    </ol>
  );
}
