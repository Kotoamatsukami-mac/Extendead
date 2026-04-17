# Extendead repository instructions

You are building Extendead, a production-grade macOS operator shell.

## Product truth

Extendead is:
- a local-first operator shell
- a single-strip command surface
- a deterministic interpreter with controlled fallback
- a native-feeling utility
- not a chatbot
- not a transcript UI
- not a plugin platform
- not a prompt playground

The workspace root is:
`~/Desktop/anbu/Extended`

## Core stack

- Tauri 2
- Rust core
- React 18
- TypeScript
- Vite
- macOS first-class
- Intel first-class
- Apple Silicon supported
- Windows later, but do not degrade macOS architecture to pre-optimize Windows

## Architecture rules

Business logic lives in Rust.
The frontend is a membrane only.
Do not move validation, execution, permission logic, or shell policy into React.

Use this pipeline for every command:
1. capture
2. normalize
3. classify
4. resolve locally if possible
5. if unresolved, generate structured plan
6. validate
7. risk-score
8. request approval if required
9. execute
10. stream logs
11. persist run record
12. persist inverse action if valid
13. expose undo

## Non-negotiables

- No chatbot transcript UI
- No message bubbles
- No assistant persona in the product
- No arbitrary shell execution in v1
- No freeform model output in execution flow
- No hidden retries beyond declared retry policy
- No silent fallback
- No private API usage for the macOS visual layer
- No over-abstracted frontend state sprawl
- No untyped executor results

## Performance rules

Optimize for:
- instant-feeling cold launch
- immediate strip readiness
- very fast local intent classification
- asynchronous interpreter warmup
- responsive input while logs stream

Do not block app launch on remote model/network initialization.

## Security rules

- Frontend never stores provider API keys in plain text
- Frontend never executes raw commands
- Rust owns interpreter calls, validation, execution, permissions, and persistence
- Use macOS keychain-backed storage for provider credentials on macOS-first builds
- Treat every shell or AppleScript step as untrusted until validated

## UI rules

The app has two modes only:
- lounge strip
- expanded console

Lounge mode:
- slim
- premium
- minimal
- visually quiet
- dark glass
- high contrast
- evil-rainbow accent only as accent
- non-boxy
- can toggle always-on-top
- designed to live in front while working

Expanded mode:
- parsed intent
- browser/app choices if ambiguous
- risk badge
- plan preview
- Y / N confirmation
- live execution timeline
- result card
- undo/history drawer
- permission status

## Confirmation model

Y/N confirmation is core UI.
If the action is ambiguous or medium/high risk:
- show route
- show target
- show risk
- show reversible status
- require explicit confirmation

## Execution rules

Prefer:
1. deterministic internal Rust action
2. approved command template
3. AppleScript against scriptable app
4. System Events UI scripting
5. remote structured planner

Never reverse this order.

## Coding rules

- Strong types first
- Small files with clear responsibility
- No placeholder architecture
- No fake mocks left in production path
- Tests for parser, validator, risk, inverse generation
- Comments only when they add real value
- Prefer boring reliability over cleverness
- Remove and replace a subsystem only when root cause proves architecture mismatch

## Delivery rules

Do not jump ahead.
Build in phases.
Each phase must compile, run, and satisfy its acceptance criteria before moving on.
