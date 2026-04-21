import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useState } from 'react';

import type { PermissionStatus } from '../types/commands';

export function usePermissionStatus() {
  const [permissionStatus, setPermissionStatus] = useState<PermissionStatus | null>(null);

  const refresh = useCallback(async () => {
    try {
      const status = await invoke<PermissionStatus>('get_permission_status');
      setPermissionStatus(status);
    } catch (err) {
      console.warn('get_permission_status failed:', err);
      setPermissionStatus(null);
    }
  }, []);

  useEffect(() => {
    void refresh();

    const refreshOnFocus = () => {
      void refresh();
    };
    const interval = window.setInterval(() => {
      if (document.visibilityState === 'visible') {
        void refresh();
      }
    }, 10_000);

    window.addEventListener('focus', refreshOnFocus);
    document.addEventListener('visibilitychange', refreshOnFocus);

    return () => {
      window.clearInterval(interval);
      window.removeEventListener('focus', refreshOnFocus);
      document.removeEventListener('visibilitychange', refreshOnFocus);
    };
  }, [refresh]);

  return { permissionStatus, refresh };
}
