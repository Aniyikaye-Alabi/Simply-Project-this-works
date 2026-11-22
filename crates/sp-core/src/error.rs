use std::fmt;

/// A failure explained in plain language, with one suggested next step.
///
/// Every IPC command returns `Result<T, UserError>` (PRD section 8). Messages must
/// avoid technical jargon: no "codec", "bitrate", "WebRTC" and so on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserError {
    /// Stable machine-readable code, for example `"file_missing"`.
    pub code: &'static str,
    /// What happened, in plain words.
    pub message: String,
    /// What the user can do about it.
    pub suggested_fix: String,
}

impl UserError {
    /// Creates an error from a code, a plain message and a suggested fix.
    pub fn new(
        code: &'static str,
        message: impl Into<String>,
        suggested_fix: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            suggested_fix: suggested_fix.into(),
        }
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.message, self.suggested_fix)
    }
}

impl std::error::Error for UserError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_error_shows_message_then_fix() {
        let e = UserError::new("file_missing", "We can't find that file.", "Use Find file.");
        assert_eq!(e.to_string(), "We can't find that file. Use Find file.");
        assert_eq!(e.code, "file_missing");
    }
}
