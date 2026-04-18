/// Structured planner for commands that cannot be resolved locally.
/// Phase 2 stub — remote planner is not available until Phase 3.
/// Returns a typed error so the UI can surface a clear message.
pub fn plan(_input: &str) -> Result<(), crate::errors::AppError> {
    Err(crate::errors::AppError::NotFound(
        "Remote planner is not available yet. Use a supported local command.".to_string(),
    ))
}
