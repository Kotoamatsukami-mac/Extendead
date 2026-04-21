import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { DeveloperPanel } from './components/DeveloperPanel';
import { ExpandedConsole } from './components/ExpandedConsole';
import { LoungeStrip } from './components/LoungeStrip';
import { useCommandBridge } from './hooks/useCommandBridge';
import { useMachineState } from './hooks/useMachineState';
import { usePermissionStatus } from './hooks/usePermissionStatus';
import type {
  CommandSuggestion,
  ExecutionResult,
  HistoryEntry,
  MachineInfo,
  ParsedCommand,
  ProviderKeyStatus,
  ResultFeedback,
  ServiceDefinition,
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
  const [serviceCatalog, setServiceCatalog] = useState<ServiceDefinition[]>([]);
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
    bridge.getServiceCatalog().then(setServiceCatalog);
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

  const suggestions = useMemo(
    () => getCommandSuggestions(inputValue, machineInfo, serviceCatalog),
    [inputValue, machineInfo, serviceCatalog],
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

  void machineInfo;

  return (
    <div className={`app app--${mode}`}>
      <LoungeStrip
        inputValue={inputValue}
        prediction={prediction}
        suggestions={suggestions}
        execState={execState}
        alwaysOnTop={alwaysOnTop}
        focusTrigger={focusTrigger}
        resultFeedback={resultFeedback}
        embedded={mode === 'expanded'}
        onInput={handleInputChange}
        onSubmit={handleSubmit}
        onAcceptPrediction={handleAcceptPrediction}
        onApplySuggestion={handleApplySuggestion}
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

function getCommandSuggestions(
  inputValue: string,
  machineInfo: MachineInfo | null,
  serviceCatalog: ServiceDefinition[],
): CommandSuggestion[] {
  const normalized = normalizePhrase(inputValue);
  if (normalized.length < 2) return [];

  const commands: CommandSuggestion[] = [];
  const appNames = getInstalledAppNames(machineInfo);
  const appQuery = getRemainderAfterPrefix(normalized, ['close ', 'quit ', 'exit ', 'open ', 'launch ', 'start ', 'run ']);
  const serviceQuery = getServiceQuery(normalized);
  const titleCaseQuery = titleCase(appQuery);

  if (startsWithAny(normalized, ['close ', 'quit ', 'exit '])) {
    const matchingApps = filterAppNames(appNames, appQuery);
    for (const app of matchingApps) {
      commands.push({
        id: `close-${app.toLowerCase()}`,
        family: 'close app',
        canonical: `close ${app.toLowerCase()}`,
        detail: `quit ${app}`,
      });
    }
  }

  if (startsWithAny(normalized, ['open ', 'launch ', 'start ', 'run '])) {
    if (looksLikePath(appQuery)) {
      commands.push({
        id: `path-${appQuery}`,
        family: 'open path',
        canonical: `open ${appQuery}`,
        detail: 'open path in Finder',
      });
    } else {
      const matchingApps = filterAppNames(appNames, appQuery);
      for (const app of matchingApps) {
        commands.push({
          id: `open-${app.toLowerCase()}`,
          family: 'open app',
          canonical: `open ${app.toLowerCase()}`,
          detail: `launch ${app}`,
        });
      }
    }
  }

  const matchingServices = filterServices(serviceCatalog, serviceQuery);
  for (const service of matchingServices) {
    commands.push({
      id: `service-${service.id}`,
      family: 'open service',
      canonical: `open ${service.display_name.toLowerCase()}`,
      detail: `open ${service.display_name} in browser`,
    });
  }

  if (startsWithAny(normalized, ['create folder', 'make folder', 'new folder'])) {
    const folderName = extractFolderName(inputValue);
    const canonicalName = folderName || (titleCaseQuery || 'New Folder');
    commands.push({
      id: `create-folder-${canonicalName.toLowerCase()}`,
      family: 'create folder',
      canonical: `create folder called ${canonicalName} in home`,
      detail: 'create folder in home directory',
    });
  }

  if (startsWithAny(normalized, ['move ', 'put '])) {
    const moveSuggestion = extractMoveSuggestion(inputValue);
    if (moveSuggestion) {
      commands.push(moveSuggestion);
    }
  }

  if (normalized.includes('download')) {
    commands.push({
      id: 'downloads',
      family: 'open path',
      canonical: 'downloads',
      detail: 'open Downloads in Finder',
    });
  }

  if (normalized.includes('display') || normalized.includes('screen') || normalized.includes('monitor')) {
    commands.push({
      id: 'display-settings',
      family: 'settings',
      canonical: 'display settings',
      detail: 'open System Settings → Displays',
    });
  }

  if (normalized.includes('mute')) {
    commands.push({
      id: 'mute',
      family: 'sound',
      canonical: 'mute',
      detail: 'mute system audio',
    });
  }

  const volumeSuggestion = extractVolumeSuggestion(normalized);
  if (volumeSuggestion) {
    commands.push(volumeSuggestion);
  }

  return dedupeSuggestions(commands).slice(0, 4);
}

function getInstalledAppNames(machineInfo: MachineInfo | null): string[] {
  const fromApps = machineInfo?.installed_apps?.map((app) => app.name) ?? [];
  const fromBrowsers = machineInfo?.installed_browsers?.map((app) => app.name) ?? [];
  const merged = [...fromApps, ...fromBrowsers];
  return Array.from(new Set(merged)).sort((a, b) => a.localeCompare(b));
}

function filterAppNames(appNames: string[], query: string): string[] {
  const normalizedQuery = normalizePhrase(query);
  if (!normalizedQuery) return appNames.slice(0, 4);
  return appNames
    .filter((app) => normalizePhrase(app).includes(normalizedQuery))
    .slice(0, 4);
}

function filterServices(serviceCatalog: ServiceDefinition[], query: string): ServiceDefinition[] {
  const normalizedQuery = normalizePhrase(query);
  if (!normalizedQuery) {
    return [];
  }

  return serviceCatalog
    .filter((service) => {
      const display = normalizePhrase(service.display_name);
      if (display.includes(normalizedQuery) || normalizedQuery.includes(display)) {
        return true;
      }
      return service.aliases.some((alias) => {
        const normalizedAlias = normalizePhrase(alias);
        return normalizedAlias.includes(normalizedQuery) || normalizedQuery.includes(normalizedAlias);
      });
    })
    .slice(0, 4);
}

function getServiceQuery(value: string): string {
  const normalized = normalizePhrase(value);
  for (const prefix of ['open ', 'watch ', 'browse ', 'visit ', 'go to ']) {
    if (normalized.startsWith(prefix)) {
      return normalized.slice(prefix.length).split(/\s+in\s+/i)[0]?.trim() ?? '';
    }
  }
  return normalized.split(/\s+in\s+/i)[0]?.trim() ?? '';
}

function extractFolderName(raw: string): string {
  const trimmed = raw.trim();
  const lower = normalizePhrase(trimmed);
  for (const marker of [' called ', ' named ']) {
    const index = lower.indexOf(marker);
    if (index >= 0) {
      const after = trimmed.slice(index + marker.length).trim();
      const baseSplit = after.split(/\s+(?:in|inside|under)\s+/i)[0]?.trim();
      return stripQuotes(baseSplit || '');
    }
  }
  return '';
}

function extractMoveSuggestion(raw: string): CommandSuggestion | null {
  const lower = normalizePhrase(raw);
  const marker = lower.includes(' into ') ? ' into ' : lower.includes(' to ') ? ' to ' : '';
  if (!marker) return null;
  const parts = raw.trim().split(new RegExp(marker, 'i'));
  if (parts.length < 2) return null;
  const source = stripQuotes(parts[0].replace(/^(move|put)\s+/i, '').trim());
  const destination = stripQuotes(parts[1].trim());
  if (!source || !destination) return null;
  return {
    id: `move-${source}-${destination}`,
    family: 'move path',
    canonical: `move ${source} to ${destination}`,
    detail: 'move path inside home directory',
  };
}

function extractVolumeSuggestion(normalized: string): CommandSuggestion | null {
  const match = normalized.match(/(?:set\s+volume\s+to|volume\s+to|set\s+volume|volume\s+at)\s+(\d{1,3})/);
  if (!match) return null;
  const level = Math.max(0, Math.min(100, Number(match[1])));
  return {
    id: `volume-${level}`,
    family: 'sound',
    canonical: `set volume to ${level}`,
    detail: 'set output volume',
  };
}

function dedupeSuggestions(items: CommandSuggestion[]): CommandSuggestion[] {
  const seen = new Set<string>();
  return items.filter((item) => {
    const key = `${item.family}:${item.canonical.toLowerCase()}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function startsWithAny(value: string, prefixes: string[]): boolean {
  return prefixes.some((prefix) => value.startsWith(prefix));
}

function getRemainderAfterPrefix(value: string, prefixes: string[]): string {
  for (const prefix of prefixes) {
    if (value.startsWith(prefix)) {
      return value.slice(prefix.length).trim();
    }
  }
  return value;
}

function normalizePhrase(value: string): string {
  return value.trim().toLowerCase().replace(/\s+/g, ' ');
}

function titleCase(value: string): string {
  return value
    .split(' ')
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

function stripQuotes(value: string): string {
  return value.trim().replace(/^['"“”]+|['"“”]+$/g, '');
}

function looksLikePath(value: string): boolean {
  const trimmed = value.trim();
  return trimmed.startsWith('~/')
    || trimmed.startsWith('/')
    || trimmed.includes('/')
    || ['desktop', 'downloads', 'documents', 'home'].includes(trimmed);
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
    case 'base_path_unresolved':
      return cmd.unresolved_message?.trim() || 'I could not resolve where to create that folder.';
    case 'target_already_exists':
      return cmd.unresolved_message?.trim() || 'That target already exists.';
    case 'destination_path_unresolved':
      return cmd.unresolved_message?.trim() || 'I could not resolve the destination path.';
    case 'destination_parent_missing':
      return cmd.unresolved_message?.trim() || 'The destination parent folder does not exist.';
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
