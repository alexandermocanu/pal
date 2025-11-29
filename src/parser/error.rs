use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseError {
    #[error("reached unit error, state is invalid")]
    Unit,
    #[error("invalid character; expected one of {expected:?}, found {found:?}")]
    CharacterMismatch {
        expected: Option<char>,
        found: Option<char>,
    },
}
