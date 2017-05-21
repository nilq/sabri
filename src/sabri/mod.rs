pub mod syntax;
pub mod value;
pub mod error;
pub mod env;

pub use self::value::Value;
pub use self::env::Env;
pub use self::error::{RunError, RunErrorValue};

pub type RunResult<T> = Result<T, RunError>;