import { useRef, useEffect } from 'react';
import type { ChangeEvent, KeyboardEvent } from 'react';
import './LoungeStrip.css';

interface LoungeStripProps {
  inputValue: string;
  execState: 'idle' | 'parsing' | 'awaiting_route' | 'awaiting_confirm' | 'executing' | 'done' | 'error';
  alwaysOnTop: boolean;
  onInput: (value: string) => void;
  onSubmit: (value: string) => void;
  onEscape: () => void;
  onToggleAlwaysOnTop: () => void;
}

export function LoungeStrip({
  inputValue,
  execState,
  alwaysOnTop,
  onInput,
  onSubmit,
  onEscape,
  onToggleAlwaysOnTop,
}: LoungeStripProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const isActive = execState !== 'idle';
  const isLoading = execState === 'parsing' || execState === 'executing';

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

  const placeholder =
    execState === 'parsing'
      ? 'Parsing…'
      : execState === 'executing'
        ? 'Executing…'
        : 'Type a command…';

  return (
    <div
      className={`lounge-strip ${isActive ? 'lounge-strip--active' : ''} ${isLoading ? 'lounge-strip--loading' : ''}`}
      data-tauri-drag-region
    >
      <div className="lounge-strip__inner">
        <span className="lounge-strip__icon" aria-hidden="true">
          {isLoading ? '⟳' : '◈'}
        </span>

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

        <button
          className={`lounge-strip__pin ${alwaysOnTop ? 'lounge-strip__pin--active' : ''}`}
          onClick={onToggleAlwaysOnTop}
          title={alwaysOnTop ? 'Unpin window' : 'Pin window on top'}
          aria-label={alwaysOnTop ? 'Unpin window' : 'Pin window on top'}
        >
          ⊛
        </button>
      </div>
    </div>
  );
}
