pub mod syntax;
pub mod value;
pub mod error;

pub use self::value::Value;
pub use self::error::{RunError, RunErrorValue};

pub type RunResult<T> = Result<T, RunError>;