/// Structured planner for commands that cannot be resolved locally.
/// Phase 1 stub — remote planner is not available in Phase 1.
/// Returns an explicit error so the UI can surface a clear message.
pub fn plan(_input: &str) -> Result<(), crate::errors::AppError> {
    Err(crate::errors::AppError::NotFound(
        "Remote planner is not available in Phase 1. Use a supported local command.".to_string(),
    ))
}
