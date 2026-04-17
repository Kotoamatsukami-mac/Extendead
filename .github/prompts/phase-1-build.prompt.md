Read the repository instructions, path-specific instructions, agent contract, and docs first.

Project root: ~/Desktop/anbu/Extended

You are building Extendead.

Do not drift from the repository instructions.

Phase target: Phase 1 only.

## Phase 1 goal

Create a production-grade macOS-first Tauri 2 + Rust + React/TypeScript application with:

- one lounge strip window
- one expanded console mode
- global shortcut to summon/hide
- toggle always-on-top
- command input
- typed Rust <-> frontend command bridge
- machine signature scan
- local resolver for first golden tasks
- Y / N confirmation rail
- execution event timeline scaffold
- history record types scaffold
- secure provider-key service interface scaffold
- no transcript UI
- no arbitrary shell execution

## Build requirements

Implement these first golden tasks end-to-end:
1. open youtube
2. open slack
3. mute the mac
4. set volume to 30 percent
5. open display settings
6. reveal downloads

For "open youtube":
- inspect installed browser candidates locally
- prefer Chrome and Safari if present
- show route choices in UI
- allow confirmation through Y / N rail
- do not hardcode a single browser

## Architecture rules

Rust owns:
- machine scan
- parser
- resolver
- validator scaffolding
- risk scaffolding
- execution scaffolding
- events
- settings
- provider-key interface

Frontend owns:
- rendering
- input
- route selection
- confirmation rail
- live event display

## Visual rules

Lounge strip must be:
- slim
- dark glass
- non-boxy
- high contrast
- premium
- minimal
- with a subtle menacing rainbow accent only on focus/loading/approval

Do not build a chat layout.

## Output format

1. state exact files you will create or modify
2. implement phase 1 completely
3. include any capability/config files needed
4. keep code production-grade
5. do not leave placeholder TODO architecture for core phase-1 paths
6. if you must defer anything, state exactly why and keep the architecture ready for phase 2

Begin now.
