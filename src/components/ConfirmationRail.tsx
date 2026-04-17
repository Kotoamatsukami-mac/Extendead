import type { KeyboardEvent } from 'react';

interface ConfirmationRailProps {
  label: string;
  description: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmationRail({
  label,
  description,
  onConfirm,
  onCancel,
}: ConfirmationRailProps) {
  function handleKeyDown(e: KeyboardEvent<HTMLDivElement>) {
    if (e.key === 'y' || e.key === 'Y') {
      e.preventDefault();
      onConfirm();
    } else if (e.key === 'n' || e.key === 'N' || e.key === 'Escape') {
      e.preventDefault();
      onCancel();
    }
  }

  return (
    <div className="confirm-rail" onKeyDown={handleKeyDown} tabIndex={0}>
      <div className="confirm-rail__intent">
        <span className="confirm-rail__label">{label}</span>
        <span className="confirm-rail__desc">{description}</span>
      </div>

      <div className="confirm-rail__actions">
        <button
          className="confirm-rail__btn confirm-rail__btn--confirm"
          onClick={onConfirm}
          title="Confirm (Y)"
        >
          Y
        </button>
        <button
          className="confirm-rail__btn confirm-rail__btn--cancel"
          onClick={onCancel}
          title="Cancel (N)"
        >
          N
        </button>
      </div>
    </div>
  );
}
