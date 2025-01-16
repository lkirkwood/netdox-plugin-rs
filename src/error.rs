use thiserror::Error;

#[derive(Error, Debug)]
pub enum FCallError {
    #[error("function call to {function} had the wrong arguments: {problem}")]
    WrongArgs {
        function: &'static str,
        problem: &'static str,
    },
    #[error("function call failed on redis: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("a logical error was encountered: {0}")]
    Logic(&'static str),
}

pub type FCallResult<T> = Result<T, FCallError>;
