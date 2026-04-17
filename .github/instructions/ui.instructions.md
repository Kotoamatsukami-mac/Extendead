---
applyTo: "src/**/*.ts,src/**/*.tsx,src/**/*.css"
---

The frontend is a membrane, not the brain.

## UI mission

The app should feel like:
- one elegant strip
- one clear command sentence
- one immediate path to confirmation
- one place to see execution

## Prohibited UI patterns

- chat bubbles
- transcript timelines as primary layout
- oversized cards everywhere
- loud gradients
- toy-like AI aesthetics
- multiple competing panes
- boxy command bar

## Required UI behaviors

Lounge strip:
- visible, thin, attractive
- dark glass feel
- high contrast text
- subtle menacing rainbow accent on focus/loading/approval
- strong input focus state
- can toggle always-on-top
- non-boxy silhouette

Expanded console:
- parsed intent chip
- route selector if ambiguity exists
- Y / N confirmation
- risk badge
- live event timeline
- result summary
- undo drawer
- permission state

## Data flow

React does not invent execution logic.
React renders Rust state and sends typed user intents back to Rust.

## Interaction rules

The first action after typing must always feel obvious:
- Enter parses
- if ambiguity exists, show choices
- Y confirms
- N cancels
- Escape collapses
