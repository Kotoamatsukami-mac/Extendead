use std::fmt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "detail")]
pub enum AppError {
    PlatformNotSupported(String),
    ExecutionError(String),
    ValidationError(String),
    NotFound(String),
    IoError(String),
    SerializationError(String),
    StateLockError,
    ShellPolicyViolation(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::PlatformNotSupported(s) => write!(f, "Platform not supported: {s}"),
            AppError::ExecutionError(s) => write!(f, "Execution error: {s}"),
            AppError::ValidationError(s) => write!(f, "Validation error: {s}"),
            AppError::NotFound(s) => write!(f, "Not found: {s}"),
            AppError::IoError(s) => write!(f, "IO error: {s}"),
            AppError::SerializationError(s) => write!(f, "Serialization error: {s}"),
            AppError::StateLockError => write!(f, "State lock poisoned"),
            AppError::ShellPolicyViolation(s) => write!(f, "Shell policy violation: {s}"),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::IoError(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::SerializationError(e.to_string())
    }
}
