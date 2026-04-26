import { useEffect, useRef } from 'react';
import type { ChangeEvent, KeyboardEvent } from 'react';
import type { ResolvedRoute } from '../types/commands';
import { WindowDragHandle } from './WindowDragHandle';
import './LoungeStrip.css';

type StatusTone = 'neutral' | 'success' | 'error';

type StatusLine = {
  message: string;
  tone: StatusTone;
};

interface LoungeStripProps {
  inputValue: string;
  execState:
    | 'idle'
    | 'parsing'
    | 'awaiting_clarify'
    | 'awaiting_choice'
    | 'awaiting_route'
    | 'awaiting_confirm'
    | 'awaiting_key'
    | 'executing'
    | 'done'
    | 'error';
  alwaysOnTop: boolean;
  pinBusy?: boolean;
  focusTrigger: number;
  statusLine?: StatusLine | null;
  clarificationMessage?: string | null;
  clarificationSlots?: string[];
  choices?: string[];
  routes?: ResolvedRoute[];
  confirmLabel?: string | null;
  confirmDescription?: string | null;
  showApiKeyPrompt?: boolean;
  apiKeyValue?: string;
  apiKeyBusy?: boolean;
  onInput: (value: string) => void;
  onSubmit: (value: string) => void;
  onSelectChoice: (value: string) => void;
  onSelectRoute: (index: number) => void;
  onConfirm: () => void;
  onCancel: () => void;
  onToggleAlwaysOnTop: () => void;
  onApiKeyChange: (value: string) => void;
  onApiKeySubmit: () => void;
  onApiKeyCancel: () => void;
  onEscape: () => void;
}

export function LoungeStrip({
  inputValue,
  execState,
  alwaysOnTop,
  pinBusy,
  focusTrigger,
  statusLine,
  clarificationMessage,
  clarificationSlots = [],
  choices = [],
  routes = [],
  confirmLabel,
  confirmDescription,
  showApiKeyPrompt,
  apiKeyValue = '',
  apiKeyBusy,
  onInput,
  onSubmit,
  onSelectChoice,
  onSelectRoute,
  onConfirm,
  onCancel,
  onToggleAlwaysOnTop,
  onApiKeyChange,
  onApiKeySubmit,
  onApiKeyCancel,
  onEscape,
}: LoungeStripProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const keyInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (showApiKeyPrompt) {
      keyInputRef.current?.focus();
    } else {
      inputRef.current?.focus();
    }
  }, [focusTrigger, showApiKeyPrompt]);

  const isActive = execState !== 'idle';
  const isLoading = execState === 'parsing' || execState === 'executing';
  const isDone = execState === 'done';
  const isError = execState === 'error';
  const isAwaitingClarify = execState === 'awaiting_clarify';
  const isAwaitingChoice = execState === 'awaiting_choice';

  const showKeyPrompt = Boolean(showApiKeyPrompt);
  const showConfirm = !showKeyPrompt && Boolean(confirmLabel);
  const showRoutes = !showKeyPrompt && !showConfirm && routes.length > 0;
  const showChoices = !showKeyPrompt && !showConfirm && !showRoutes && choices.length > 0 && isAwaitingChoice;
  const showClarify = !showKeyPrompt && !showConfirm && !showRoutes && !showChoices && isAwaitingClarify;

  function handleKeyDown(e: KeyboardEvent<HTMLInputElement>) {
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

  function handleKeyInputDown(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Enter') {
      e.preventDefault();
      onApiKeySubmit();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onApiKeyCancel();
    }
  }

  const placeholder =
    execState === 'parsing'
      ? 'reading intent…'
      : execState === 'executing'
        ? 'running sequence…'
        : execState === 'awaiting_clarify'
          ? 'add the missing detail…'
          : execState === 'awaiting_choice'
            ? 'choose an action or refine…'
            : execState === 'awaiting_confirm'
              ? 'awaiting approval…'
              : execState === 'awaiting_route'
                ? 'select a route…'
                : execState === 'awaiting_key'
                  ? 'enter API key below…'
                  : 'tell extendead what to do';

  const stateClass = [
    'lounge-strip',
    isActive ? 'lounge-strip--active' : '',
    isLoading ? 'lounge-strip--loading' : '',
    !isActive ? 'lounge-strip--ready' : '',
    isDone ? 'lounge-strip--done' : '',
    isError ? 'lounge-strip--error' : '',
    isAwaitingClarify ? 'lounge-strip--clarify' : '',
    isAwaitingChoice ? 'lounge-strip--choice' : '',
    !alwaysOnTop ? 'lounge-strip--floating' : '',
  ].filter(Boolean).join(' ');

  return (
    <div className={stateClass}>
      <div className="lounge-strip__body">
        <WindowDragHandle
          locked={alwaysOnTop}
          className="lounge-strip__drag-handle"
        />
        <span className="lounge-strip__marker" aria-hidden="true" />
        <div className="lounge-strip__input-shell">
          <input
            ref={inputRef}
            className="lounge-strip__input"
            type="text"
            value={inputValue}
            placeholder={placeholder}
            onChange={handleChange}
            onKeyDown={handleKeyDown}
            disabled={isLoading || showKeyPrompt}
            autoComplete="off"
            spellCheck={false}
          />

          {showKeyPrompt && (
            <div className="lounge-strip__panel lounge-strip__panel--key" role="group" aria-label="API key required">
              <span className="lounge-strip__panel-message">API key required for broader interpretation.</span>
              <div className="lounge-strip__key-row">
                <input
                  ref={keyInputRef}
                  className="lounge-strip__key-input"
                  type="password"
                  placeholder="API key"
                  value={apiKeyValue}
                  onChange={(e) => onApiKeyChange(e.target.value)}
                  onKeyDown={handleKeyInputDown}
                  disabled={apiKeyBusy}
                  autoComplete="off"
                  spellCheck={false}
                />
                <button
                  type="button"
                  className="lounge-strip__key-btn lounge-strip__key-btn--primary"
                  onClick={onApiKeySubmit}
                  disabled={apiKeyBusy || !apiKeyValue.trim()}
                >
                  Save
                </button>
                <button
                  type="button"
                  className="lounge-strip__key-btn"
                  onClick={onApiKeyCancel}
                  disabled={apiKeyBusy}
                >
                  Cancel
                </button>
              </div>
            </div>
          )}

          {showConfirm && (
            <div className="lounge-strip__panel lounge-strip__panel--confirm" role="group" aria-label="Approval required">
              <div className="lounge-strip__panel-message">
                <span className="lounge-strip__confirm-label">{confirmLabel}</span>
                {confirmDescription && (
                  <span className="lounge-strip__confirm-desc">{confirmDescription}</span>
                )}
              </div>
              <div className="lounge-strip__confirm-actions">
                <button
                  type="button"
                  className="lounge-strip__confirm-btn lounge-strip__confirm-btn--yes"
                  onClick={onConfirm}
                >
                  Y
                </button>
                <button
                  type="button"
                  className="lounge-strip__confirm-btn lounge-strip__confirm-btn--no"
                  onClick={onCancel}
                >
                  N
                </button>
              </div>
            </div>
          )}

          {showRoutes && (
            <div className="lounge-strip__panel lounge-strip__panel--routes" role="group" aria-label="Choose a route">
              <span className="lounge-strip__panel-message">Choose a route to continue.</span>
              <div className="lounge-strip__route-list">
                {routes.map((route, index) => (
                  <button
                    key={`${route.label}-${index}`}
                    type="button"
                    className="lounge-strip__route"
                    onMouseDown={(e) => {
                      e.preventDefault();
                      onSelectRoute(index);
                    }}
                  >
                    <span className="lounge-strip__route-label">{route.label}</span>
                    <span className="lounge-strip__route-desc">{route.description}</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          {showChoices && (
            <div className="lounge-strip__panel lounge-strip__panel--choices" role="group" aria-label="Choose an action">
              <span className="lounge-strip__panel-message">Choose an action to continue.</span>
              <div className="lounge-strip__choices">
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
            </div>
          )}

          {showClarify && (
            <div className="lounge-strip__panel lounge-strip__panel--clarify" role="status" aria-live="polite">
              <span className="lounge-strip__clarify-message">
                {clarificationMessage || 'Need one more detail before I can run this.'}
              </span>
              {clarificationSlots.length > 0 && (
                <span className="lounge-strip__clarify-slots">
                  Needed: {clarificationSlots.map((slot) => slot.replace(/_/g, ' ')).join(', ')}
                </span>
              )}
            </div>
          )}
        </div>

        {statusLine && (
          <span
            className={
              statusLine.tone === 'neutral'
                ? 'lounge-strip__status'
                : `lounge-strip__status lounge-strip__status--${statusLine.tone}`
            }
          >
            {statusLine.message}
          </span>
        )}

        <div className="lounge-strip__meta">
          <span className={`lounge-strip__pin-state ${alwaysOnTop ? 'lounge-strip__pin-state--active' : ''}`}>
            {alwaysOnTop ? 'Pinned' : 'Floating'}
          </span>
          <button
            className={`lounge-strip__pin ${alwaysOnTop ? 'lounge-strip__pin--active' : ''}`}
            onClick={onToggleAlwaysOnTop}
            disabled={pinBusy}
            title={alwaysOnTop ? 'Unpin window' : 'Pin window on top'}
            aria-label={alwaysOnTop ? 'Unpin window' : 'Pin window on top'}
            aria-pressed={alwaysOnTop}
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
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
