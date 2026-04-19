import { useCallback, useEffect, useRef, useState } from 'react';
import { ExpandedConsole } from './components/ExpandedConsole';
import { LoungeStrip } from './components/LoungeStrip';
import { useCommandBridge } from './hooks/useCommandBridge';
import { useMachineState } from './hooks/useMachineState';
import { usePermissionStatus } from './hooks/usePermissionStatus';
import type { CommandKind, ExecutionResult, HistoryEntry, ParsedCommand } from './types/commands';
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

function zeroRouteMessage(kind: CommandKind): string {
  switch (kind) {
    case 'app_control':
      return '✗ App not available on this Mac';
    case 'mixed_workflow':
      return '✗ No valid route available on this Mac';
    default:
      return '✗ Command not recognised';
  }
}

export function App() {
  const [mode, setMode] = useState<AppMode>('lounge');
  const [inputValue, setInputValue] = useState('');
  const [parsedCommand, setParsedCommand] = useState<ParsedCommand | null>(null);
  const [selectedRouteIndex, setSelectedRouteIndex] = useState<number | null>(null);
  const [execState, setExecState] = useState<ExecState>('idle');
  const [events, setEvents] = useState<ExecutionEvent[]>([]);
  const [result, setResult] = useState<ExecutionResult | null>(null);
  const [alwaysOnTop, setAlwaysOnTop] = useState(true);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [focusTrigger, setFocusTrigger] = useState(0);

  const [autoExec, setAutoExec] = useState<{
    cmd: ParsedCommand;
    routeIdx: number;
  } | null>(null);

  const { machineInfo } = useMachineState();
  const { permissionStatus } = usePermissionStatus();

  const parsedCommandRef = useRef<ParsedCommand | null>(null);
  parsedCommandRef.current = parsedCommand;

  const bridge = useCommandBridge({
    onParseStart: () => {
      setExecState('parsing');
      setEvents([]);
      setResult(null);
      setAutoExec(null);
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
          human_message: zeroRouteMessage(cmd.kind),
          duration_ms: 0,
        });
        return;
      }

      if (cmd.routes.length === 1) {
        setSelectedRouteIndex(0);
        if (cmd.requires_approval) {
          setExecState('awaiting_confirm');
        } else {
          setAutoExec({ cmd, routeIdx: 0 });
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
      bridge.getHistory().then(setHistory);
    },
    onExecuteError: (err) => {
      setExecState('error');
      setResult({
        command_id: parsedCommandRef.current?.id ?? '',
        outcome: 'recoverable_failure',
        message: err,
        human_message: `✗ ${err}`,
        duration_ms: 0,
      });
    },
  });

  useEffect(() => {
    bridge.getHistory().then(setHistory);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    bridge.setWindowMode(mode);
  }, [mode, bridge]);

  useEffect(() => {
    if (!autoExec) return;
    setAutoExec(null);
    setExecState('executing');
    bridge.approveAndExecute(autoExec.cmd.id, autoExec.routeIdx);
  }, [autoExec, bridge]);

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
        setExecState('executing');
        bridge.approveAndExecute(parsedCommand.id, index);
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
    setAutoExec(null);
    setFocusTrigger((n) => n + 1);
  }

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape' && mode === 'expanded') {
        handleCollapse();
      } else if (execState === 'awaiting_confirm') {
        if (e.key === 'y' || e.key === 'Y') {
          e.preventDefault();
          handleConfirm();
        } else if (e.key === 'n' || e.key === 'N') {
          e.preventDefault();
          handleCancel();
        }
      }
    }
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [mode, execState, handleCollapse, handleConfirm, handleCancel]);

  void machineInfo;

  return (
    <div className={`app app--${mode}`}>
      {mode === 'lounge' ? (
        <LoungeStrip
          inputValue={inputValue}
          execState={execState}
          alwaysOnTop={alwaysOnTop}
          focusTrigger={focusTrigger}
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
            focusTrigger={focusTrigger}
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
            permissionStatus={permissionStatus}
            history={history}
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
