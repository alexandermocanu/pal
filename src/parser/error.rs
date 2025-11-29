use thiserror::Error;

/// An error type that describes any possible parsing error.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseError {
    #[error("reached invalid state (this error should never be returned, please report)")]
    Unit,
    #[error("invalid character; expected one of {expected:?}, found {found:?}")]
    CharacterMismatch {
        expected: Option<char>,
        found: Option<char>,
    },
}
