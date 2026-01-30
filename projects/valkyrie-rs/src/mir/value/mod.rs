use crate::{
    ast::{BinaryOperator, Expression, Statement},
    ast::parser::ParseError,
    mir::debug::DebugInfo,
};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

// Define OpCode here or in bytecode.rs, but RuntimeValue depends on it if we store Bytecode in Function
use crate::mir::bytecode::OpCode;

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectData {
    pub class: String,
    pub fields: HashMap<String, RuntimeValue>,
}

#[derive(Clone)]
pub struct AsyncFuture(pub Arc<tokio::sync::Mutex<Pin<Box<dyn Future<Output = Result<RuntimeValue, EvalError>> + Send>>>>);

impl std::fmt::Debug for AsyncFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Future")
    }
}
impl PartialEq for AsyncFuture {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone)]
pub struct GeneratorStream(pub Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<RuntimeValue>>>);

impl std::fmt::Debug for GeneratorStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Generator")
    }
}
impl PartialEq for GeneratorStream {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone)]
pub struct NativeFunctionWrapper(pub Arc<dyn Fn(Vec<RuntimeValue>) -> Result<RuntimeValue, EvalError> + Send + Sync>);

impl std::fmt::Debug for NativeFunctionWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NativeFunction")
    }
}
impl PartialEq for NativeFunctionWrapper {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    // AST Function (Old)
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Statement>,
        owner: Option<String>,
        is_async: bool,
        is_generator: bool,
    },
    // MIR Function (New)
    MirFunction {
        name: String,
        params: Vec<String>,
        body: Arc<Vec<OpCode>>,
        debug_info: DebugInfo,
        owner: Option<String>,
        is_async: bool,
        is_generator: bool,
    },
    NativeFunction {
        name: String,
        func: NativeFunctionWrapper,
    },

    OverloadedFunction {
        name: String,
        variants: Vec<RuntimeValue>,
    },
    Class {
        name: String,
        parents: Vec<(Option<String>, String)>,
        mro: Vec<String>,
        fields: Vec<(String, String)>,
        methods: HashMap<String, RuntimeValue>,
    },
    Trait {
        name: String,
        parents: Vec<String>,
        methods: Vec<String>,
    },
    Macro {
        name: String,
        params: Vec<String>,
        body: Box<Statement>,
    },
    Object(Arc<Mutex<ObjectData>>),
    Super {
        obj: Arc<Mutex<ObjectData>>,
        start_class: String,
    },
    Future(AsyncFuture),
    Generator(GeneratorStream),
    YieldSender(tokio::sync::mpsc::Sender<RuntimeValue>),
    Ast(Box<Expression>),
    List(Arc<Mutex<Vec<RuntimeValue>>>),
    Void,
}

// Implement Clone manually or via Arc for List if needed?
// RuntimeValue derives Clone. Vec<RuntimeValue> is Clone.
// So it should be fine.

impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => a == b,
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => a == b,
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a == b,
            (RuntimeValue::String(a), RuntimeValue::String(b)) => a == b,
            (RuntimeValue::Void, RuntimeValue::Void) => true,
            (RuntimeValue::Object(a), RuntimeValue::Object(b)) => Arc::ptr_eq(a, b),
            (RuntimeValue::Trait { name: a, .. }, RuntimeValue::Trait { name: b, .. }) => a == b,
            (RuntimeValue::Future(a), RuntimeValue::Future(b)) => a == b,
            (RuntimeValue::Generator(a), RuntimeValue::Generator(b)) => a == b,
            (RuntimeValue::NativeFunction { func: a, .. }, RuntimeValue::NativeFunction { func: b, .. }) => a == b,
            (RuntimeValue::Ast(a), RuntimeValue::Ast(b)) => a == b,
            (RuntimeValue::List(l1), RuntimeValue::List(l2)) => Arc::ptr_eq(l1, l2),
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum EvalError {
    UndefinedIdentifier(String),
    TypeError { expected: String, got: String },
    ArgumentCountMismatch { expected: usize, got: usize },
    Return(RuntimeValue),
    NotAFunction,
    UnsupportedLiteral,
    Unknown(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UndefinedIdentifier(s) => write!(f, "undefined identifier `{}`", s),
            EvalError::TypeError { expected, got } => write!(f, "type error: expected {}, got {}", expected, got),
            EvalError::ArgumentCountMismatch { expected, got } => write!(f, "argument count mismatch: expected {}, got {}", expected, got),
            EvalError::Return(_) => write!(f, "return value"),
            EvalError::NotAFunction => write!(f, "not a function"),
            EvalError::UnsupportedLiteral => write!(f, "unsupported literal"),
            EvalError::Unknown(s) => write!(f, "unknown error: {}", s),
        }
    }
}

impl std::error::Error for EvalError {}

impl From<ParseError> for EvalError {
    fn from(e: ParseError) -> Self {
        EvalError::Unknown(format!("Parse error: {:?}", e))
    }
}
