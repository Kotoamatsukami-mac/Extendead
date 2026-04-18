import { useMemo, useRef, useState } from 'react';
import type { FormEvent } from 'react';
import type { ProviderKeyStatus } from '../types/commands';

interface DeveloperPanelProps {
  status: ProviderKeyStatus | null;
  busy: boolean;
  onRefresh: () => void;
  onLink: (value: string) => Promise<void>;
  onClear: () => Promise<void>;
  onClose: () => void;
}

export function DeveloperPanel({
  status,
  busy,
  onRefresh,
  onLink,
  onClear,
  onClose,
}: DeveloperPanelProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [message, setMessage] = useState<string>('');

  const statusLabel = useMemo(() => {
    switch (status?.status) {
      case 'set':
        return 'linked';
      case 'access_denied':
        return 'access denied';
      case 'not_set':
      default:
        return 'not linked';
    }
  }, [status]);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const value = inputRef.current?.value.trim() ?? '';
    if (!value) {
      setMessage('Enter a link string first.');
      return;
    }

    try {
      await onLink(value);
      if (inputRef.current) inputRef.current.value = '';
      setMessage('Engine link stored.');
    } catch (error) {
      setMessage(String(error));
    }
  }

  async function handleClear() {
    try {
      await onClear();
      setMessage('Engine link cleared.');
    } catch (error) {
      setMessage(String(error));
    }
  }

  return (
    <section className="developer-panel" aria-label="Developer engine panel">
      <div className="developer-panel__header">
        <div>
          <span className="developer-panel__eyebrow">Developer only</span>
          <h2 className="developer-panel__title">Engine link</h2>
        </div>
        <button
          className="developer-panel__close"
          type="button"
          onClick={onClose}
          aria-label="Close developer panel"
        >
          ✕
        </button>
      </div>

      <div className="developer-panel__status-row">
        <div>
          <span className="developer-panel__label">Bridge state</span>
          <div className={`developer-panel__status developer-panel__status--${status?.status ?? 'not_set'}`}>
            {statusLabel}
          </div>
        </div>
        <button
          className="developer-panel__ghost"
          type="button"
          onClick={onRefresh}
          disabled={busy}
        >
          Refresh
        </button>
      </div>

      <form className="developer-panel__form" onSubmit={handleSubmit}>
        <label className="developer-panel__label" htmlFor="engine-link-input">
          Link string
        </label>
        <input
          id="engine-link-input"
          ref={inputRef}
          className="developer-panel__input"
          type="password"
          placeholder="Paste engine string"
          autoComplete="off"
          autoCorrect="off"
          spellCheck={false}
          disabled={busy}
        />

        <div className="developer-panel__actions">
          <button className="developer-panel__primary" type="submit" disabled={busy}>
            {busy ? 'Linking…' : 'Link engine'}
          </button>
          <button
            className="developer-panel__ghost"
            type="button"
            onClick={handleClear}
            disabled={busy}
          >
            Clear link
          </button>
        </div>
      </form>

      <p className="developer-panel__hint">
        Hidden bridge for provider access. Nothing here is shown in the normal shell.
      </p>

      {message && <div className="developer-panel__message">{message}</div>}
    </section>
  );
}
