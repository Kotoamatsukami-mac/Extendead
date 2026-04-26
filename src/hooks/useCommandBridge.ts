import { invoke } from '@tauri-apps/api/core';
import { useCallback, useRef } from 'react';

import type {
  ExecutionResult,
  AppConfig,
  ParsedCommand,
} from '../types/commands';

export interface CommandBridgeCallbacks {
  onParseStart: () => void;
  onParsed: (cmd: ParsedCommand) => void;
  onParseError: (err: string) => void;
  onExecuted: (result: ExecutionResult) => void;
  onExecuteError: (err: string) => void;
}

export function useCommandBridge(callbacks: CommandBridgeCallbacks) {
  const cbRef = useRef(callbacks);
  cbRef.current = callbacks;

  const parseCommand = useCallback(async (input: string) => {
    cbRef.current.onParseStart();
    try {
      const cmd = await invoke<ParsedCommand>('parse_command', { input });
      cbRef.current.onParsed(cmd);
    } catch (e) {
      cbRef.current.onParseError(String(e));
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

  const setProviderKey = useCallback(
    async (provider: string, key: string): Promise<void> => {
      await invoke('set_provider_key', { provider, key });
    },
    [],
  );

  return {
    parseCommand,
    approveAndExecute,
    denyCommand,
    getAppConfig,
    setWindowMode,
    toggleAlwaysOnTop,
    setProviderKey,
  };
}
