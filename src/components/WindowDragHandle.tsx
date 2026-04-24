import { useCallback } from 'react';
import type { PointerEvent as ReactPointerEvent } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

import './WindowDragHandle.css';

interface WindowDragHandleProps {
  locked: boolean;
  titleWhenLocked?: string;
  titleWhenUnlocked?: string;
  className?: string;
}

export function WindowDragHandle({
  locked,
  titleWhenLocked = 'Pinned: unpin to move',
  titleWhenUnlocked = 'Drag shell',
  className,
}: WindowDragHandleProps) {
  const handlePointerDown = useCallback(
    (e: ReactPointerEvent<HTMLDivElement>) => {
      if (locked) return;
      if (e.button !== 0) return;

      // Prevent accidental text selection and keep the gesture crisp.
      e.preventDefault();
      void getCurrentWindow().startDragging();
    },
    [locked],
  );

  const title = locked ? titleWhenLocked : titleWhenUnlocked;

  return (
    <div
      className={[
        'window-drag-handle',
        locked ? 'window-drag-handle--locked' : 'window-drag-handle--free',
        className ?? '',
      ].filter(Boolean).join(' ')}
      onPointerDown={handlePointerDown}
      role="button"
      tabIndex={-1}
      aria-label={title}
      title={title}
      data-tauri-drag-region={locked ? undefined : 'true'}
    >
      <span className="window-drag-handle__dot" aria-hidden="true" />
      <span className="window-drag-handle__dot" aria-hidden="true" />
      <span className="window-drag-handle__dot" aria-hidden="true" />
    </div>
  );
}
