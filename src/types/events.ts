// Mirror of Rust events.rs — execution event streaming types.

export type ExecutionEventKind =
  | 'started'
  | 'progress'
  | 'completed'
  | 'failed'
  | 'cancelled';

export interface ExecutionEvent {
  id: string;
  command_id: string;
  timestamp: string;
  kind: ExecutionEventKind;
  message: string;
}

export interface ExecutionEventPayload {
  event: ExecutionEvent;
}
