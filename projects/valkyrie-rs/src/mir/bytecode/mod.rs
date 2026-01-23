use crate::{ast::BinaryOperator, mir::debug::DebugInfo};

use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct ClassMethod {
    pub name: String,
    pub params: Vec<String>,
    pub body: Arc<Vec<OpCode>>,
    pub debug_info: DebugInfo,
    pub is_async: bool,
    pub is_generator: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // Stack Manipulation
    PushInt(i64),
    PushFloat(f64),
    PushBool(bool),
    PushString(String),
    PushVoid,
    Pop,
    Dup,

    // Variables
    LoadLocal(usize), // Index in stack frame (args + locals)
    StoreLocal(usize),
    LoadGlobal(String),
    StoreGlobal(String),

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    BitAnd,
    BitOr,
    Not,
    Neg,

    // Control Flow
    Label(usize), // Pseudo-instruction for compiler, removed/resolved in final bytecode? Or just keep for VM jump targets
    Jump(usize),  // Absolute index
    JumpIfFalse(usize),
    JumpIfTrue(usize),

    // Functions
    Call(String, usize),       // name, arg_count
    CallStack(usize),          // arg_count, callee is on stack
    CallMethod(String, usize), // method_name, arg_count
    Return,
    Yield,
    Await,
    DeclareFunction {
        name: String,
        params: Vec<String>,
        body: Arc<Vec<OpCode>>,
        debug_info: DebugInfo,
        is_async: bool,
        is_generator: bool,
    },
    Lambda {
        params: Vec<String>,
        body: Arc<Vec<OpCode>>,
        debug_info: DebugInfo,
        is_async: bool,
        is_generator: bool,
    },

    // Objects
    New(String, usize), // class_name, arg_count
    GetField(String),
    SetField(String),
    DeclareClass {
        name: String,
        parents: Vec<String>,
        fields: Vec<String>,
        methods: Vec<ClassMethod>,
    },
    DeclareTrait {
        name: String,
        methods: Vec<String>, // Just names of required methods
    },
    ExtendClass {
        name: String,
        trait_name: Option<String>,
        methods: Vec<ClassMethod>,
    },

    // Debug
    Print,

    // List
    NewList(usize), // count
    GetIndex,
}
