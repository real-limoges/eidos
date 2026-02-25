use std::fmt;

/// Single opaque error type for eidos Phase 1.
/// No variant categorization by design — callers match on EidosError, not variants.
#[derive(Debug)]
pub enum EidosError {
    InvalidConfig(String),
    RenderFailed(String),
}

impl fmt::Display for EidosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EidosError::InvalidConfig(msg) => write!(f, "invalid configuration: {}", msg),
            EidosError::RenderFailed(msg) => write!(f, "render failed: {}", msg),
        }
    }
}

impl std::error::Error for EidosError {}
