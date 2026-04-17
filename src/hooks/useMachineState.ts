import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useState } from 'react';

import type { MachineInfo } from '../types/commands';

export function useMachineState() {
  const [machineInfo, setMachineInfo] = useState<MachineInfo | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const info = await invoke<MachineInfo>('get_machine_info');
      setMachineInfo(info);
    } catch {
      setMachineInfo(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { machineInfo, loading, refresh };
}
