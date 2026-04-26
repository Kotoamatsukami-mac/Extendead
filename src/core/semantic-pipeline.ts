/**
 * Semantic Pipeline Core
 *
 * Input text → Semantic parse → Intent extraction → Constraint hierarchy →
 * Context retrieval → Risk classification → Reasoning-effort selection →
 * Plan generation → Action/answer → Verification against original constraints
 */

import type {
  CommandKind,
  InterpretationDecision,
  RiskLevel,
} from "../types/commands";

export interface SemanticContext {
  originalInput: string;
  normalizedInput: string;
  history: string[];
  machineState?: {
    installedApps: string[];
    installedBrowsers: string[];
    osVersion: string;
  };
  apiKey?: string;
  provider?: string;
}

export interface SemanticParseResult {
  rawInput: string;
  normalized: string;
  tokens: string[];
  confidence: number;
}

export interface Intent {
  category: CommandKind;
  action: string;
  target?: string;
  params: Record<string, string | number | boolean>;
  confidence: number;
}

export interface ConstraintSet {
  required: string[];
  forbidden: string[];
  preferred: string[];
  conditional: Record<string, string[]>;
}

export interface RiskAssessment {
  level: RiskLevel;
  factors: string[];
  requiresApproval: boolean;
  suggestedVerification?: string[];
}

export interface ReasoningEffort {
  level: "low" | "medium" | "high" | "xhigh";
  justification: string;
  estimatedComplexity: number;
}

export interface ExecutionPlan {
  id: string;
  steps: ExecutionStep[];
  totalRisk: RiskLevel;
  estimatedDuration: number;
  canUndo: boolean;
}

export interface ExecutionStep {
  type: string;
  description: string;
  action: string;
  riskLevel: RiskLevel;
  verifiable: boolean;
}

export class SemanticPipeline {
  constructor(private context: SemanticContext) {}

  async process(input: string): Promise<PipelineResult> {
    try {
      // Stage 1: Semantic parsing
      const parsed = this.semanticParse(input);

      // Stage 2: Intent extraction
      const intent = await this.extractIntent(parsed);

      // Stage 3: Constraint hierarchy
      const constraints = this.buildConstraintHierarchy(intent);

      // Stage 4: Context retrieval & binding
      const boundContext = this.retrieveContext(intent, constraints);

      // Stage 5: Risk classification
      const risk = this.classifyRisk(intent, constraints);

      // Stage 6: Reasoning effort selection
      const effort = this.selectReasoningEffort(risk, constraints);

      // Stage 7: Plan generation
      const plan = await this.generatePlan(intent, constraints, risk, effort);

      // Stage 8: Action/Answer preparation
      const decision = this.makeDecision(plan, risk, constraints);

      // Stage 9: Verification
      const verified = this.verifyAgainstConstraints(
        plan,
        constraints,
        this.context,
      );

      return {
        success: verified.isValid,
        intent,
        plan,
        decision,
        verification: verified,
        metadata: {
          parsed,
          risk,
          effort,
          constraints,
          context: boundContext,
        },
      };
    } catch (err) {
      return {
        success: false,
        error: String(err),
      };
    }
  }

  private semanticParse(input: string): SemanticParseResult {
    const normalized = input.toLowerCase().trim();
    const tokens = normalized.split(/\s+/).filter(Boolean);

    return {
      rawInput: input,
      normalized,
      tokens,
      confidence: Math.min(1, tokens.length / 10),
    };
  }

  private async extractIntent(parsed: SemanticParseResult): Promise<Intent> {
    const [verb, ...rest] = parsed.tokens;

    const categoryMap: Record<string, CommandKind> = {
      open: "app_control",
      close: "app_control",
      quit: "app_control",
      hide: "app_control",
      set: "settings",
      show: "ui_automation",
      find: "query",
      search: "query",
      move: "filesystem",
      copy: "filesystem",
      delete: "filesystem",
    };

    const kind = categoryMap[verb] || "unknown";

    return {
      category: kind,
      action: verb,
      target: rest.join(" "),
      params: this.extractParameters(parsed.tokens),
      confidence: Math.min(1, parsed.tokens.length / 5),
    };
  }

  private extractParameters(
    tokens: string[],
  ): Record<string, string | number | boolean> {
    const params: Record<string, string | number | boolean> = {};

    for (let i = 0; i < tokens.length; i++) {
      if (tokens[i] === "to" && i + 1 < tokens.length) {
        params.target = tokens[i + 1];
      }
      if (tokens[i] === "in" && i + 1 < tokens.length) {
        params.location = tokens[i + 1];
      }
    }

    return params;
  }

  private buildConstraintHierarchy(intent: Intent): ConstraintSet {
    const constraints: ConstraintSet = {
      required: [],
      forbidden: [],
      preferred: [],
      conditional: {},
    };

    // App control constraints
    if (intent.category === "app_control") {
      constraints.required.push("app_installed");
      constraints.required.push("bundle_id_valid");
    }

    // File operation constraints
    if (intent.category === "filesystem") {
      constraints.required.push("source_exists");
      constraints.preferred.push("no_overwrite");
      constraints.conditional.move = ["destination_parent_exists"];
    }

    // Settings constraints
    if (intent.category === "settings") {
      constraints.required.push("permissions_granted");
      constraints.preferred.push("safe_defaults");
    }

    return constraints;
  }

  private retrieveContext(
    intent: Intent,
    constraints: ConstraintSet,
  ): Record<string, unknown> {
    const context: Record<string, unknown> = {
      timestamp: Date.now(),
      intent,
      constraints,
    };

    // Add machine state if constraints require it
    if (constraints.required.includes("app_installed")) {
      context.installedApps = this.context.machineState?.installedApps || [];
    }

    return context;
  }

  private classifyRisk(
    intent: Intent,
    _constraints: ConstraintSet,
  ): RiskAssessment {
    let level: RiskLevel = "R0";
    const factors: string[] = [];

    if (intent.category === "filesystem" && intent.action === "delete") {
      level = "R3";
      factors.push("destructive_operation");
      factors.push("data_loss_risk");
    } else if (intent.category === "settings") {
      level = "R2";
      factors.push("system_modification");
    } else if (intent.category === "app_control") {
      level = "R1";
      factors.push("process_control");
    }

    return {
      level,
      factors,
      requiresApproval: level !== "R0",
      suggestedVerification:
        factors.length > 0 ? ["confirm_action", "show_affected_items"] : [],
    };
  }

  private selectReasoningEffort(
    risk: RiskAssessment,
    constraints: ConstraintSet,
  ): ReasoningEffort {
    const constraintCount =
      Object.keys(constraints.conditional).length + constraints.required.length;
    const complexity =
      constraintCount + (risk.level.charCodeAt(1) - "0".charCodeAt(0));

    let level: "low" | "medium" | "high" | "xhigh" = "low";
    if (complexity > 5) level = "high";
    else if (complexity > 3) level = "medium";

    if (risk.level === "R3") level = "xhigh";
    else if (risk.level === "R2") level = "high";

    return {
      level,
      justification: `Risk: ${risk.level}, Constraints: ${constraintCount}`,
      estimatedComplexity: complexity,
    };
  }

  private async generatePlan(
    intent: Intent,
    constraints: ConstraintSet,
    risk: RiskAssessment,
    _effort: ReasoningEffort,
  ): Promise<ExecutionPlan> {
    const steps: ExecutionStep[] = [];

    // Validation steps
    for (const req of constraints.required) {
      steps.push({
        type: "validation",
        description: `Verify ${req}`,
        action: `validate_${req}`,
        riskLevel: "R0",
        verifiable: true,
      });
    }

    // Execution step
    steps.push({
      type: "execution",
      description: `Execute ${intent.action} on ${intent.target || "target"}`,
      action: intent.action,
      riskLevel: risk.level,
      verifiable: true,
    });

    // Post-execution verification
    steps.push({
      type: "verification",
      description: "Verify action completed successfully",
      action: "verify_completion",
      riskLevel: "R0",
      verifiable: true,
    });

    return {
      id: `plan_${Date.now()}`,
      steps,
      totalRisk: risk.level,
      estimatedDuration: steps.length * 100,
      canUndo:
        intent.category === "filesystem" || intent.category === "settings",
    };
  }

  private makeDecision(
    plan: ExecutionPlan,
    risk: RiskAssessment,
    constraints: ConstraintSet,
  ): InterpretationDecision {
    // If any required constraint is missing, clarify
    if (constraints.required.length > 0) {
      return "clarify";
    }

    // If high risk, ask for approval via clarification
    if (risk.requiresApproval) {
      return "clarify";
    }

    // If no issues, execute
    if (plan.steps.length > 0) {
      return "execute";
    }

    return "deny";
  }

  private verifyAgainstConstraints(
    plan: ExecutionPlan,
    originalConstraints: ConstraintSet,
    _context: SemanticContext,
  ): VerificationResult {
    const violations: string[] = [];
    const warnings: string[] = [];

    // Verify required constraints are addressed
    for (const req of originalConstraints.required) {
      const hasValidation = plan.steps.some((s) => s.action.includes(req));
      if (!hasValidation) {
        violations.push(`Missing validation for: ${req}`);
      }
    }

    // Check for forbidden patterns
    for (const forbidden of originalConstraints.forbidden) {
      const hasForbidden = plan.steps.some((s) => s.action.includes(forbidden));
      if (hasForbidden) {
        violations.push(`Forbidden constraint violated: ${forbidden}`);
      }
    }

    // Verify step sequence
    if (plan.steps.length < 3) {
      warnings.push(
        "Plan may be incomplete - missing validation or verification steps",
      );
    }

    return {
      isValid: violations.length === 0,
      violations,
      warnings,
      passedChecks: originalConstraints.required.length - violations.length,
      totalChecks: originalConstraints.required.length,
    };
  }
}

export interface PipelineResult {
  success: boolean;
  intent?: Intent;
  plan?: ExecutionPlan;
  decision?: InterpretationDecision;
  verification?: VerificationResult;
  metadata?: Record<string, unknown>;
  error?: string;
}

export interface VerificationResult {
  isValid: boolean;
  violations: string[];
  warnings: string[];
  passedChecks: number;
  totalChecks: number;
}
