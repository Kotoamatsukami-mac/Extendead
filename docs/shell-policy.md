# Extendead shell policy v1

## Principle

V1 does not permit arbitrary shell execution.

## Allowed execution forms

- internal Rust actions
- approved command templates
- approved AppleScript templates
- approved UI automation templates

## Shell template rules

Allowed binaries in v1:
- open
- osascript
- defaults (read-only only unless explicitly approved per template)
- say (debug only, disabled by default)
- ls
- pwd
- whoami
- git (read-focused templates first)
- npm / pnpm / yarn / bun (repo-local scripted templates only, later phase)
- cargo (repo-local scripted templates only, later phase)

Disallowed in v1:
- sudo
- rm permanent deletion templates
- chmod/chown
- launchctl write actions
- network scanners
- curl/wget execution pipelines
- arbitrary sh/bash/zsh -c
- python -c
- osascript with model-generated raw body bypassing validator

Forbidden syntax:
- &&
- ||
- ;
- >
- >>
- <
- $()
- backticks
- glob expansion in destructive contexts

## Deletion policy

Prefer move-to-trash semantics over permanent deletion.
Permanent deletion is out of scope for v1 unless explicitly added later with separate approval policy.
