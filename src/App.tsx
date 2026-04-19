import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { DeveloperPanel } from './components/DeveloperPanel';
import { ExpandedConsole } from './components/ExpandedConsole';
import { LoungeStrip } from './components/LoungeStrip';
import { useCommandBridge } from './hooks/useCommandBridge';
import { useMachineState } from './hooks/useMachineState';
import { usePermissionStatus } from './hooks/usePermissionStatus';
import type {
  ExecutionResult,
  HistoryEntry,
  ParsedCommand,
  ProviderKeyStatus,
  ResultFeedback,
} from './types/commands';
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

const BUILT_IN_PREDICTIONS = [
  'open youtube',
  'open safari',
  'open chrome',
  'open firefox',
  'open brave',
  'open arc',
  'slack',
  'display settings',
  'downloads',
  'set volume to 40',
] as const;

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
  const [showDeveloperPanel, setShowDeveloperPanel] = useState(false);
  const [primaryProviderStatus, setPrimaryProviderStatus] = useState<ProviderKeyStatus | null>(null);
  const [developerBusy, setDeveloperBusy] = useState(false);
  const [resultFeedback, setResultFeedback] = useState<ResultFeedback | null>(null);
  const [focusTrigger, setFocusTrigger] = useState(0);

  const [autoExec, setAutoExec] = useState<{
    cmd: ParsedCommand;
    routeIdx: number;
  } | null>(null);

  const oneShotRef = useRef(false);
  const feedbackTimerRef = useRef<number>(0);

  const { machineInfo } = useMachineState();
  const { permissionStatus } = usePermissionStatus();

  const parsedCommandRef = useRef<ParsedCommand | null>(null);
  parsedCommandRef.current = parsedCommand;

  function clearInlineFeedback() {
    window.clearTimeout(feedbackTimerRef.current);
    setResultFeedback(null);
  }

  function showInlineFeedback(message: string, type: 'success' | 'error', duration: number) {
    clearInlineFeedback();
    setResultFeedback({ message, type });
    feedbackTimerRef.current = window.setTimeout(() => {
      setResultFeedback(null);
      setExecState('idle');
      setParsedCommand(null);
      setResult(null);
      setFocusTrigger((n) => n + 1);
    }, duration);
  }

  const bridge = useCommandBridge({
    onParseStart: () => {
      setShowDeveloperPanel(false);
      setExecState('parsing');
      setEvents([]);
      setResult(null);
      setAutoExec(null);
      clearInlineFeedback();
      oneShotRef.current = false;
    },
    onParsed: (cmd) => {
      setParsedCommand(cmd);

      if (cmd.routes.length === 0) {
        setExecState('error');
        showInlineFeedback('Command not recognised', 'error', 3500);
        return;
      }

      if (cmd.routes.length === 1) {
        setSelectedRouteIndex(0);
        if (cmd.requires_approval) {
          setMode('expanded');
          setExecState('awaiting_confirm');
        } else {
          oneShotRef.current = true;
          setAutoExec({ cmd, routeIdx: 0 });
        }
        return;
      }

      setMode('expanded');
      setSelectedRouteIndex(null);
      setExecState('awaiting_route');
    },
    onParseError: (err) => {
      setExecState('error');
      showInlineFeedback(err, 'error', 3500);
    },
    onExecutionEvent: (event) => {
      setEvents((prev) => [...prev, event]);
    },
    onExecuted: (res) => {
      setResult(res);
      const isSuccess = res.outcome === 'success';
      setExecState(isSuccess ? 'done' : 'error');

      if (oneShotRef.current) {
        oneShotRef.current = false;
        const msg = isSuccess
          ? (res.human_message || '✓ Done')
          : (res.human_message || '✗ Failed');
        showInlineFeedback(msg, isSuccess ? 'success' : 'error', isSuccess ? 2000 : 3500);
      }

      bridge.getHistory().then(setHistory);
    },
    onExecuteError: (err) => {
      setExecState('error');

      if (oneShotRef.current) {
        oneShotRef.current = false;
        showInlineFeedback(err, 'error', 3500);
      } else {
        setResult({
          command_id: parsedCommandRef.current?.id ?? '',
          outcome: 'recoverable_failure',
          message: err,
          human_message: `✗ ${err}`,
          duration_ms: 0,
        });
      }
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

  const prediction = useMemo(
    () => getPrediction(inputValue, history),
    [inputValue, history],
  );

  const refreshDeveloperStatus = useCallback(async () => {
    setDeveloperBusy(true);
    try {
      const status = await bridge.getProviderKeyStatus('perplexity');
      setPrimaryProviderStatus(status);
    } finally {
      setDeveloperBusy(false);
    }
  }, [bridge]);

  const handleOpenEngineLink = useCallback(() => {
    setShowDeveloperPanel(true);
    setMode('expanded');
    setParsedCommand(null);
    setSelectedRouteIndex(null);
    setExecState('idle');
    setEvents([]);
    setResult(null);
    setAutoExec(null);
    clearInlineFeedback();
    void refreshDeveloperStatus();
  }, [refreshDeveloperStatus]);

  const handleSubmit = useCallback(
    (value: string) => {
      const trimmed = value.trim();
      if (!trimmed) return;

      setInputValue('');
      bridge.parseCommand(trimmed);
    },
    [bridge],
  );

  const handleAcceptPrediction = useCallback(() => {
    if (prediction) {
      setInputValue(prediction);
    }
  }, [prediction]);

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

  const handleLinkPrimaryEngine = useCallback(
    async (value: string) => {
      setDeveloperBusy(true);
      try {
        await bridge.setProviderKey('perplexity', value);
        const status = await bridge.getProviderKeyStatus('perplexity');
        setPrimaryProviderStatus(status);
      } finally {
        setDeveloperBusy(false);
      }
    },
    [bridge],
  );

  const handleClearPrimaryEngine = useCallback(async () => {
    setDeveloperBusy(true);
    try {
      await bridge.deleteProviderKey('perplexity');
      const status = await bridge.getProviderKeyStatus('perplexity');
      setPrimaryProviderStatus(status);
    } finally {
      setDeveloperBusy(false);
    }
  }, [bridge]);

  const handleInputChange = useCallback((value: string) => {
    setInputValue(value);
    if (resultFeedback) {
      clearInlineFeedback();
      setExecState('idle');
      setParsedCommand(null);
      setResult(null);
    }
  }, [resultFeedback]);

  function reset() {
    clearInlineFeedback();
    setMode('lounge');
    setInputValue('');
    setParsedCommand(null);
    setSelectedRouteIndex(null);
    setExecState('idle');
    setEvents([]);
    setResult(null);
    setAutoExec(null);
    setShowDeveloperPanel(false);
    oneShotRef.current = false;
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
          prediction={prediction}
          execState={execState}
          alwaysOnTop={alwaysOnTop}
          focusTrigger={focusTrigger}
          resultFeedback={resultFeedback}
          onInput={handleInputChange}
          onSubmit={handleSubmit}
          onAcceptPrediction={handleAcceptPrediction}
          onEscape={handleCollapse}
          onToggleAlwaysOnTop={handleToggleAlwaysOnTop}
          onOpenEngineLink={handleOpenEngineLink}
        />
      ) : (
        <div className="app__panel">
          <LoungeStrip
            inputValue={inputValue}
            prediction={prediction}
            execState={execState}
            alwaysOnTop={alwaysOnTop}
            focusTrigger={focusTrigger}
            resultFeedback={resultFeedback}
            embedded
            onInput={handleInputChange}
            onSubmit={handleSubmit}
            onAcceptPrediction={handleAcceptPrediction}
            onEscape={handleCollapse}
            onToggleAlwaysOnTop={handleToggleAlwaysOnTop}
          />
          {showDeveloperPanel ? (
            <DeveloperPanel
              status={primaryProviderStatus}
              busy={developerBusy}
              onRefresh={() => void refreshDeveloperStatus()}
              onLink={handleLinkPrimaryEngine}
              onClear={handleClearPrimaryEngine}
              onClose={handleCollapse}
            />
          ) : (
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
          )}
        </div>
      )}
    </div>
  );
}

function getPrediction(inputValue: string, history: HistoryEntry[]): string {
  const normalized = inputValue.trim().toLowerCase();
  if (!normalized) return '';

  const candidates = [...history.map((entry) => entry.command.raw_input), ...BUILT_IN_PREDICTIONS];
  const seen = new Set<string>();

  for (const candidate of candidates) {
    const lowered = candidate.toLowerCase();
    if (seen.has(lowered)) continue;
    seen.add(lowered);

    if (lowered.startsWith(normalized) && lowered !== normalized) {
      return candidate;
    }
  }

  return '';
}
