# Semantic Pipeline Architecture

## Overview

The semantic pipeline is the **core processing engine** that replaces scattered command processing logic throughout the app. It implements a 10-stage constraint-based reasoning system:

```
Input text
    ↓
Semantic parse (tokenization, normalization)
    ↓
Intent extraction (category, action, target, params)
    ↓
Constraint hierarchy (required, forbidden, preferred, conditional)
    ↓
Context retrieval (machine state, API keys, history binding)
    ↓
Risk classification (R0-R3, approval requirements)
    ↓
Reasoning-effort selection (low/medium/high/xhigh)
    ↓
Plan generation (validation steps, execution, verification)
    ↓
Decision making (execute, clarify, offer_choices, deny)
    ↓
Verification against original constraints
    ↓
Result with structured output
```

## Core Modules

### `src/core/semantic-pipeline.ts`
The main `SemanticPipeline` class:
- 10 private methods implementing each stage
- Constraint-based reasoning
- Risk assessment
- Plan generation with verifiable steps
- Full verification loop

**Key exports:**
- `SemanticPipeline` - Main class
- `SemanticContext` - Input context with history/machine state
- `Intent` - Extracted intent with category and params
- `ConstraintSet` - Required/forbidden/preferred/conditional rules
- `RiskAssessment` - Risk level + approval requirements
- `ExecutionPlan` - Generated plan with verification-ready steps
- `PipelineResult` - Structured output for UI consumption

### `src/core/api-handler.ts`
Unified API key + provider management:
- `APIHandler` class with provider registry
- Seamless API integration into pipeline stages
- Support for multiple providers (perplexity, claude, etc)
- Proper error handling and typing

**Key exports:**
- `APIHandler` - Provider management
- `apiHandler` - Singleton instance
- `APIResponse<T>` - Typed API responses
- `ProviderConfig` - Provider configuration

### `src/hooks/useSemanticalPipeline.ts`
React bridge for the semantic pipeline:
- `useSemanticalPipeline()` - Hook that initializes pipeline
- `processCommand()` - Process user input through full pipeline
- `setPrimaryProvider()` - Set active API provider
- `callAPI()` - Make API calls with provider routing
- `hasApiKey()` - Check provider readiness

## Integration Points

### Current State (Pre-refactor)
App.tsx has scattered logic:
- Command parsing in `useCommandBridge`
- Decision making in multiple `handleSubmit` / `handleExecute` branches
- Risk assessment buried in condition checks
- History management as separate useState

### Target State (Post-refactor)
App.tsx becomes a clean view layer:
```tsx
const { processCommand, callAPI } = useSemanticalPipeline({ history, machineState });

const handleSubmit = async (input: string) => {
  const result = await processCommand(input);
  
  // result.intent - what user wants
  // result.plan - how to do it
  // result.decision - what to ask/do next
  // result.verification - safety checks passed
  
  // UI responds to decision enum, not to scattered conditions
  if (result.decision === 'clarify') {
    setExecState('awaiting_clarify');
  } else if (result.decision === 'execute') {
    executeWithVerifiedPlan(result.plan);
  }
};
```

## Benefits of This Architecture

1. **Separation of Concerns**: Logic layer fully separated from UI
2. **Testability**: Each stage can be tested independently
3. **Extensibility**: New stages or providers plug in cleanly
4. **Constraint-Based**: Safety encoded in constraints, not scattered checks
5. **Verifiable**: Every plan includes verification steps
6. **Context-Aware**: Long-context binding + history integrated
7. **Risk-Explicit**: Risk assessment is first-class, not implicit

## Next Implementation Steps

### Phase 2: App.tsx Refactor
1. Replace `useCommandBridge` with `useSemanticalPipeline`
2. Consolidate 15+ useState hooks into pipeline result handling
3. Remove scattered decision logic
4. Update handlers to respond to `decision` enum

### Phase 3: Clean Up
1. Remove duplicate parsing logic from Rust bridge
2. Eliminate overlapping risk assessment code
3. Merge constraint checking logic
4. Consolidate history management

### Phase 4: API Integration
1. Blend API keys into context seamlessly
2. Add provider selection to pipeline
3. Route complex queries through selected provider
4. Cache provider responses

## Example: How "open safari" Flows

```
Input: "open safari"
  ↓ Parse: ["open", "safari"]
  ↓ Intent: category=app_control, action=open, target=safari
  ↓ Constraints: required=[app_installed, bundle_id_valid]
  ↓ Context: installedApps=[Safari, Chrome, Firefox], ...
  ↓ Risk: R0 (safe, no approval needed)
  ↓ Effort: low (simple operation)
  ↓ Plan: [validate_app_installed, execute_open, verify_completion]
  ↓ Decision: execute
  ↓ Verify: all constraints satisfied
  ↓ Result: success=true, decision=execute, plan ready
```

The 10 stages ensure Safari is verified to exist, then opened, then verified running.
