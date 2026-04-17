import React, { useCallback, useEffect, useState } from 'react';
import { ExpandedConsole } from './components/ExpandedConsole';
import { LoungeStrip } from './components/LoungeStrip';
import { useCommandBridge } from './hooks/useCommandBridge';
import { useMachineState } from './hooks/useMachineState';
import type { ExecutionResult, ParsedCommand } from './types/commands';
import type { ExecutionEvent } from './types/events';

type AppMode = 'lounge' | 'expanded';
type ExecState =
  | 'idle'
  | 'parsing'
  | 'awaiting_route'
  | 'awaiting_confirm'
  | 'executing'
  | 'done'
  | 'error';

export function App() {
  const [mode, setMode] = useState<AppMode>('lounge');
  const [inputValue, setInputValue] = useState('');
  const [parsedCommand, setParsedCommand] = useState<ParsedCommand | null>(null);
  const [selectedRouteIndex, setSelectedRouteIndex] = useState<number | null>(null);
  const [execState, setExecState] = useState<ExecState>('idle');
  const [events, setEvents] = useState<ExecutionEvent[]>([]);
  const [result, setResult] = useState<ExecutionResult | null>(null);
  const [alwaysOnTop, setAlwaysOnTop] = useState(true);

  const { machineInfo } = useMachineState();

  const bridge = useCommandBridge({
    onParseStart: () => {
      setExecState('parsing');
      setEvents([]);
      setResult(null);
    },
    onParsed: (cmd) => {
      setParsedCommand(cmd);
      setMode('expanded');

      if (cmd.routes.length === 0) {
        setExecState('error');
        setResult({
          command_id: cmd.id,
          outcome: 'blocked',
          message: 'No routes resolved for this command',
          human_message: '✗ Command not recognised',
          duration_ms: 0,
        });
        return;
      }

      // Auto-select when there's only one route.
      if (cmd.routes.length === 1) {
        setSelectedRouteIndex(0);
        if (cmd.requires_approval) {
          setExecState('awaiting_confirm');
        } else {
          // Auto-execute if no approval needed.
          handleAutoExecute(cmd, 0);
        }
      } else {
        setSelectedRouteIndex(null);
        setExecState('awaiting_route');
      }
    },
    onParseError: (err) => {
      setExecState('error');
      setResult({
        command_id: '',
        outcome: 'recoverable_failure',
        message: err,
        human_message: `✗ ${err}`,
        duration_ms: 0,
      });
    },
    onExecutionEvent: (event) => {
      setEvents((prev) => [...prev, event]);
    },
    onExecuted: (res) => {
      setResult(res);
      setExecState(res.outcome === 'success' ? 'done' : 'error');
    },
    onExecuteError: (err) => {
      setExecState('error');
      setResult({
        command_id: parsedCommand?.id ?? '',
        outcome: 'recoverable_failure',
        message: err,
        human_message: `✗ ${err}`,
        duration_ms: 0,
      });
    },
  });

  // Expand/collapse window with Rust when mode changes.
  useEffect(() => {
    bridge.setWindowMode(mode);
  }, [mode, bridge]);

  function handleAutoExecute(cmd: ParsedCommand, routeIdx: number) {
    setExecState('executing');
    bridge.approveAndExecute(cmd.id, routeIdx);
  }

  const handleSubmit = useCallback(
    (value: string) => {
      if (!value.trim()) return;
      bridge.parseCommand(value);
    },
    [bridge],
  );

  const handleSelectRoute = useCallback(
    (index: number) => {
      setSelectedRouteIndex(index);
      if (!parsedCommand) return;
      if (parsedCommand.requires_approval) {
        setExecState('awaiting_confirm');
      } else {
        handleAutoExecute(parsedCommand, index);
      }
    },
    [parsedCommand, bridge],
  );

  const handleConfirm = useCallback(() => {
    if (!parsedCommand || selectedRouteIndex === null) return;
    setExecState('executing');
    bridge.approveAndExecute(parsedCommand.id, selectedRouteIndex);
  }, [parsedCommand, selectedRouteIndex, bridge]);

  const handleCancel = useCallback(() => {
    if (parsedCommand) {
      bridge.denyCommand(parsedCommand.id);
    }
    reset();
  }, [parsedCommand, bridge]);

  const handleUndo = useCallback(() => {
    setEvents([]);
    setResult(null);
    setExecState('executing');
    bridge.undoLast();
  }, [bridge]);

  const handleCollapse = useCallback(() => {
    if (parsedCommand && execState === 'awaiting_confirm') {
      bridge.denyCommand(parsedCommand.id);
    }
    reset();
  }, [parsedCommand, execState, bridge]);

  const handleToggleAlwaysOnTop = useCallback(() => {
    const next = !alwaysOnTop;
    setAlwaysOnTop(next);
    bridge.toggleAlwaysOnTop(next);
  }, [alwaysOnTop, bridge]);

  function reset() {
    setMode('lounge');
    setInputValue('');
    setParsedCommand(null);
    setSelectedRouteIndex(null);
    setExecState('idle');
    setEvents([]);
    setResult(null);
  }

  // Keyboard: Escape collapses.
  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape' && mode === 'expanded') {
        handleCollapse();
      }
    }
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [mode, handleCollapse]);

  // Suppress unused variable warning for machineInfo — it's available for
  // child components if needed in future phases.
  void machineInfo;

  return (
    <div className={`app app--${mode}`}>
      {mode === 'lounge' ? (
        <LoungeStrip
          inputValue={inputValue}
          execState={execState}
          alwaysOnTop={alwaysOnTop}
          onInput={setInputValue}
          onSubmit={handleSubmit}
          onEscape={handleCollapse}
          onToggleAlwaysOnTop={handleToggleAlwaysOnTop}
        />
      ) : (
        <>
          <LoungeStrip
            inputValue={inputValue}
            execState={execState}
            alwaysOnTop={alwaysOnTop}
            onInput={setInputValue}
            onSubmit={handleSubmit}
            onEscape={handleCollapse}
            onToggleAlwaysOnTop={handleToggleAlwaysOnTop}
          />
          <ExpandedConsole
            parsedCommand={parsedCommand}
            selectedRouteIndex={selectedRouteIndex}
            execState={execState}
            events={events}
            result={result}
            onSelectRoute={handleSelectRoute}
            onConfirm={handleConfirm}
            onCancel={handleCancel}
            onUndo={handleUndo}
            onCollapse={handleCollapse}
          />
        </>
      )}
    </div>
  );
}
