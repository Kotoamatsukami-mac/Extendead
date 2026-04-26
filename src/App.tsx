import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { DeveloperPanel } from "./components/DeveloperPanel";
import { ExpandedConsole } from "./components/ExpandedConsole";
import { LoungeStrip } from "./components/LoungeStrip";
import { useCommandBridge } from "./hooks/useCommandBridge";
import { usePermissionStatus } from "./hooks/usePermissionStatus";
import type {
  CommandSuggestion,
  ExecutionResult,
  HistoryEntry,
  ParsedCommand,
  ProviderKeyStatus,
  ResultFeedback,
} from "./types/commands";
import type { ExecutionEvent } from "./types/events";

type AppMode = "lounge" | "expanded";
type ExecState =
  | "idle"
  | "parsing"
  | "awaiting_clarify"
  | "awaiting_choice"
  | "awaiting_route"
  | "awaiting_confirm"
  | "executing"
  | "done"
  | "error";

const DEV_PANEL_UNLOCK = "//engine";

const BUILT_IN_PREDICTIONS = [
  "open youtube",
  "open safari",
  "close safari",
  "open chrome",
  "open finder",
  "open slack",
  "study mode",
  "focus mode",
  "break mode",
  "quieter",
  "display settings",
  "downloads",
  "set volume to 40",
  DEV_PANEL_UNLOCK,
] as const;

export function App() {
  const [mode, setMode] = useState<AppMode>("lounge");
  const [inputValue, setInputValue] = useState("");
  const [parsedCommand, setParsedCommand] = useState<ParsedCommand | null>(
    null,
  );
  const [selectedRouteIndex, setSelectedRouteIndex] = useState<number | null>(
    null,
  );
  const [execState, setExecState] = useState<ExecState>("idle");
  const [events, setEvents] = useState<ExecutionEvent[]>([]);
  const [result, setResult] = useState<ExecutionResult | null>(null);
  const [alwaysOnTop, setAlwaysOnTop] = useState(true);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [suggestions, setSuggestions] = useState<CommandSuggestion[]>([]);
  const [showDeveloperPanel, setShowDeveloperPanel] = useState(false);
  const [primaryProviderStatus, setPrimaryProviderStatus] =
    useState<ProviderKeyStatus | null>(null);
  const [developerBusy, setDeveloperBusy] = useState(false);
  const [resultFeedback, setResultFeedback] = useState<ResultFeedback | null>(
    null,
  );
  const [windowFeedback, setWindowFeedback] = useState<ResultFeedback | null>(
    null,
  );
  const [pinBusy, setPinBusy] = useState(false);
  const [focusTrigger, setFocusTrigger] = useState(0);
  const [autoExec, setAutoExec] = useState<{
    cmd: ParsedCommand;
    routeIdx: number;
  } | null>(null);

  const oneShotRef = useRef(false);
  const feedbackTimerRef = useRef<number>(0);
  const windowFeedbackTimerRef = useRef<number>(0);
  const suggestionRequestRef = useRef(0);

  const { permissionStatus, refresh: refreshPermissionStatus } =
    usePermissionStatus();

  const parsedCommandRef = useRef<ParsedCommand | null>(null);
  parsedCommandRef.current = parsedCommand;

  function showInlineFeedback(
    message: string,
    type: "success" | "error",
    duration: number,
  ) {
    window.clearTimeout(feedbackTimerRef.current);
    setResultFeedback({ message, type });
    feedbackTimerRef.current = window.setTimeout(() => {
      setResultFeedback(null);
      setExecState("idle");
      setParsedCommand(null);
      setResult(null);
      setFocusTrigger((n) => n + 1);
    }, duration);
  }

  function showWindowFeedback(
    message: string,
    type: "success" | "error",
    duration: number,
  ) {
    window.clearTimeout(windowFeedbackTimerRef.current);
    setWindowFeedback({ message, type });
    windowFeedbackTimerRef.current = window.setTimeout(() => {
      setWindowFeedback(null);
    }, duration);
  }

  const bridge = useCommandBridge({
    onParseStart: () => {
      setShowDeveloperPanel(false);
      setExecState("parsing");
      setEvents([]);
      setResult(null);
      setAutoExec(null);
      setResultFeedback(null);
      window.clearTimeout(feedbackTimerRef.current);
      oneShotRef.current = false;
    },
    onParsed: (cmd) => {
      setParsedCommand(cmd);

      if (cmd.interpretation_decision === "clarify") {
        setMode("lounge");
        setSelectedRouteIndex(null);
        setExecState("awaiting_clarify");
        return;
      }

      if (
        cmd.interpretation_decision === "offer_choices" &&
        (cmd.choices?.length ?? 0) > 0
      ) {
        setMode("lounge");
        setSelectedRouteIndex(null);
        setExecState("awaiting_choice");
        return;
      }

      if (cmd.routes.length === 0) {
        setExecState("error");
        showInlineFeedback(getUnresolvedMessage(cmd), "error", 2600);
        return;
      }

      if (cmd.routes.length === 1) {
        setSelectedRouteIndex(0);
        if (cmd.requires_approval || isPlanRoute(cmd.routes[0])) {
          setMode("expanded");
          setExecState("awaiting_confirm");
        } else {
          oneShotRef.current = true;
          setAutoExec({ cmd, routeIdx: 0 });
        }
      } else {
        setMode("expanded");
        setSelectedRouteIndex(null);
        setExecState("awaiting_route");
      }
    },
    onParseError: (err) => {
      setExecState("error");
      showInlineFeedback(err, "error", 3500);
    },
    onExecutionEvent: (event) => {
      setEvents((prev) => [...prev, event]);
    },
    onExecuted: (res) => {
      setResult(res);
      const isSuccess = res.outcome === "success";
      setExecState(isSuccess ? "done" : "error");

      if (oneShotRef.current) {
        oneShotRef.current = false;
        const msg = isSuccess
          ? res.human_message || "✓ Done"
          : res.human_message || "✗ Failed";
        showInlineFeedback(
          msg,
          isSuccess ? "success" : "error",
          isSuccess ? 2000 : 3500,
        );
      }

      bridge.getHistory().then(setHistory);
    },
    onExecuteError: (err) => {
      setExecState("error");

      if (oneShotRef.current) {
        oneShotRef.current = false;
        showInlineFeedback(err, "error", 3500);
      } else {
        setResult({
          command_id: parsedCommandRef.current?.id ?? "",
          outcome: "recoverable_failure",
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
    setExecState("executing");
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
          setSuggestions(rankSuggestions(nextSuggestions, history));
        }
      });
    }, 90);

    return () => window.clearTimeout(timer);
  }, [bridge, history, inputValue, resultFeedback]);

  const refreshDeveloperStatus = useCallback(async () => {
    setDeveloperBusy(true);
    try {
      const status = await bridge.getProviderKeyStatus("perplexity");
      setPrimaryProviderStatus(status);
    } finally {
      setDeveloperBusy(false);
    }
  }, [bridge]);

  const handleOpenEngineLink = useCallback(() => {
    setShowDeveloperPanel(true);
    setMode("expanded");
    setParsedCommand(null);
    setSelectedRouteIndex(null);
    setExecState("idle");
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
        setInputValue("");
        return;
      }

      setInputValue("");
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

  const handleSelectChoice = useCallback(
    (value: string) => {
      const trimmed = value.trim();
      if (!trimmed) return;
      setInputValue("");
      bridge.parseCommand(trimmed);
    },
    [bridge],
  );

  const handleSelectRoute = useCallback(
    (index: number) => {
      setSelectedRouteIndex(index);
      if (!parsedCommand) return;
      const selectedRoute = parsedCommand.routes[index];
      if (parsedCommand.requires_approval || isPlanRoute(selectedRoute)) {
        setExecState("awaiting_confirm");
      } else {
        setExecState("executing");
        bridge.approveAndExecute(parsedCommand.id, index);
      }
    },
    [parsedCommand, bridge],
  );

  const handleConfirm = useCallback(() => {
    if (!parsedCommand || selectedRouteIndex === null) return;
    setExecState("executing");
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
    setExecState("executing");
    bridge.undoLast();
  }, [bridge]);

  const handleCollapse = useCallback(() => {
    if (parsedCommand && execState === "awaiting_confirm") {
      bridge.denyCommand(parsedCommand.id);
    }
    reset();
  }, [parsedCommand, execState, bridge]);

  const handleToggleAlwaysOnTop = useCallback(() => {
    if (pinBusy) return;
    const next = !alwaysOnTop;
    setPinBusy(true);
    void bridge
      .toggleAlwaysOnTop(next)
      .then((config) => {
        setAlwaysOnTop(config.always_on_top);
      })
      .catch((err) => {
        showWindowFeedback(`Pin toggle failed: ${String(err)}`, "error", 2600);
      })
      .finally(() => {
        setPinBusy(false);
      });
  }, [alwaysOnTop, bridge, pinBusy]);

  const handleLinkPrimaryEngine = useCallback(
    async (value: string) => {
      setDeveloperBusy(true);
      try {
        await bridge.setProviderKey("perplexity", value);
        const status = await bridge.getProviderKeyStatus("perplexity");
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
      await bridge.deleteProviderKey("perplexity");
      const status = await bridge.getProviderKeyStatus("perplexity");
      setPrimaryProviderStatus(status);
    } finally {
      setDeveloperBusy(false);
    }
  }, [bridge]);

  const handleInspectLocal = useCallback(
    async (value: string) => {
      return bridge.debugInterpretLocal(value);
    },
    [bridge],
  );

  const handleInputChange = useCallback(
    (value: string) => {
      setInputValue(value);
      if (resultFeedback) {
        window.clearTimeout(feedbackTimerRef.current);
        setResultFeedback(null);
        setExecState("idle");
        setParsedCommand(null);
        setResult(null);
      }
    },
    [resultFeedback],
  );

  function reset() {
    window.clearTimeout(feedbackTimerRef.current);
    setMode("lounge");
    setInputValue("");
    setParsedCommand(null);
    setSelectedRouteIndex(null);
    setExecState("idle");
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
      if (e.key === "Escape" && mode === "expanded") {
        e.preventDefault();
        handleCollapse();
      } else if (execState === "awaiting_confirm") {
        if (e.key === "Enter") {
          e.preventDefault();
          handleConfirm();
        } else if (e.key === "Escape") {
          e.preventDefault();
          handleCancel();
        }
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [mode, execState, handleCollapse, handleConfirm, handleCancel]);

  useEffect(() => {
    const refreshConfig = () => {
      void bridge.getAppConfig().then((config) => {
        if (config) {
          setAlwaysOnTop(config.always_on_top);
        }
      });
    };
    const refreshFocusedMachineTruth = () => {
      refreshConfig();
      void bridge.refreshMachineInfo();
    };

    refreshConfig();
    const interval = window.setInterval(() => {
      if (document.visibilityState === "visible") {
        refreshConfig();
      }
    }, 15_000);

    window.addEventListener("focus", refreshFocusedMachineTruth);
    document.addEventListener("visibilitychange", refreshConfig);
    return () => {
      window.clearInterval(interval);
      window.removeEventListener("focus", refreshFocusedMachineTruth);
      document.removeEventListener("visibilitychange", refreshConfig);
    };
  }, [bridge]);

  return (
    <div className={`app app--${mode}`}>
      <div className="app__surface">
        <LoungeStrip
          inputValue={inputValue}
          prediction={prediction}
          suggestions={suggestions}
          clarificationMessage={
            execState === "awaiting_clarify"
              ? (parsedCommand?.clarification_message ??
                parsedCommand?.unresolved_message ??
                null)
              : null
          }
          clarificationSlots={
            execState === "awaiting_clarify"
              ? (parsedCommand?.clarification_slots ?? [])
              : []
          }
          choices={
            execState === "awaiting_choice"
              ? (parsedCommand?.choices ?? [])
              : []
          }
          execState={execState}
          alwaysOnTop={alwaysOnTop}
          pinBusy={pinBusy}
          focusTrigger={focusTrigger}
          resultFeedback={resultFeedback}
          windowFeedback={windowFeedback}
          embedded={mode === "expanded"}
          onInput={handleInputChange}
          onSubmit={handleSubmit}
          onAcceptPrediction={handleAcceptPrediction}
          onApplySuggestion={handleApplySuggestion}
          onSelectChoice={handleSelectChoice}
          onEscape={handleCollapse}
          onToggleAlwaysOnTop={handleToggleAlwaysOnTop}
          onOpenEngineLink={
            mode === "lounge" ? handleOpenEngineLink : undefined
          }
        />
        {mode === "expanded" &&
          (showDeveloperPanel ? (
            <DeveloperPanel
              status={primaryProviderStatus}
              busy={developerBusy}
              alwaysOnTop={alwaysOnTop}
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
              alwaysOnTop={alwaysOnTop}
              onSelectRoute={handleSelectRoute}
              onConfirm={handleConfirm}
              onCancel={handleCancel}
              onUndo={handleUndo}
              onCollapse={handleCollapse}
            />
          ))}
      </div>
    </div>
  );
}

function getPrediction(inputValue: string, history: HistoryEntry[]): string {
  const normalized = inputValue.trim().toLowerCase();
  if (!normalized) return "";

  const candidates = [
    ...history.map((entry) => entry.command.raw_input),
    ...BUILT_IN_PREDICTIONS,
  ];
  const seen = new Set<string>();

  for (const candidate of candidates) {
    const lowered = candidate.toLowerCase();
    if (seen.has(lowered)) continue;
    seen.add(lowered);

    if (lowered.startsWith(normalized) && lowered !== normalized) {
      return candidate;
    }
  }

  return "";
}

function rankSuggestions(
  suggestions: CommandSuggestion[],
  history: HistoryEntry[],
): CommandSuggestion[] {
  if (suggestions.length <= 1 || history.length === 0) return suggestions;
  const nowHour = new Date().getHours();

  return [...suggestions].sort((a, b) => {
    return (
      scoreSuggestion(b, history, nowHour) -
      scoreSuggestion(a, history, nowHour)
    );
  });
}

function scoreSuggestion(
  suggestion: CommandSuggestion,
  history: HistoryEntry[],
  nowHour: number,
): number {
  const canonical = suggestion.canonical.toLowerCase();
  return history.reduce((score, entry, index) => {
    const raw = entry.command.raw_input.toLowerCase();
    const recency = Math.max(0, history.length - index) / history.length;
    const hour = new Date(entry.timestamp).getHours();
    const sameHour = Math.abs(hour - nowHour) <= 1 ? 0.35 : 0;
    if (raw === canonical) return score + 2 + recency + sameHour;
    if (raw.startsWith(canonical) || canonical.startsWith(raw)) {
      return score + 0.8 + recency * 0.5 + sameHour;
    }
    return score;
  }, 0);
}

function getUnresolvedMessage(cmd: ParsedCommand): string {
  if (cmd.unresolved_message?.trim()) {
    return cmd.unresolved_message.trim();
  }

  switch (cmd.unresolved_code) {
    case "unsupported_command":
      return "Not in local command set. Try: open [app], set volume [0-100], focus mode, study mode, display settings.";
    case "unsupported_service":
      return "That service is not configured. Check available integrations in settings.";
    case "browser_not_installed":
      return "That browser is not installed. Supported: Safari, Chrome, Firefox, Edge.";
    case "app_not_installed":
      return "App not found on this Mac. Try: open finder, open settings, open terminal.";
    case "path_not_found":
      return "Path does not exist. Check the path and try again.";
    case "source_path_not_found":
      return "Source path not found. Verify the path exists before copying or moving.";
    case "base_path_unresolved":
      return "Could not resolve folder location. Try specifying a full path.";
    case "target_already_exists":
      return "That file or folder already exists. Choose a different name or remove it first.";
    case "destination_path_unresolved":
      return "Destination path could not be resolved. Try specifying a full path.";
    case "destination_parent_missing":
      return "Parent folder does not exist. Create the parent folder first.";
    case "permanent_delete_blocked":
      return "Permanent delete is blocked for safety. Use: trash [path] to move to trash instead.";
    case "ambiguous_target":
      return "Multiple matches found. Be more specific with the app or file name.";
    case "provider_configuration_required":
      return "Provider not configured. Link an API key in settings for advanced features.";
  }

  switch (cmd.kind) {
    case "unknown":
      return "Unknown command. Try: open [app], set volume, focus mode, study mode.";
    case "app_control":
      return "App action not supported on this Mac. App may not be installed.";
    case "settings":
      return "That settings pane is not available yet.";
    case "ui_automation":
      return "That automation route is not available yet.";
    case "filesystem":
      return "File operation failed. Check paths and permissions.";
    case "shell_execution":
      return "Shell command not in local coverage. Use app control or system settings instead.";
    case "query":
      return "Query not supported. Try app control or system settings.";
    default:
      return "Could not resolve a safe local route for that command.";
  }
}

function isPlanRoute(
  route: ParsedCommand["routes"][number] | undefined,
): boolean {
  return route?.action.type === "run_plan";
}
