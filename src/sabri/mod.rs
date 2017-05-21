pub mod syntax;
pub mod bytecode;

pub mod value;
pub mod error;
pub mod env;
pub mod native;

pub use self::native::NativeFunc;
pub use self::value::Value;
pub use self::env::Env;
pub use self::error::{RunError, RunErrorValue};

pub type RunResult<T> = Result<T, RunError>;