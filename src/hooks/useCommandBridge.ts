import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useCallback, useEffect, useRef } from 'react';

import type {
  ExecutionResult,
  AppConfig,
  CommandSuggestion,
  HistoryEntry,
  MachineInfo,
  ParsedCommand,
  PermissionStatus,
  ProviderKeyStatus,
  ServiceDefinition,
} from '../types/commands';
import type { ExecutionEvent, ExecutionEventPayload } from '../types/events';

export interface CommandBridgeCallbacks {
  onParseStart: () => void;
  onParsed: (cmd: ParsedCommand) => void;
  onParseError: (err: string) => void;
  onExecutionEvent: (event: ExecutionEvent) => void;
  onExecuted: (result: ExecutionResult) => void;
  onExecuteError: (err: string) => void;
}

export function useCommandBridge(callbacks: CommandBridgeCallbacks) {
  const cbRef = useRef(callbacks);
  cbRef.current = callbacks;

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen<ExecutionEventPayload>('execution-event', (e) => {
      cbRef.current.onExecutionEvent(e.payload.event);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const parseCommand = useCallback(async (input: string) => {
    cbRef.current.onParseStart();
    try {
      const cmd = await invoke<ParsedCommand>('parse_command', { input });
      cbRef.current.onParsed(cmd);
    } catch (e) {
      cbRef.current.onParseError(String(e));
    }
  }, []);

  const suggestCommands = useCallback(async (input: string): Promise<CommandSuggestion[]> => {
    try {
      return await invoke<CommandSuggestion[]>('suggest_commands', { input });
    } catch {
      return [];
    }
  }, []);

  const approveAndExecute = useCallback(
    async (commandId: string, routeIndex: number) => {
      try {
        await invoke<ParsedCommand>('approve_command', { commandId });
        const result = await invoke<ExecutionResult>('execute_command', {
          commandId,
          routeIndex,
        });
        cbRef.current.onExecuted(result);
      } catch (e) {
        cbRef.current.onExecuteError(String(e));
      }
    },
    [],
  );

  const denyCommand = useCallback(async (commandId: string) => {
    try {
      await invoke('deny_command', { commandId });
    } catch {
      // denial is best-effort
    }
  }, []);

  const undoLast = useCallback(async () => {
    try {
      const result = await invoke<ExecutionResult>('undo_last');
      cbRef.current.onExecuted(result);
    } catch (e) {
      cbRef.current.onExecuteError(String(e));
    }
  }, []);

  const getMachineInfo = useCallback(async (): Promise<MachineInfo | null> => {
    try {
      return await invoke<MachineInfo>('get_machine_info');
    } catch {
      return null;
    }
  }, []);

  const getPermissionStatus =
    useCallback(async (): Promise<PermissionStatus | null> => {
      try {
        return await invoke<PermissionStatus>('get_permission_status');
      } catch {
        return null;
      }
    }, []);

  const getHistory = useCallback(async (): Promise<HistoryEntry[]> => {
    try {
      return await invoke<HistoryEntry[]>('get_history');
    } catch {
      return [];
    }
  }, []);

  const getServiceCatalog = useCallback(async (): Promise<ServiceDefinition[]> => {
    try {
      return await invoke<ServiceDefinition[]>('get_service_catalog');
    } catch {
      return [];
    }
  }, []);

  const getAppConfig = useCallback(async (): Promise<AppConfig | null> => {
    try {
      return await invoke<AppConfig>('get_app_config');
    } catch {
      return null;
    }
  }, []);

  const setWindowMode = useCallback(
    async (mode: 'lounge' | 'expanded'): Promise<void> => {
      try {
        await invoke('set_window_mode', { mode });
      } catch {
        // best-effort
      }
    },
    [],
  );

  const toggleAlwaysOnTop = useCallback(
    async (enabled: boolean) => {
      return invoke<AppConfig>('toggle_always_on_top', { enabled });
    },
    [],
  );

  const getProviderKeyStatus = useCallback(
    async (provider: string): Promise<ProviderKeyStatus | null> => {
      try {
        return await invoke<ProviderKeyStatus>('get_provider_key_status', { provider });
      } catch {
        return null;
      }
    },
    [],
  );

  const setProviderKey = useCallback(
    async (provider: string, key: string): Promise<void> => {
      await invoke('set_provider_key', { provider, key });
    },
    [],
  );

  const deleteProviderKey = useCallback(
    async (provider: string): Promise<void> => {
      await invoke('delete_provider_key', { provider });
    },
    [],
  );

  const debugInterpretLocal = useCallback(async (input: string): Promise<string> => {
    try {
      return await invoke<string>('debug_interpret_local', { input });
    } catch (error) {
      return `Local interpretation probe unavailable: ${String(error)}`;
    }
  }, []);

  return {
    parseCommand,
    suggestCommands,
    approveAndExecute,
    denyCommand,
    undoLast,
    getMachineInfo,
    getPermissionStatus,
    getHistory,
    getServiceCatalog,
    getAppConfig,
    setWindowMode,
    toggleAlwaysOnTop,
    getProviderKeyStatus,
    setProviderKey,
    deleteProviderKey,
    debugInterpretLocal,
  };
}
