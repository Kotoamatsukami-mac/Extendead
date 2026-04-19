# Extendead: Start Here

This repository is for **Extendead**, a production-grade, local-first macOS operator shell.

## Product identity

- **Stack:** Tauri 2 + Rust + React/TypeScript
- **Platform priority:** macOS first
- **Primary surface:** compact floating operator strip
- **Secondary surface:** expanded panel only when functionally necessary
- **Product shape:** operator shell, **not** a chatbot

Extendead accepts normal language requests, interprets them into structured local actions, and executes them through a strict native pipeline.

## What Extendead is not

Do not drift this repo into:

- a chat app
- a dashboard
- a terminal clone
- a cloud puppet
- a model-led sandbox where the AI can bypass execution rules

## Locked execution spine

```text
parse -> resolve -> validate -> risk -> approve -> execute -> persist -> undo
```

Do not break this chain.
Do not reorder it.
Do not let any AI or provider layer bypass it.

Interpretation can be flexible.
Execution authority must remain local.

## Current design truths to preserve

The compact shell is already in the keep-it zone.

Do **not** casually redesign:

- compact strip direction
- strip length
- font feel
- ghost / prediction text
- darker glass direction
- engine/settings panel family

If repo truth proves a regression, fix the regression.
Do not reopen visual identity because of boredom.

## Current problems worth fixing

This repo is in **behavioral tightening** mode.
Priority issues:

1. drag reliability
2. expanded lower-panel massing
3. unsupported-command feedback quality
4. command coverage truth
5. meaningful pin/unpin behavior
6. permission surfacing

## Engineering doctrine

Prefer:

- small durable diffs
- runtime truth over theory
- verified capability over confident bluffing
- local deterministic coverage for common commands
- typed failure states instead of vague fallback messages

Avoid:

- fake broad support
- hidden magical command paths as core UX
- glow / bloom / style churn pretending to be progress
- backend rewrites when the actual problem is shell behavior

## Interpretation doctrine

Extendead should use a hybrid model:

- **deterministic local path first** for common commands
- **model-assisted interpretation second** for ambiguity, argument extraction, and broader language

The provider key is **not** the architecture.
A provider only supplies interpretation capability.
The product remains the validated local execution shell.

## Required review format

Future check-ins should report in this order:

1. what is already good
2. what is still broken
3. why it is broken
4. what should change
5. what should not be touched
6. runtime confidence: verified vs inferred

## Immediate next move

Before broad code changes, lock the repo contract in text and align new work to it.
That is the purpose of the docs in this folder.
