use thiserror::Error;

/// Represents any non-LLVM codegen error.
#[derive(Error, Debug)]
pub enum CodegenError {
    #[error("tried to reference a type that does not exist.")]
    TypeDoesNotExist,
    #[error("no such function was found")]
    FunctionDoesNotExist,
}
