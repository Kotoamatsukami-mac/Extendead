import { useEffect, useRef } from 'react';
import type { ChangeEvent, KeyboardEvent } from 'react';
import './LoungeStrip.css';

interface LoungeStripProps {
  inputValue: string;
  prediction: string;
  execState: 'idle' | 'parsing' | 'awaiting_route' | 'awaiting_confirm' | 'executing' | 'done' | 'error';
  alwaysOnTop: boolean;
  /** Increment to trigger a re-focus of the input (e.g. after collapse). */
  focusTrigger: number;
  onInput: (value: string) => void;
  onSubmit: (value: string) => void;
  onAcceptPrediction: () => void;
  onEscape: () => void;
  onToggleAlwaysOnTop: () => void;
}

export function LoungeStrip({
  inputValue,
  prediction,
  execState,
  alwaysOnTop,
  focusTrigger,
  onInput,
  onSubmit,
  onAcceptPrediction,
  onEscape,
  onToggleAlwaysOnTop,
}: LoungeStripProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus on mount and whenever focusTrigger increments (e.g. after collapse).
  useEffect(() => {
    inputRef.current?.focus();
  }, [focusTrigger]);

  const isActive = execState !== 'idle';
  const isLoading = execState === 'parsing' || execState === 'executing';

  const normalizedInputValue = inputValue.toLowerCase();
  const normalizedPrediction = prediction.toLowerCase();
  const hasPredictionPrefix = Boolean(
    prediction && inputValue && normalizedPrediction.startsWith(normalizedInputValue),
  );
  const predictionTail = hasPredictionPrefix ? prediction.slice(inputValue.length) : '';
  const showPrediction = predictionTail.length > 0;

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

  return (
    <div
      className={`lounge-strip ${isActive ? 'lounge-strip--active' : ''} ${isLoading ? 'lounge-strip--loading' : ''}`}
      data-tauri-drag-region
    >
      <div className="lounge-strip__inner">
        <span className="lounge-strip__marker" aria-hidden="true" />

        <div className="lounge-strip__input-shell">
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
            disabled={execState === 'parsing' || execState === 'executing'}
            autoComplete="off"
            spellCheck={false}
          />
        </div>

        <div className="lounge-strip__meta">
          {showPrediction && <span className="lounge-strip__hint">tab</span>}
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
