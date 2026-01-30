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

impl OpCode {
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            OpCode::PushInt(v) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            OpCode::PushFloat(v) => {
                bytes.push(0x02);
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            OpCode::PushBool(v) => {
                bytes.push(0x03);
                bytes.push(if *v { 1 } else { 0 });
            }
            OpCode::PushString(v) => {
                bytes.push(0x04);
                let s_bytes = v.as_bytes();
                bytes.extend_from_slice(&(s_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(s_bytes);
            }
            OpCode::PushVoid => bytes.push(0x05),
            OpCode::Pop => bytes.push(0x06),
            OpCode::Dup => bytes.push(0x07),
            OpCode::LoadLocal(idx) => {
                bytes.push(0x10);
                bytes.extend_from_slice(&(*idx as u32).to_le_bytes());
            }
            OpCode::StoreLocal(idx) => {
                bytes.push(0x11);
                bytes.extend_from_slice(&(*idx as u32).to_le_bytes());
            }
            OpCode::LoadGlobal(name) => {
                bytes.push(0x12);
                let s_bytes = name.as_bytes();
                bytes.extend_from_slice(&(s_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(s_bytes);
            }
            OpCode::StoreGlobal(name) => {
                bytes.push(0x13);
                let s_bytes = name.as_bytes();
                bytes.extend_from_slice(&(s_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(s_bytes);
            }
            OpCode::Add => bytes.push(0x20),
            OpCode::Sub => bytes.push(0x21),
            OpCode::Mul => bytes.push(0x22),
            OpCode::Div => bytes.push(0x23),
            OpCode::Rem => bytes.push(0x24),
            OpCode::Eq => bytes.push(0x25),
            OpCode::Neq => bytes.push(0x26),
            OpCode::Lt => bytes.push(0x27),
            OpCode::Le => bytes.push(0x28),
            OpCode::Gt => bytes.push(0x29),
            OpCode::Ge => bytes.push(0x2a),
            OpCode::BitAnd => bytes.push(0x2b),
            OpCode::BitOr => bytes.push(0x2c),
            OpCode::Not => bytes.push(0x2d),
            OpCode::Neg => bytes.push(0x2e),
            OpCode::Jump(addr) => {
                bytes.push(0x30);
                bytes.extend_from_slice(&(*addr as u32).to_le_bytes());
            }
            OpCode::JumpIfFalse(addr) => {
                bytes.push(0x31);
                bytes.extend_from_slice(&(*addr as u32).to_le_bytes());
            }
            OpCode::JumpIfTrue(addr) => {
                bytes.push(0x32);
                bytes.extend_from_slice(&(*addr as u32).to_le_bytes());
            }
            OpCode::Call(name, args) => {
                bytes.push(0x40);
                let s_bytes = name.as_bytes();
                bytes.extend_from_slice(&(s_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(s_bytes);
                bytes.extend_from_slice(&(*args as u32).to_le_bytes());
            }
            OpCode::Return => bytes.push(0x43),
            OpCode::Print => bytes.push(0x90),
            _ => {
                // Placeholder for unimplemented encodings
                bytes.push(0xff);
            }
        }
        bytes
    }
}

pub struct MirModule {
    pub instructions: Vec<OpCode>,
}

impl MirModule {
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Magic header: NYAR
        bytes.extend_from_slice(b"NYAR");
        // Version: 1
        bytes.extend_from_slice(&1u32.to_le_bytes());
        // Instruction count
        bytes.extend_from_slice(&(self.instructions.len() as u32).to_le_bytes());
        for inst in &self.instructions {
            bytes.extend(inst.encode());
        }
        bytes
    }
}
