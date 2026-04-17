import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useCallback, useEffect, useRef } from 'react';

import type {
  ExecutionResult,
  MachineInfo,
  ParsedCommand,
  PermissionStatus,
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

  // Subscribe to Rust execution-event stream.
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
    async (enabled: boolean): Promise<void> => {
      try {
        await invoke('toggle_always_on_top', { enabled });
      } catch {
        // best-effort
      }
    },
    [],
  );

  return {
    parseCommand,
    approveAndExecute,
    denyCommand,
    undoLast,
    getMachineInfo,
    getPermissionStatus,
    setWindowMode,
    toggleAlwaysOnTop,
  };
}
