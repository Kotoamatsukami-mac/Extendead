import "./WindowDragHandle.css";

interface WindowDragHandleProps {
  locked: boolean;
  titleWhenLocked?: string;
  titleWhenUnlocked?: string;
  className?: string;
}

export function WindowDragHandle({
  locked,
  titleWhenLocked = "Pinned: unpin to move",
  titleWhenUnlocked = "Drag shell",
  className,
}: WindowDragHandleProps) {
  const title = locked ? titleWhenLocked : titleWhenUnlocked;

  return (
    <div
      className={[
        "window-drag-handle",
        locked ? "window-drag-handle--locked" : "window-drag-handle--free",
        className ?? "",
      ]
        .filter(Boolean)
        .join(" ")}
      role="button"
      tabIndex={-1}
      aria-label={title}
      title={title}
      {...(locked ? {} : { "data-tauri-drag-region": "" })}
    >
      <span className="window-drag-handle__dot" aria-hidden="true" />
      <span className="window-drag-handle__dot" aria-hidden="true" />
      <span className="window-drag-handle__dot" aria-hidden="true" />
    </div>
  );
}
