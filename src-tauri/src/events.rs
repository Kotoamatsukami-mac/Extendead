use serde::{Deserialize, Serialize};

use crate::models::ExecutionEvent;

/// Tauri event name emitted during execution streaming.
pub const EXECUTION_EVENT_NAME: &str = "execution-event";

/// Wrapper payload sent to the frontend via `app_handle.emit`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEventPayload {
    pub event: ExecutionEvent,
}
