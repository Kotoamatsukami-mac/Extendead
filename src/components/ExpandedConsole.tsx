import type { ExecutionResult, ParsedCommand, ResolvedRoute } from '../types/commands';
import type { ExecutionEvent } from '../types/events';

import { ConfirmationRail } from './ConfirmationRail';
import { EventTimeline } from './EventTimeline';
import { RiskBadge } from './RiskBadge';
import { RouteSelector } from './RouteSelector';
import './ExpandedConsole.css';

type ExecState =
  | 'idle'
  | 'parsing'
  | 'awaiting_route'
  | 'awaiting_confirm'
  | 'executing'
  | 'done'
  | 'error';

interface ExpandedConsoleProps {
  parsedCommand: ParsedCommand | null;
  selectedRouteIndex: number | null;
  execState: ExecState;
  events: ExecutionEvent[];
  result: ExecutionResult | null;
  onSelectRoute: (index: number) => void;
  onConfirm: () => void;
  onCancel: () => void;
  onUndo: () => void;
  onCollapse: () => void;
}

export function ExpandedConsole({
  parsedCommand,
  selectedRouteIndex,
  execState,
  events,
  result,
  onSelectRoute,
  onConfirm,
  onCancel,
  onUndo,
  onCollapse,
}: ExpandedConsoleProps) {
  const selectedRoute: ResolvedRoute | null =
    parsedCommand && selectedRouteIndex !== null
      ? (parsedCommand.routes[selectedRouteIndex] ?? null)
      : null;

  return (
    <div className="expanded-console">
      {/* Header */}
      <div className="expanded-console__header" data-tauri-drag-region="true">
        <div className="expanded-console__intent">
          {parsedCommand && (
            <>
              <span className="expanded-console__kind">
                {parsedCommand.kind.replace(/_/g, ' ')}
              </span>
              <span className="expanded-console__input">
                {parsedCommand.raw_input}
              </span>
            </>
          )}
          {execState === 'parsing' && (
            <span className="expanded-console__spinner">Parsing…</span>
          )}
        </div>

        <div className="expanded-console__meta">
          {parsedCommand && <RiskBadge risk={parsedCommand.risk} />}
          <button
            className="expanded-console__close"
            onClick={onCollapse}
            title="Collapse (Esc)"
            aria-label="Collapse console"
          >
            ✕
          </button>
        </div>
      </div>

      {/* Route selector — shown when multiple routes available */}
      {parsedCommand && parsedCommand.routes.length > 1 && (
        <RouteSelector
          routes={parsedCommand.routes}
          selectedIndex={selectedRouteIndex}
          onSelect={onSelectRoute}
        />
      )}

      {/* Confirmation rail */}
      {execState === 'awaiting_confirm' && selectedRoute && parsedCommand && (
        <ConfirmationRail
          label={selectedRoute.label}
          description={selectedRoute.description}
          onConfirm={onConfirm}
          onCancel={onCancel}
        />
      )}

      {/* Executing state */}
      {execState === 'executing' && (
        <div className="expanded-console__executing">
          <span className="expanded-console__spinner">Executing…</span>
        </div>
      )}

      {/* Event timeline */}
      <EventTimeline events={events} />

      {/* Result card */}
      {result && execState === 'done' && (
        <div
          className={`expanded-console__result expanded-console__result--${result.outcome}`}
        >
          <span className="expanded-console__result-msg">
            {result.human_message}
          </span>
          {result.duration_ms > 0 && (
            <span className="expanded-console__result-dur">
              {result.duration_ms}ms
            </span>
          )}
          {result.inverse_action && (
            <button
              className="expanded-console__undo"
              onClick={onUndo}
              title="Undo this action"
            >
              ↩ Undo
            </button>
          )}
        </div>
      )}

      {/* Error state */}
      {execState === 'error' && result && (
        <div className="expanded-console__result expanded-console__result--recoverable_failure">
          <span className="expanded-console__result-msg">
            {result.human_message}
          </span>
        </div>
      )}
    </div>
  );
}
