import { useState } from "react";
import type {
  ExecutionResult,
  HistoryEntry,
  ParsedCommand,
  PermissionStatus,
  ResolvedRoute,
} from "../types/commands";
import type { ExecutionEvent } from "../types/events";

import { ConfirmationRail } from "./ConfirmationRail";
import { EventTimeline } from "./EventTimeline";
import { HistoryList } from "./HistoryList";
import { PermissionBanner } from "./PermissionBanner";
import { RiskBadge } from "./RiskBadge";
import { RouteSelector } from "./RouteSelector";
import "./ExpandedConsole.css";

type ExecState =
  | "idle"
  | "parsing"
  | "awaiting_clarify"
  | "awaiting_choice"
  | "awaiting_route"
  | "awaiting_confirm"
  | "executing"
  | "done"
  | "error";

interface ExpandedConsoleProps {
  parsedCommand: ParsedCommand | null;
  selectedRouteIndex: number | null;
  execState: ExecState;
  events: ExecutionEvent[];
  result: ExecutionResult | null;
  permissionStatus: PermissionStatus | null;
  history: HistoryEntry[];
  alwaysOnTop: boolean;
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
  permissionStatus,
  history,
  alwaysOnTop,
  onSelectRoute,
  onConfirm,
  onCancel,
  onUndo,
  onCollapse,
}: ExpandedConsoleProps) {
  const [showHistory, setShowHistory] = useState(false);

  const selectedRoute: ResolvedRoute | null =
    parsedCommand && selectedRouteIndex !== null
      ? (parsedCommand.routes[selectedRouteIndex] ?? null)
      : null;

  // The undo button in the result card is shown only when the current result has an inverse.
  // The history drawer shows a per-entry undo only for the most recent reversible entry.
  const latestReversibleIndex =
    history.length > 0 &&
    history[history.length - 1].inverse_action !== undefined
      ? 0
      : null;

  return (
    <div className="expanded-console">
      {/* Header */}
      <div className="expanded-console__header">
        <div className="expanded-console__intent">
          {parsedCommand && (
            <>
              <span className="expanded-console__kind">
                {parsedCommand.kind.replace(/_/g, " ")}
              </span>
              <span className="expanded-console__input">
                {parsedCommand.raw_input}
              </span>
            </>
          )}
          {execState === "parsing" && (
            <span className="expanded-console__spinner">Parsing…</span>
          )}
        </div>

        <div className="expanded-console__meta">
          {parsedCommand && <RiskBadge risk={parsedCommand.risk} />}
          <button
            className={`expanded-console__history-btn ${showHistory ? "expanded-console__history-btn--active" : ""}`}
            onClick={() => setShowHistory((v) => !v)}
            title={showHistory ? "Hide history" : "Show history"}
            aria-label={
              showHistory ? "Hide command history" : "Show command history"
            }
            aria-pressed={showHistory}
          >
            🕒
          </button>
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

      {/* History drawer */}
      {showHistory && (
        <div className="expanded-console__history-drawer">
          <HistoryList
            entries={history}
            undoableIndex={latestReversibleIndex}
            onUndo={onUndo}
          />
        </div>
      )}

      {/* Route selector — shown when multiple routes available */}
      {!showHistory && parsedCommand && parsedCommand.routes.length > 1 && (
        <RouteSelector
          routes={parsedCommand.routes}
          selectedIndex={selectedRouteIndex}
          onSelect={onSelectRoute}
        />
      )}

      {/* Plan preview — routes are alternatives; steps live inside one selected route. */}
      {!showHistory && selectedRoute?.action.type === "run_plan" && (
        <div className="plan-preview">
          <div className="plan-preview__header">
            <span className="plan-preview__eyebrow">Plan preview</span>
            <span className="plan-preview__title">
              {selectedRoute.action.mode_name} mode
            </span>
          </div>
          <ol className="plan-preview__steps">
            {selectedRoute.action.steps.map((step, index) => (
              <li
                key={`${step.execution_group}-${index}`}
                className="plan-preview__step"
              >
                <span className="plan-preview__group">
                  {step.execution_group.startsWith("parallel")
                    ? "Concurrent"
                    : "Sequential"}
                </span>
                <span className="plan-preview__label">{step.label}</span>
                <span className="plan-preview__risk">{step.risk}</span>
              </li>
            ))}
          </ol>
        </div>
      )}

      {/* Confirmation rail */}
      {!showHistory &&
        execState === "awaiting_confirm" &&
        selectedRoute &&
        parsedCommand && (
          <ConfirmationRail
            label={selectedRoute.label}
            description={selectedRoute.description}
            onConfirm={onConfirm}
            onCancel={onCancel}
          />
        )}

      {/* Executing state */}
      {!showHistory && execState === "executing" && (
        <div className="expanded-console__executing">
          <span className="expanded-console__spinner">Executing…</span>
        </div>
      )}

      {/* Event timeline */}
      {!showHistory && <EventTimeline events={events} />}

      {/* Result card */}
      {!showHistory && result && execState === "done" && (
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
      {!showHistory && execState === "error" && result && (
        <div className="expanded-console__result expanded-console__result--recoverable_failure">
          <span className="expanded-console__result-msg">
            {result.human_message}
          </span>
        </div>
      )}

      {/* Permission status — shown when any permission is not granted */}
      {permissionStatus && <PermissionBanner status={permissionStatus} />}
    </div>
  );
}
