# Failure semantics

Every executable step returns exactly one outcome:

- success
- recoverable_failure
- blocked
- timed_out
- partial_success

## Required fields

- code
- category
- human_message
- machine_message
- safe_next_action
- can_retry
- retry_count
- inverse_status

## Category examples

- permission_missing
- validation_rejected
- target_not_found
- selector_miss
- command_template_miss
- planner_schema_invalid
- executor_timeout
- partial_rollback_only
- unsupported_in_v1

## Rules

- no silent fallback
- no hidden retries beyond policy
- no treating blocked as failure
- no swallowing partial success
