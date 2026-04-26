/**
 * API Handler - Unified API key management and provider abstraction
 * Seamlessly integrated into the semantic pipeline
 */

export interface ProviderConfig {
  name: string;
  baseUrl: string;
  apiKey: string;
  model?: string;
  enabled: boolean;
}

export interface APIResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  provider: string;
  timestamp: number;
}

export class APIHandler {
  private providers: Map<string, ProviderConfig> = new Map();
  private primaryProvider: string | null = null;

  constructor() {
    this.initializeProviders();
  }

  private initializeProviders(): void {
    // Providers will be loaded from config/env
    // This allows for dynamic provider configuration
  }

  setProvider(name: string, config: ProviderConfig): void {
    this.providers.set(name, config);
    if (!this.primaryProvider) {
      this.primaryProvider = name;
    }
  }

  setPrimaryProvider(name: string): boolean {
    if (this.providers.has(name)) {
      this.primaryProvider = name;
      return true;
    }
    return false;
  }

  async call<T = unknown>(
    provider: string | null,
    method: string,
    params: Record<string, unknown>,
  ): Promise<APIResponse<T>> {
    const target = provider || this.primaryProvider;

    if (!target) {
      return {
        success: false,
        error: "No provider configured",
        provider: "none",
        timestamp: Date.now(),
      };
    }

    const config = this.providers.get(target);
    if (!config || !config.enabled) {
      return {
        success: false,
        error: `Provider ${target} not available`,
        provider: target,
        timestamp: Date.now(),
      };
    }

    try {
      const response = await fetch(`${config.baseUrl}/${method}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${config.apiKey}`,
        },
        body: JSON.stringify(params),
      });

      if (!response.ok) {
        return {
          success: false,
          error: `API error: ${response.statusText}`,
          provider: target,
          timestamp: Date.now(),
        };
      }

      const data = (await response.json()) as T;
      return {
        success: true,
        data,
        provider: target,
        timestamp: Date.now(),
      };
    } catch (err) {
      return {
        success: false,
        error: String(err),
        provider: target,
        timestamp: Date.now(),
      };
    }
  }

  getProvider(name: string): ProviderConfig | undefined {
    return this.providers.get(name);
  }

  listProviders(): ProviderConfig[] {
    return Array.from(this.providers.values());
  }

  hasApiKey(provider: string): boolean {
    const config = this.providers.get(provider);
    return !!(config?.apiKey && config.enabled);
  }
}

export const apiHandler = new APIHandler();
