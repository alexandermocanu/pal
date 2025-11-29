use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    #[error("invalid character; expected one of {expected:?}, found {found:?}")]
    CharacterMismatch {
        expected: Vec<char>,
        found: Option<char>,
    },
}
