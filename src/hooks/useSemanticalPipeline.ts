/**
 * useSemanticalPipeline - React hook that bridges the semantic pipeline to UI
 * Centralizes command processing with the 10-stage pipeline
 */

import { useCallback, useRef } from "react";
import {
  SemanticPipeline,
  type SemanticContext,
  type PipelineResult,
} from "../core/semantic-pipeline";
import { apiHandler } from "../core/api-handler";
import type { HistoryEntry } from "../types/commands";

interface UsePipelineOptions {
  history: HistoryEntry[];
  apiKey?: string;
  provider?: string;
  machineState?: SemanticContext["machineState"];
}

export function useSemanticalPipeline(options: UsePipelineOptions) {
  const pipelineRef = useRef<SemanticPipeline | null>(null);

  const initializePipeline = useCallback(() => {
    const context: SemanticContext = {
      originalInput: "",
      normalizedInput: "",
      history: options.history.map((h) => h.command.raw_input),
      machineState: options.machineState,
      apiKey: options.apiKey,
      provider: options.provider,
    };

    pipelineRef.current = new SemanticPipeline(context);
  }, [options.history, options.apiKey, options.provider, options.machineState]);

  const processCommand = useCallback(
    async (input: string): Promise<PipelineResult> => {
      if (!pipelineRef.current) {
        initializePipeline();
      }

      if (!pipelineRef.current) {
        return {
          success: false,
          error: "Pipeline initialization failed",
        };
      }

      return pipelineRef.current.process(input);
    },
    [initializePipeline],
  );

  const setPrimaryProvider = useCallback((provider: string): boolean => {
    return apiHandler.setPrimaryProvider(provider);
  }, []);

  const callAPI = useCallback(
    async <T = unknown>(
      method: string,
      params: Record<string, unknown>,
      provider?: string,
    ) => {
      return apiHandler.call<T>(provider || null, method, params);
    },
    [],
  );

  const hasApiKey = useCallback((provider: string): boolean => {
    return apiHandler.hasApiKey(provider);
  }, []);

  return {
    processCommand,
    setPrimaryProvider,
    callAPI,
    hasApiKey,
    apiHandler,
  };
}
