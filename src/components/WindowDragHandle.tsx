import "./WindowDragHandle.css";

interface WindowDragHandleProps {
  pinned: boolean;
  className?: string;
}

export function WindowDragHandle({ pinned, className }: WindowDragHandleProps) {
  const title = pinned ? "Pinned (always visible)" : "Drag to move";

  return (
    <div
      className={[
        "window-drag-handle",
        pinned ? "window-drag-handle--pinned" : "window-drag-handle--floating",
        className ?? "",
      ]
        .filter(Boolean)
        .join(" ")}
      role="button"
      tabIndex={-1}
      aria-label={title}
      title={title}
      data-tauri-drag-region=""
    >
      {/* Visual affordance: three dots = drag handle (universal UI pattern) */}
      <span className="window-drag-handle__dot" aria-hidden="true" />
      <span className="window-drag-handle__dot" aria-hidden="true" />
      <span className="window-drag-handle__dot" aria-hidden="true" />
    </div>
  );
}
