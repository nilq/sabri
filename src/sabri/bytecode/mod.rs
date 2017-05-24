pub mod op;
pub mod instr;
pub mod gen;
pub mod run;
pub mod closure;

pub use self::closure::Closure;
pub use self::gen::{FixupContext, Program};
pub use self::run::Run;

pub use super::syntax;
pub use syntax::parser::{ParserResult, ParserError};
pub use syntax::lexer::TokenPosition;

pub use sabri::{Value, Env, RunResult, RunError};

pub type Addr = u32;
pub const INVALID: Addr = -1i32 as Addr;