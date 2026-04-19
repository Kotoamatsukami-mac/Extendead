import { useEffect, useRef } from 'react';
import type { ChangeEvent, KeyboardEvent } from 'react';
import type { ResultFeedback } from '../types/commands';
import './LoungeStrip.css';

interface LoungeStripProps {
  inputValue: string;
  prediction: string;
  execState: 'idle' | 'parsing' | 'awaiting_route' | 'awaiting_confirm' | 'executing' | 'done' | 'error';
  alwaysOnTop: boolean;
  /** Increment to trigger a re-focus of the input (e.g. after collapse). */
  focusTrigger: number;
  /** Inline result feedback for one-shot commands. */
  resultFeedback?: ResultFeedback | null;
  /** True when the strip is embedded inside the expanded shell panel. */
  embedded?: boolean;
  onInput: (value: string) => void;
  onSubmit: (value: string) => void;
  onAcceptPrediction: () => void;
  onEscape: () => void;
  onToggleAlwaysOnTop: () => void;
  /** Open the engine-link panel from normal UI. */
  onOpenEngineLink?: () => void;
}

export function LoungeStrip({
  inputValue,
  prediction,
  execState,
  alwaysOnTop,
  focusTrigger,
  resultFeedback,
  embedded,
  onInput,
  onSubmit,
  onAcceptPrediction,
  onEscape,
  onToggleAlwaysOnTop,
  onOpenEngineLink,
}: LoungeStripProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus on mount and whenever focusTrigger increments (e.g. after collapse).
  useEffect(() => {
    inputRef.current?.focus();
  }, [focusTrigger]);

  const isActive = execState !== 'idle';
  const isLoading = execState === 'parsing' || execState === 'executing';
  const isDone = execState === 'done';
  const isError = execState === 'error';

  const normalizedInputValue = inputValue.toLowerCase();
  const normalizedPrediction = prediction.toLowerCase();
  const hasPredictionPrefix = Boolean(
    prediction && inputValue && normalizedPrediction.startsWith(normalizedInputValue),
  );
  const predictionTail = hasPredictionPrefix ? prediction.slice(inputValue.length) : '';
  const showPrediction = predictionTail.length > 0 && !resultFeedback;

  function handleKeyDown(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Tab' && showPrediction) {
      e.preventDefault();
      onAcceptPrediction();
      return;
    }

    if (e.key === 'Enter' && inputValue.trim()) {
      e.preventDefault();
      onSubmit(inputValue.trim());
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onEscape();
    }
  }

  function handleChange(e: ChangeEvent<HTMLInputElement>) {
    onInput(e.target.value);
  }

  const placeholder =
    execState === 'parsing'
      ? 'reading intent…'
      : execState === 'executing'
        ? 'running sequence…'
        : 'tell extendead what to do';

  const stateClass = [
    'lounge-strip',
    isActive ? 'lounge-strip--active' : '',
    isLoading ? 'lounge-strip--loading' : '',
    !isActive ? 'lounge-strip--ready' : '',
    isDone ? 'lounge-strip--done' : '',
    isError ? 'lounge-strip--error' : '',
    embedded ? 'lounge-strip--embedded' : '',
  ].filter(Boolean).join(' ');

  return (
    <div className={stateClass}>
      {/* Drag lane — dedicated grab region on the left edge */}
      <div
        className="lounge-strip__drag"
        data-tauri-drag-region
        aria-hidden="true"
      >
        <span className="lounge-strip__marker" data-tauri-drag-region />
      </div>

      <div className="lounge-strip__body">
        <div className="lounge-strip__input-shell">
          {resultFeedback ? (
            <span className={`lounge-strip__feedback lounge-strip__feedback--${resultFeedback.type}`}>
              {resultFeedback.message}
            </span>
          ) : (
            <>
              {showPrediction && (
                <div className="lounge-strip__ghost" aria-hidden="true">
                  <span className="lounge-strip__ghost-typed">{inputValue}</span>
                  <span className="lounge-strip__ghost-tail">{predictionTail}</span>
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
            </>
          )}
        </div>

        <div className="lounge-strip__meta">
          {showPrediction && <span className="lounge-strip__hint">tab</span>}

          {onOpenEngineLink && (
            <button
              className="lounge-strip__action-btn"
              onClick={onOpenEngineLink}
              title="Engine link"
              aria-label="Open engine link panel"
            >
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                <path d="M8 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z" stroke="currentColor" strokeWidth="1.3" />
                <path d="M13.5 8a5.5 5.5 0 0 1-.08.87l1.52 1.19a.36.36 0 0 1 .09.46l-1.44 2.49a.36.36 0 0 1-.44.16l-1.79-.72a5.4 5.4 0 0 1-1.51.87l-.27 1.9a.36.36 0 0 1-.36.3H6.38a.36.36 0 0 1-.36-.3l-.27-1.9a5.7 5.7 0 0 1-1.5-.87l-1.8.72a.36.36 0 0 1-.44-.16L.57 10.52a.36.36 0 0 1 .09-.46l1.52-1.19A5.6 5.6 0 0 1 2.1 8c0-.3.03-.59.08-.87L.66 5.94a.36.36 0 0 1-.09-.46l1.44-2.49a.36.36 0 0 1 .44-.16l1.79.72a5.4 5.4 0 0 1 1.51-.87l.27-1.9A.36.36 0 0 1 6.38.48h2.88c.18 0 .33.13.36.3l.27 1.9a5.7 5.7 0 0 1 1.5.87l1.8-.72a.36.36 0 0 1 .44.16l1.44 2.49a.36.36 0 0 1-.09.46l-1.52 1.19c.05.28.08.57.08.87Z" stroke="currentColor" strokeWidth="1.3" />
              </svg>
            </button>
          )}

          <button
            className={`lounge-strip__pin ${alwaysOnTop ? 'lounge-strip__pin--active' : ''}`}
            onClick={onToggleAlwaysOnTop}
            title={alwaysOnTop ? 'Unpin window' : 'Pin window on top'}
            aria-label={alwaysOnTop ? 'Unpin window' : 'Pin window on top'}
          >
            {alwaysOnTop ? '◉' : '◎'}
          </button>
        </div>
      </div>
    </div>
  );
}
