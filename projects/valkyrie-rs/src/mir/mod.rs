pub mod bytecode;
pub mod debug;
pub mod interpreter;
pub mod value;
pub mod vm;
pub mod wasm;

pub use interpreter::eval_program;
pub use value::{EvalError, RuntimeValue};
