# Extendead Interpreter Routing

This document defines how Extendead should choose between deterministic local routing and model-assisted interpretation.

The goal is simple:

- keep common commands fast and local
- use model help where it genuinely improves understanding
- never let interpretation replace execution truth

## Core rule

Extendead has **one execution system** and may have **multiple interpretation sources**.

Interpretation sources may include:

- deterministic parser
- provider-backed model
- future retrieval/search/tool layer

All roads must normalize back into the same validated command schema.

## Routing order

### 1. Deterministic local path first

Use the local path first for common, high-frequency, high-confidence commands such as:

- open app
- quit app
- reveal common folder
- open common settings pane
- open common browser target
- mute / set volume
- simple OS toggles

Why:

- faster
- cheaper
- offline-capable
- predictable
- easier to test

### 2. Typo-tolerant local recovery second

Before escalating to a provider, do lightweight local recovery:

- lowercase and whitespace normalization
- alias mapping
- common typo tolerance
- fuzzy matching against installed apps and known folders
- verb synonym mapping

Examples:

- `opne safrai` -> `open safari`
- `downlaods` -> `downloads`
- `slcak` -> `slack`

If confidence is strong and the target is real on this Mac, local recovery should win.

### 3. Model-assisted interpretation third

Use a provider only when local routing is insufficient.

Good use cases:

- broader human phrasing
- multi-step intent drafting
- ambiguous target extraction
- recovery from weak spelling when local confidence is too low
- mapping intent into a candidate command plan

Examples:

- `close everything distracting me`
- `put all desktop stuff into a new folder called chat`
- `open the browser I use for work`

## Provider abstraction doctrine

Do not lock Extendead to one provider.

Use a provider adapter interface such as:

```ts
interface InterpreterProvider {
  name: string;
  interpret(input: string, context: InterpreterContext): Promise<InterpreterProposal>;
}
```

The provider is replaceable.
The schema is not.

## Provider usage rules

A provider proposal should be treated as a **candidate**, not as execution truth.

The local system must still verify:

- installed app truth
- file/path truth
- permission truth
- risk level
- approval requirement
- action validity

## Confidence routing

Suggested routing behavior:

- **high local confidence** -> execute local path
- **medium local confidence** -> offer route or clarification
- **low local confidence** -> provider proposal if configured
- **no provider + low confidence** -> fail honestly with typed reason

## Provider unavailable behavior

If no provider key is configured:

- deterministic coverage must still work
- unresolved commands should fail honestly
- the UI may explain that broader interpretation requires a linked provider

Bad behavior:

- pretending the shell is smarter than it is
- silently doing nothing
- vague `not recognised` failure with no next step

## Search / retrieval / connectors

These are optional support layers, not execution authority.

Potential future uses:

- web grounding for current information queries
- document retrieval for user files
- tool adapters via MCP or equivalent

These layers may improve understanding.
They still must not bypass local validation and execution rules.

## Provenance goal

Future interpreted commands should carry provenance metadata:

- `deterministic`
- `local_fuzzy`
- `provider_interpreted`

This helps debugging, testing, and trust.

## Immediate implementation direction

Short-term priorities:

1. strengthen deterministic coverage for common macOS actions
2. add local typo and alias recovery
3. support one provider-backed interpretation path behind a replaceable adapter
4. return typed unresolved reasons when neither path can safely resolve the request

## Non-goal

Do not turn Extendead into a remote-first agent.
The point is a believable local operator shell with optional intelligence, not a cloud puppet with glass effects.
