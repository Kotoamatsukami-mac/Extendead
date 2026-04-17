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
    refresh();
  }, [refresh]);

  return { permissionStatus, refresh };
}
