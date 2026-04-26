import { useEffect, useRef, useState } from "react";
import type { ChangeEvent, KeyboardEvent } from "react";
import type { CommandSuggestion, ResultFeedback } from "../types/commands";
import "./LoungeStrip.css";

interface LoungeStripProps {
  inputValue: string;
  prediction: string;
  suggestions: CommandSuggestion[];
  clarificationMessage?: string | null;
  clarificationSlots?: string[];
  choices?: string[];
  execState:
    | "idle"
    | "parsing"
    | "awaiting_clarify"
    | "awaiting_choice"
    | "awaiting_route"
    | "awaiting_confirm"
    | "executing"
    | "done"
    | "error";
  alwaysOnTop: boolean;
  pinBusy?: boolean;
  focusTrigger: number;
  resultFeedback?: ResultFeedback | null;
  windowFeedback?: ResultFeedback | null;
  embedded?: boolean;
  onInput: (value: string) => void;
  onSubmit: (value: string) => void;
  onAcceptPrediction: () => void;
  onApplySuggestion: (value: string) => void;
  onSelectChoice: (value: string) => void;
  onEscape: () => void;
  onToggleAlwaysOnTop: () => void;
  onOpenEngineLink?: () => void;
}

export function LoungeStrip({
  inputValue,
  prediction,
  suggestions,
  clarificationMessage,
  clarificationSlots = [],
  choices = [],
  execState,
  alwaysOnTop,
  pinBusy,
  focusTrigger,
  resultFeedback,
  windowFeedback,
  embedded,
  onInput,
  onSubmit,
  onAcceptPrediction,
  onApplySuggestion,
  onSelectChoice,
  onEscape,
  onToggleAlwaysOnTop,
  onOpenEngineLink,
}: LoungeStripProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [selectedSuggestionIndex, setSelectedSuggestionIndex] = useState(0);

  useEffect(() => {
    inputRef.current?.focus();
  }, [focusTrigger]);

  useEffect(() => {
    setSelectedSuggestionIndex(0);
  }, [inputValue, suggestions.length]);

  const isActive = execState !== "idle";
  const isLoading = execState === "parsing" || execState === "executing";
  const isDone = execState === "done";
  const isError = execState === "error";
  const isAwaitingClarify = execState === "awaiting_clarify";
  const isAwaitingChoice = execState === "awaiting_choice";

  const normalizedInputValue = inputValue.toLowerCase();
  const normalizedPrediction = prediction.toLowerCase();
  const hasPredictionPrefix = Boolean(
    prediction &&
    inputValue &&
    normalizedPrediction.startsWith(normalizedInputValue),
  );
  const predictionTail = hasPredictionPrefix
    ? prediction.slice(inputValue.length)
    : "";
  const showPrediction = predictionTail.length > 0 && !resultFeedback;
  const showSuggestions =
    suggestions.length > 0 &&
    !resultFeedback &&
    !isLoading &&
    !isAwaitingClarify &&
    !isAwaitingChoice;
  const showChoices = choices.length > 0 && !resultFeedback && isAwaitingChoice;
  const showClarify = isAwaitingClarify && !resultFeedback;

  function applySelectedSuggestion() {
    const suggestion = suggestions[selectedSuggestionIndex];
    if (suggestion) {
      onApplySuggestion(suggestion.canonical);
    }
  }

  function handleKeyDown(e: KeyboardEvent<HTMLInputElement>) {
    if (showSuggestions && e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedSuggestionIndex(
        (current) => (current + 1) % suggestions.length,
      );
      return;
    }

    if (showSuggestions && e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedSuggestionIndex(
        (current) => (current - 1 + suggestions.length) % suggestions.length,
      );
      return;
    }

    if (
      (e.key === "Tab" && showSuggestions) ||
      (e.key === "Tab" && showPrediction)
    ) {
      e.preventDefault();
      if (showSuggestions) {
        applySelectedSuggestion();
      } else {
        onAcceptPrediction();
      }
      return;
    }

    if (e.key === "Enter" && inputValue.trim()) {
      e.preventDefault();
      const selectedSuggestion = suggestions[selectedSuggestionIndex];
      if (
        showSuggestions &&
        selectedSuggestion &&
        selectedSuggestion.canonical !== inputValue.trim()
      ) {
        onApplySuggestion(selectedSuggestion.canonical);
        return;
      }
      onSubmit(inputValue.trim());
    } else if (e.key === "Escape") {
      e.preventDefault();
      onEscape();
    }
  }

  function handleChange(e: ChangeEvent<HTMLInputElement>) {
    onInput(e.target.value);
  }

  const placeholder =
    execState === "parsing"
      ? "reading intent…"
      : execState === "executing"
        ? "running sequence…"
        : execState === "awaiting_clarify"
          ? "add the missing detail…"
          : execState === "awaiting_choice"
            ? "choose an action or refine…"
            : "tell extendead what to do";

  const stateClass = [
    "lounge-strip",
    isActive ? "lounge-strip--active" : "",
    isLoading ? "lounge-strip--loading" : "",
    !isActive ? "lounge-strip--ready" : "",
    isDone ? "lounge-strip--done" : "",
    isError ? "lounge-strip--error" : "",
    isAwaitingClarify ? "lounge-strip--clarify" : "",
    isAwaitingChoice ? "lounge-strip--choice" : "",
    !alwaysOnTop ? "lounge-strip--floating" : "",
    embedded ? "lounge-strip--embedded" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={stateClass}>
      <div className="lounge-strip__body">
        <span className="lounge-strip__marker" aria-hidden="true" />
        <div className="lounge-strip__input-shell">
          {resultFeedback ? (
            <span
              className={`lounge-strip__feedback lounge-strip__feedback--${resultFeedback.type}`}
            >
              {resultFeedback.message}
            </span>
          ) : (
            <>
              {showPrediction && (
                <div className="lounge-strip__ghost" aria-hidden="true">
                  <span className="lounge-strip__ghost-typed">
                    {inputValue}
                  </span>
                  <span className="lounge-strip__ghost-tail">
                    {predictionTail}
                  </span>
                </div>
              )}

              <input
                ref={inputRef}
                className="lounge-strip__input"
                type="text"
                value={inputValue}
                placeholder={placeholder}
                onChange={handleChange}
                onKeyDown={handleKeyDown}
                disabled={isLoading}
                autoComplete="off"
                spellCheck={false}
              />

              {showSuggestions && (
                <div
                  className="lounge-strip__suggestions"
                  role="listbox"
                  aria-label="Command suggestions"
                >
                  {suggestions.map((suggestion, index) => (
                    <button
                      key={suggestion.id}
                      type="button"
                      className={[
                        "lounge-strip__suggestion",
                        index === selectedSuggestionIndex
                          ? "lounge-strip__suggestion--active"
                          : "",
                      ]
                        .filter(Boolean)
                        .join(" ")}
                      onMouseDown={(e) => {
                        e.preventDefault();
                        onApplySuggestion(suggestion.canonical);
                      }}
                    >
                      <span className="lounge-strip__suggestion-family">
                        {suggestion.family}
                      </span>
                      <span className="lounge-strip__suggestion-command">
                        {suggestion.canonical}
                      </span>
                      <span className="lounge-strip__suggestion-detail">
                        {suggestion.detail}
                      </span>
                    </button>
                  ))}
                </div>
              )}

              {showChoices && (
                <div
                  className="lounge-strip__choices"
                  role="group"
                  aria-label="Choose an action"
                >
                  {choices.map((choice) => (
                    <button
                      key={choice}
                      type="button"
                      className="lounge-strip__choice"
                      onMouseDown={(e) => {
                        e.preventDefault();
                        onSelectChoice(choice);
                      }}
                    >
                      {choice}
                    </button>
                  ))}
                </div>
              )}

              {showClarify && (
                <div
                  className="lounge-strip__clarify"
                  role="status"
                  aria-live="polite"
                >
                  <span className="lounge-strip__clarify-message">
                    {clarificationMessage ||
                      "Need one more detail before I can run this."}
                  </span>
                  {clarificationSlots.length > 0 && (
                    <span className="lounge-strip__clarify-slots">
                      Needed:{" "}
                      {clarificationSlots
                        .map((slot) => slot.replace(/_/g, " "))
                        .join(", ")}
                    </span>
                  )}
                </div>
              )}
            </>
          )}
        </div>

        <div className="lounge-strip__meta">
          <span
            className={`lounge-strip__pin-state ${alwaysOnTop ? "lounge-strip__pin-state--active" : ""}`}
          >
            {alwaysOnTop ? "Pinned" : "Floating"}
          </span>
          <span
            className="lounge-strip__capability-badge"
            title="Offline-capable: local apps and system commands"
          >
            Local
          </span>
          {windowFeedback && (
            <span
              className={`lounge-strip__window-feedback lounge-strip__window-feedback--${windowFeedback.type}`}
            >
              {windowFeedback.message}
            </span>
          )}
          {(showPrediction || showSuggestions) && (
            <span className="lounge-strip__hint">tab</span>
          )}
          {showChoices && <span className="lounge-strip__hint">pick</span>}
          {showClarify && <span className="lounge-strip__hint">clarify</span>}

          {onOpenEngineLink && (
            <button
              className="lounge-strip__action-btn"
              onClick={onOpenEngineLink}
              title="Engine link"
              aria-label="Open engine link panel"
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 16 16"
                fill="none"
                aria-hidden="true"
              >
                <path
                  d="M8 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z"
                  stroke="currentColor"
                  strokeWidth="1.3"
                />
                <path
                  d="M13.5 8a5.5 5.5 0 0 1-.08.87l1.52 1.19a.36.36 0 0 1 .09.46l-1.44 2.49a.36.36 0 0 1-.44.16l-1.79-.72a5.4 5.4 0 0 1-1.51.87l-.27 1.9a.36.36 0 0 1-.36.3H6.38a.36.36 0 0 1-.36-.3l-.27-1.9a5.7 5.7 0 0 1-1.5-.87l-1.8.72a.36.36 0 0 1-.44-.16L.57 10.52a.36.36 0 0 1 .09-.46l1.52-1.19A5.6 5.6 0 0 1 2.1 8c0-.3.03-.59.08-.87L.66 5.94a.36.36 0 0 1-.09-.46l1.44-2.49a.36.36 0 0 1 .44-.16l1.79.72a5.4 5.4 0 0 1 1.51-.87l.27-1.9A.36.36 0 0 1 6.38.48h2.88c.18 0 .33.13.36.3l.27 1.9a5.7 5.7 0 0 1 1.5.87l1.8-.72a.36.36 0 0 1 .44.16l1.44 2.49a.36.36 0 0 1-.09.46l-1.52 1.19c.05.28.08.57.08.87Z"
                  stroke="currentColor"
                  strokeWidth="1.3"
                />
              </svg>
            </button>
          )}

          <button
            className={`lounge-strip__pin ${alwaysOnTop ? "lounge-strip__pin--active" : ""}`}
            onClick={onToggleAlwaysOnTop}
            disabled={pinBusy}
            title={alwaysOnTop ? "Unpin window" : "Pin window on top"}
            aria-label={alwaysOnTop ? "Unpin window" : "Pin window on top"}
            aria-pressed={alwaysOnTop}
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 16 16"
              fill="none"
              aria-hidden="true"
            >
              <path
                d="M5.6 1.6h4.8l-.7 4 2.2 2.2v1H9v4.1L8 14.4l-1-1.5V8.8H4.1v-1l2.2-2.2-.7-4Z"
                fill="currentColor"
              />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}
