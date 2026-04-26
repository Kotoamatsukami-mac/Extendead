import { useCallback, useEffect, useRef, useState } from 'react';
import { LoungeStrip } from './components/LoungeStrip';
import { useCommandBridge } from './hooks/useCommandBridge';
import type { ExecutionResult, InterpretationPreview, ParsedCommand } from './types/commands';

type ExecState =
  | 'idle'
  | 'parsing'
  | 'awaiting_clarify'
  | 'awaiting_choice'
  | 'awaiting_route'
  | 'awaiting_confirm'
  | 'awaiting_key'
  | 'executing'
  | 'done'
  | 'error';

type StatusTone = 'neutral' | 'success' | 'error';

type StatusLine = {
  message: string;
  tone: StatusTone;
};

const DEFAULT_PROVIDER = 'perplexity';
const API_KEY_REQUIRED_MESSAGE = 'API key required for broader interpretation.';

export function App() {
  const [inputValue, setInputValue] = useState('');
  const [preview, setPreview] = useState<InterpretationPreview | null>(null);
  const [parsedCommand, setParsedCommand] = useState<ParsedCommand | null>(null);
  const [selectedRouteIndex, setSelectedRouteIndex] = useState<number | null>(null);
  const [execState, setExecState] = useState<ExecState>('idle');
  const [result, setResult] = useState<ExecutionResult | null>(null);
  const [alwaysOnTop, setAlwaysOnTop] = useState(true);
  const [pinBusy, setPinBusy] = useState(false);
  const [focusTrigger, setFocusTrigger] = useState(0);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [pendingProviderInput, setPendingProviderInput] = useState<string | null>(null);
  const [apiKeyValue, setApiKeyValue] = useState('');
  const [apiKeyBusy, setApiKeyBusy] = useState(false);

  const statusTimerRef = useRef<number>(0);
  const previewRequestRef = useRef(0);
  const parsedCommandRef = useRef<ParsedCommand | null>(null);
  parsedCommandRef.current = parsedCommand;

  function clearStatusTimer() {
    window.clearTimeout(statusTimerRef.current);
  }

  function resetToIdle(clearInput = true) {
    clearStatusTimer();
    if (clearInput) {
      setInputValue('');
      setPreview(null);
    }
    setParsedCommand(null);
    setSelectedRouteIndex(null);
    setExecState('idle');
    setResult(null);
    setErrorMessage(null);
    setSuccessMessage(null);
    setPendingProviderInput(null);
    setApiKeyValue('');
    setFocusTrigger((n) => n + 1);
  }

  function settleWithMessage(message: string, tone: StatusTone, duration: number) {
    clearStatusTimer();
    setPreview(null);
    if (tone === 'success') {
      setSuccessMessage(message);
      setErrorMessage(null);
      setExecState('done');
    } else if (tone === 'error') {
      setErrorMessage(message);
      setSuccessMessage(null);
      setExecState('error');
    } else {
      setSuccessMessage(null);
      setErrorMessage(null);
    }
    statusTimerRef.current = window.setTimeout(() => {
      resetToIdle(false);
    }, duration);
  }

  const bridge = useCommandBridge({
    onParseStart: () => {
      clearStatusTimer();
      setPreview(null);
      setExecState('parsing');
      setParsedCommand(null);
      setSelectedRouteIndex(null);
      setResult(null);
      setErrorMessage(null);
      setSuccessMessage(null);
      setPendingProviderInput(null);
    },
    onParsed: (cmd) => {
      setParsedCommand(cmd);

      if (cmd.interpretation_decision === 'clarify') {
        setExecState('awaiting_clarify');
        return;
      }

      if (cmd.interpretation_decision === 'offer_choices' && (cmd.choices?.length ?? 0) > 0) {
        setExecState('awaiting_choice');
        return;
      }

      if (cmd.routes.length === 0) {
        if (cmd.unresolved_code === 'provider_configuration_required') {
          setPendingProviderInput(cmd.raw_input);
          setExecState('awaiting_key');
          return;
        }
        bridge.denyCommand(cmd.id);
        settleWithMessage(getUnresolvedMessage(cmd), 'error', 3500);
        return;
      }

      if (cmd.routes.length === 1) {
        setSelectedRouteIndex(0);
        if (cmd.requires_approval || isPlanRoute(cmd.routes[0])) {
          setExecState('awaiting_confirm');
        } else {
          setExecState('executing');
          bridge.approveAndExecute(cmd.id, 0);
        }
        return;
      }

      setSelectedRouteIndex(null);
      setExecState('awaiting_route');
    },
    onParseError: (err) => {
      settleWithMessage(toGuidanceMessage(err), 'error', 3500);
    },
    onExecuted: (res) => {
      setResult(res);
      const isSuccess = res.outcome === 'success';
      if (isSuccess) {
        settleWithMessage(res.human_message || 'Done', 'success', 2000);
      } else {
        settleWithMessage(toGuidanceMessage(res.human_message || res.message), 'error', 3500);
      }
    },
    onExecuteError: (err) => {
      const guidance = toGuidanceMessage(err);
      setResult({
        command_id: parsedCommandRef.current?.id ?? '',
        outcome: 'recoverable_failure',
        message: err,
        human_message: guidance,
        duration_ms: 0,
      });
      settleWithMessage(guidance, 'error', 3500);
    },
  });

  useEffect(() => {
    bridge.getAppConfig().then((config) => {
      if (config) {
        setAlwaysOnTop(config.always_on_top);
      }
    });
    void bridge.setWindowMode('lounge');
  }, [bridge]);

  useEffect(() => {
    const value = inputValue.trim();
    if (!value || execState !== 'idle') {
      setPreview(null);
      return;
    }

    const requestId = previewRequestRef.current + 1;
    previewRequestRef.current = requestId;

    const timer = window.setTimeout(() => {
      bridge.interpretPreview(value).then((nextPreview) => {
        if (previewRequestRef.current === requestId) {
          setPreview(nextPreview);
        }
      });
    }, 90);

    return () => window.clearTimeout(timer);
  }, [bridge, execState, inputValue]);

  const handleSubmit = useCallback(
    (value: string) => {
      const trimmed = value.trim();
      if (!trimmed) return;
      setInputValue('');
      setPreview(null);
      bridge.parseCommand(trimmed);
    },
    [bridge],
  );

  const handleSelectChoice = useCallback(
    (value: string) => {
      const trimmed = value.trim();
      if (!trimmed) return;
      setInputValue('');
      setPreview(null);
      bridge.parseCommand(trimmed);
    },
    [bridge],
  );

  const handleSelectRoute = useCallback(
    (index: number) => {
      if (!parsedCommand) return;
      setSelectedRouteIndex(index);
      const selectedRoute = parsedCommand.routes[index];
      if (parsedCommand.requires_approval || isPlanRoute(selectedRoute)) {
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
    resetToIdle();
  }, [parsedCommand, bridge]);

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
        settleWithMessage(toGuidanceMessage(`Pin toggle failed: ${String(err)}`), 'error', 2600);
      })
      .finally(() => {
        setPinBusy(false);
      });
  }, [alwaysOnTop, bridge, pinBusy]);

  const handleInputChange = useCallback((value: string) => {
    setInputValue(value);
    if (execState === 'error' || execState === 'done') {
      clearStatusTimer();
      setErrorMessage(null);
      setSuccessMessage(null);
      setExecState('idle');
      setParsedCommand(null);
      setSelectedRouteIndex(null);
      setResult(null);
    }
  }, [execState]);

  const handleApiKeySubmit = useCallback(async () => {
    if (!pendingProviderInput || !apiKeyValue.trim() || apiKeyBusy) return;
    setApiKeyBusy(true);
    try {
      await bridge.setProviderKey(DEFAULT_PROVIDER, apiKeyValue.trim());
      setApiKeyValue('');
      bridge.parseCommand(pendingProviderInput);
    } catch (err) {
      settleWithMessage(toGuidanceMessage(`API key save failed: ${String(err)}`), 'error', 3500);
    } finally {
      setApiKeyBusy(false);
    }
  }, [apiKeyBusy, apiKeyValue, bridge, pendingProviderInput]);

  const handleApiKeyCancel = useCallback(() => {
    if (parsedCommand) {
      bridge.denyCommand(parsedCommand.id);
    }
    setPendingProviderInput(null);
    settleWithMessage(API_KEY_REQUIRED_MESSAGE, 'error', 3500);
  }, [parsedCommand, bridge]);

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (execState === 'awaiting_confirm') {
        if (e.key === 'Enter' || e.key.toLowerCase() === 'y') {
          e.preventDefault();
          handleConfirm();
        } else if (e.key.toLowerCase() === 'n' || e.key === 'Escape') {
          e.preventDefault();
          handleCancel();
        }
      }
    }
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [execState, handleConfirm, handleCancel]);

  const statusLine = buildStatusLine(execState, parsedCommand, result, errorMessage, successMessage);

  return (
    <div className="app">
      <div className="app__surface">
        <LoungeStrip
          inputValue={inputValue}
          preview={preview}
          execState={execState}
          alwaysOnTop={alwaysOnTop}
          pinBusy={pinBusy}
          focusTrigger={focusTrigger}
          statusLine={statusLine}
          clarificationMessage={execState === 'awaiting_clarify' ? parsedCommand?.clarification_message ?? parsedCommand?.unresolved_message ?? null : null}
          clarificationSlots={execState === 'awaiting_clarify' ? (parsedCommand?.clarification_slots ?? []) : []}
          choices={execState === 'awaiting_choice' ? (parsedCommand?.choices ?? []) : []}
          routes={execState === 'awaiting_route' ? (parsedCommand?.routes ?? []) : []}
          confirmLabel={execState === 'awaiting_confirm' ? buildConfirmLabel(parsedCommand, selectedRouteIndex) : null}
          confirmDescription={execState === 'awaiting_confirm' ? buildConfirmDescription(parsedCommand, selectedRouteIndex) : null}
          showApiKeyPrompt={execState === 'awaiting_key'}
          apiKeyPromptMessage={API_KEY_REQUIRED_MESSAGE}
          apiKeyValue={apiKeyValue}
          apiKeyBusy={apiKeyBusy}
          onInput={handleInputChange}
          onSubmit={handleSubmit}
          onSelectChoice={handleSelectChoice}
          onSelectRoute={handleSelectRoute}
          onConfirm={handleConfirm}
          onCancel={handleCancel}
          onToggleAlwaysOnTop={handleToggleAlwaysOnTop}
          onApiKeyChange={setApiKeyValue}
          onApiKeySubmit={handleApiKeySubmit}
          onApiKeyCancel={handleApiKeyCancel}
          onEscape={handleCancel}
        />
      </div>
    </div>
  );
}

function buildStatusLine(
  execState: ExecState,
  command: ParsedCommand | null,
  result: ExecutionResult | null,
  errorMessage: string | null,
  successMessage: string | null,
): StatusLine | null {
  if (execState === 'idle') return null;

  if (execState === 'parsing') {
    return { message: 'Parsing', tone: 'neutral' };
  }

  if (execState === 'awaiting_key') {
    return { message: API_KEY_REQUIRED_MESSAGE, tone: 'neutral' };
  }

  if (execState === 'awaiting_clarify') {
    return {
      message: command?.clarification_message
        || command?.unresolved_message
        || 'Need more detail before continuing.',
      tone: 'neutral',
    };
  }

  if (execState === 'awaiting_choice') {
    return {
      message: command?.clarification_message || 'Choose an action to continue.',
      tone: 'neutral',
    };
  }

  if (execState === 'awaiting_route') {
    return { message: 'Choose a route to continue.', tone: 'neutral' };
  }

  if (execState === 'awaiting_confirm') {
    const riskLevel = command?.risk ? ` ${command.risk}` : '';
    return { message: `Approval needed${riskLevel}.`, tone: 'neutral' };
  }

  if (execState === 'executing') {
    return { message: 'Executing', tone: 'neutral' };
  }

  if (execState === 'done') {
    return { message: successMessage || result?.human_message || 'Done', tone: 'success' };
  }

  if (execState === 'error') {
    return { message: errorMessage || result?.human_message || 'Needs attention', tone: 'error' };
  }

  return null;
}

function buildConfirmLabel(command: ParsedCommand | null, selectedRouteIndex: number | null): string {
  if (!command || selectedRouteIndex === null) return 'Approve action';
  return command.routes[selectedRouteIndex]?.label || 'Approve action';
}

function buildConfirmDescription(command: ParsedCommand | null, selectedRouteIndex: number | null): string | null {
  if (!command || selectedRouteIndex === null) return null;
  return command.routes[selectedRouteIndex]?.description || null;
}

function getUnresolvedMessage(cmd: ParsedCommand): string {
  switch (cmd.unresolved_code) {
    case 'unsupported_command':
      return 'Unsupported yet. Try open app create folder move file trash file volume or mode.';
    case 'unsupported_service':
      return 'Unsupported yet. That service is outside current local coverage.';
    case 'browser_not_installed':
      return cmd.unresolved_message?.trim() || 'That browser is not installed on this Mac.';
    case 'app_not_installed':
      return cmd.unresolved_message?.trim() || 'That app is not installed on this Mac.';
    case 'path_not_found':
      return cmd.unresolved_message?.trim() || 'That path does not exist.';
    case 'source_path_not_found':
      return cmd.unresolved_message?.trim() || 'The source path does not exist.';
    case 'base_path_unresolved':
      return cmd.unresolved_message?.trim() || 'Need a place to create that folder.';
    case 'target_already_exists':
      return cmd.unresolved_message?.trim() || 'That target already exists.';
    case 'destination_path_unresolved':
      return cmd.unresolved_message?.trim() || 'Need a valid destination path.';
    case 'destination_parent_missing':
      return cmd.unresolved_message?.trim() || 'The destination parent folder does not exist.';
    case 'permanent_delete_blocked':
      return cmd.unresolved_message?.trim() || 'Blocked. Permanent delete is disabled. Use trash instead.';
    case 'ambiguous_target':
      return cmd.unresolved_message?.trim() || 'Choose one. Multiple targets match that name.';
    case 'provider_configuration_required':
      return cmd.unresolved_message?.trim() || API_KEY_REQUIRED_MESSAGE;
    default:
      break;
  }

  if (cmd.unresolved_message?.trim()) {
    return cmd.unresolved_message.trim();
  }

  return 'Unsupported yet. Try open app create folder move file trash file volume or mode.';
}

function toGuidanceMessage(value: string): string {
  const text = value.replace(/^✗\s*/, '').trim();
  if (!text) return 'Needs attention.';
  if (/permission|not authorized|accessibility|apple events/i.test(text)) {
    return text;
  }
  if (/blocked|denied|permanent delete/i.test(text)) {
    return text;
  }
  if (/not found|does not exist|missing/i.test(text)) {
    return text;
  }
  return text.replace(/\berror\b/gi, 'needs attention');
}

function isPlanRoute(route: ParsedCommand['routes'][number] | undefined): boolean {
  return route?.action.type === 'run_plan';
}
