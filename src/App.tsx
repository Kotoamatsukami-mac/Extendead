import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { DeveloperPanel } from './components/DeveloperPanel';
import { ExpandedConsole } from './components/ExpandedConsole';
import { LoungeStrip } from './components/LoungeStrip';
import { useCommandBridge } from './hooks/useCommandBridge';
import { usePermissionStatus } from './hooks/usePermissionStatus';
import type {
  CommandSuggestion,
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
  | 'awaiting_clarify'
  | 'awaiting_choice'
  | 'awaiting_route'
  | 'awaiting_confirm'
  | 'executing'
  | 'done'
  | 'error';

const DEV_PANEL_UNLOCK = '//engine';

const BUILT_IN_PREDICTIONS = [
  'open youtube',
  'open safari',
  'close safari',
  'open chrome',
  'open finder',
  'open slack',
  'display settings',
  'downloads',
  'set volume to 40',
  DEV_PANEL_UNLOCK,
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
  const [suggestions, setSuggestions] = useState<CommandSuggestion[]>([]);
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
  const suggestionRequestRef = useRef(0);

  const { permissionStatus, refresh: refreshPermissionStatus } = usePermissionStatus();

  const parsedCommandRef = useRef<ParsedCommand | null>(null);
  parsedCommandRef.current = parsedCommand;

  function showInlineFeedback(message: string, type: 'success' | 'error', duration: number) {
    window.clearTimeout(feedbackTimerRef.current);
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
      setResultFeedback(null);
      window.clearTimeout(feedbackTimerRef.current);
      oneShotRef.current = false;
    },
    onParsed: (cmd) => {
      setParsedCommand(cmd);

      if (cmd.interpretation_decision === 'clarify') {
        setMode('lounge');
        setSelectedRouteIndex(null);
        setExecState('awaiting_clarify');
        return;
      }

      if (cmd.interpretation_decision === 'offer_choices' && (cmd.choices?.length ?? 0) > 0) {
        setMode('lounge');
        setSelectedRouteIndex(null);
        setExecState('awaiting_choice');
        return;
      }

      if (cmd.routes.length === 0) {
        setExecState('error');
        showInlineFeedback(getUnresolvedMessage(cmd), 'error', 2600);
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
      } else {
        setMode('expanded');
        setSelectedRouteIndex(null);
        setExecState('awaiting_route');
      }
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
    bridge.getAppConfig().then((config) => {
      if (config) {
        setAlwaysOnTop(config.always_on_top);
      }
    });
    void refreshPermissionStatus();
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

  useEffect(() => {
    const normalized = inputValue.trim();
    if (normalized.length < 2 || !!resultFeedback) {
      setSuggestions([]);
      return;
    }

    const requestId = suggestionRequestRef.current + 1;
    suggestionRequestRef.current = requestId;

    const timer = window.setTimeout(() => {
      bridge.suggestCommands(normalized).then((nextSuggestions) => {
        if (suggestionRequestRef.current === requestId) {
          setSuggestions(nextSuggestions);
        }
      });
    }, 90);

    return () => window.clearTimeout(timer);
  }, [bridge, inputValue, resultFeedback]);

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
    setResultFeedback(null);
    window.clearTimeout(feedbackTimerRef.current);
    void refreshDeveloperStatus();
  }, [refreshDeveloperStatus]);

  const handleSubmit = useCallback(
    (value: string) => {
      const trimmed = value.trim();
      if (!trimmed) return;

      if (trimmed.toLowerCase() === DEV_PANEL_UNLOCK) {
        handleOpenEngineLink();
        setInputValue('');
        return;
      }

      setInputValue('');
      bridge.parseCommand(trimmed);
    },
    [bridge, handleOpenEngineLink],
  );

  const handleAcceptPrediction = useCallback(() => {
    if (prediction) {
      setInputValue(prediction);
    }
  }, [prediction]);

  const handleApplySuggestion = useCallback((value: string) => {
    setInputValue(value);
  }, []);

  const handleSelectChoice = useCallback((value: string) => {
    const trimmed = value.trim();
    if (!trimmed) return;
    setInputValue('');
    bridge.parseCommand(trimmed);
  }, [bridge]);

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
    void bridge.toggleAlwaysOnTop(next).then(async () => {
      const config = await bridge.getAppConfig();
      if (config) {
        setAlwaysOnTop(config.always_on_top);
      }
    });
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

  const handleInspectLocal = useCallback(async (value: string) => {
    return bridge.debugInterpretLocal(value);
  }, [bridge]);

  const handleInputChange = useCallback((value: string) => {
    setInputValue(value);
    if (resultFeedback) {
      window.clearTimeout(feedbackTimerRef.current);
      setResultFeedback(null);
      setExecState('idle');
      setParsedCommand(null);
      setResult(null);
    }
  }, [resultFeedback]);

  function reset() {
    window.clearTimeout(feedbackTimerRef.current);
    setMode('lounge');
    setInputValue('');
    setParsedCommand(null);
    setSelectedRouteIndex(null);
    setExecState('idle');
    setEvents([]);
    setResult(null);
    setAutoExec(null);
    setShowDeveloperPanel(false);
    setResultFeedback(null);
    oneShotRef.current = false;
    setFocusTrigger((n) => n + 1);
  }

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape' && mode === 'expanded') {
        e.preventDefault();
        handleCollapse();
      } else if (execState === 'awaiting_confirm') {
        if (e.key === 'Enter') {
          e.preventDefault();
          handleConfirm();
        } else if (e.key === 'Escape') {
          e.preventDefault();
          handleCancel();
        }
      }
    }
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [mode, execState, handleCollapse, handleConfirm, handleCancel]);

  useEffect(() => {
    const refreshConfig = () => {
      void bridge.getAppConfig().then((config) => {
        if (config) {
          setAlwaysOnTop(config.always_on_top);
        }
      });
    };

    refreshConfig();
    const interval = window.setInterval(() => {
      if (document.visibilityState === 'visible') {
        refreshConfig();
      }
    }, 15_000);

    window.addEventListener('focus', refreshConfig);
    document.addEventListener('visibilitychange', refreshConfig);
    return () => {
      window.clearInterval(interval);
      window.removeEventListener('focus', refreshConfig);
      document.removeEventListener('visibilitychange', refreshConfig);
    };
  }, [bridge]);

  return (
    <div className={`app app--${mode}`}>
      <div className="app__surface" data-tauri-drag-region>
        <LoungeStrip
          inputValue={inputValue}
          prediction={prediction}
          suggestions={suggestions}
          clarificationMessage={
            execState === 'awaiting_clarify'
              ? (parsedCommand?.clarification_message ?? parsedCommand?.unresolved_message ?? null)
              : null
          }
          clarificationSlots={execState === 'awaiting_clarify' ? (parsedCommand?.clarification_slots ?? []) : []}
          choices={execState === 'awaiting_choice' ? (parsedCommand?.choices ?? []) : []}
          execState={execState}
          alwaysOnTop={alwaysOnTop}
          focusTrigger={focusTrigger}
          resultFeedback={resultFeedback}
          embedded={mode === 'expanded'}
          onInput={handleInputChange}
          onSubmit={handleSubmit}
          onAcceptPrediction={handleAcceptPrediction}
          onApplySuggestion={handleApplySuggestion}
          onSelectChoice={handleSelectChoice}
          onEscape={handleCollapse}
          onToggleAlwaysOnTop={handleToggleAlwaysOnTop}
          onOpenEngineLink={mode === 'lounge' ? handleOpenEngineLink : undefined}
        />
        {mode === 'expanded' && (
          showDeveloperPanel ? (
            <DeveloperPanel
              status={primaryProviderStatus}
              busy={developerBusy}
              onRefresh={() => void refreshDeveloperStatus()}
              onLink={handleLinkPrimaryEngine}
              onClear={handleClearPrimaryEngine}
              onInspectLocal={handleInspectLocal}
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
          )
        )}
      </div>
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

function getUnresolvedMessage(cmd: ParsedCommand): string {
  switch (cmd.unresolved_code) {
    case 'unsupported_command':
      return 'That command is outside current local coverage.';
    case 'unsupported_service':
      return 'That service is outside current local coverage.';
    case 'browser_not_installed':
      return cmd.unresolved_message?.trim() || 'That browser is not installed on this Mac.';
    case 'app_not_installed':
      return cmd.unresolved_message?.trim() || 'That app is not installed on this Mac.';
    case 'path_not_found':
      return cmd.unresolved_message?.trim() || 'That path does not exist.';
    case 'source_path_not_found':
      return cmd.unresolved_message?.trim() || 'The source path does not exist.';
    case 'base_path_unresolved':
      return cmd.unresolved_message?.trim() || 'I could not resolve where to create that folder.';
    case 'target_already_exists':
      return cmd.unresolved_message?.trim() || 'That target already exists.';
    case 'destination_path_unresolved':
      return cmd.unresolved_message?.trim() || 'I could not resolve the destination path.';
    case 'destination_parent_missing':
      return cmd.unresolved_message?.trim() || 'The destination parent folder does not exist.';
    case 'permanent_delete_blocked':
      return cmd.unresolved_message?.trim() || 'Permanent delete is blocked. Use trash <path> instead.';
    default:
      break;
  }

  if (cmd.unresolved_message?.trim()) {
    return cmd.unresolved_message.trim();
  }

  switch (cmd.kind) {
    case 'unknown':
      return 'That command is outside current local coverage.';
    case 'app_control':
      return 'I could not resolve that app action on this Mac.';
    case 'settings':
      return 'That settings route is not available yet.';
    case 'ui_automation':
      return 'That UI automation route is not available yet.';
    default:
      return 'I could not resolve a safe local route for that command.';
  }
}
