import type { PermissionStatus, PermState } from '../types/commands';

interface PermissionBannerProps {
  status: PermissionStatus;
}

function stateLabel(state: PermState): string {
  switch (state) {
    case 'granted':
      return 'granted';
    case 'denied':
      return 'denied';
    case 'unknown':
      return 'unknown';
  }
}

function isWarning(state: PermState): boolean {
  return state !== 'granted';
}

export function PermissionBanner({ status }: PermissionBannerProps) {
  const accessWarn = isWarning(status.accessibility);
  const eventsWarn = isWarning(status.apple_events);

  if (!accessWarn && !eventsWarn) {
    return null;
  }

  return (
    <div className="permission-banner" role="status" aria-label="Permission status">
      <span className="permission-banner__icon" aria-hidden="true">⚠</span>
      <div className="permission-banner__items">
        {accessWarn && (
          <span className="permission-banner__item">
            Accessibility: {stateLabel(status.accessibility)} — AppleScript and UI automation
            require this permission in System Settings → Privacy & Security → Accessibility
          </span>
        )}
        {eventsWarn && (
          <span className="permission-banner__item">
            Apple Events: {stateLabel(status.apple_events)}
          </span>
        )}
      </div>
    </div>
  );
}
