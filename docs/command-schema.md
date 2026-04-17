# Command schema

## Command lifecycle

```
input → capture → normalize → classify → resolve → validate → risk → approve → execute → stream → persist → undo
```

## Command struct

Every command flowing through the system must carry:

- `id`: unique identifier
- `raw_input`: original user text
- `normalized`: cleaned/lowercased form
- `classification`: enum category
- `resolved_action`: the concrete action to take
- `validation_status`: pass / fail / pending
- `risk_level`: R0 / R1 / R2 / R3
- `approval_status`: approved / denied / pending / not_required
- `execution_result`: success / recoverable_failure / blocked / timed_out / partial_success
- `inverse_action`: optional reversible action
- `timestamp`: when the command was issued
- `duration_ms`: execution duration

## Classification categories

- app_control
- mixed_workflow
- local_system
- filesystem
- ui_automation
- shell_execution
- settings
- query (informational, no side effects)

## Risk levels

- R0: no side effects, safe to auto-execute if policy allows
- R1: minor side effects, reversible, low risk
- R2: meaningful side effects, requires confirmation
- R3: destructive or irreversible, requires explicit confirmation and warning
