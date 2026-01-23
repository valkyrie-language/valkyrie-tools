use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};
use crate::mir::bytecode::OpCode;

pub struct WasmCompiler {
    module: Module,
    codes: CodeSection,
    funcs: FunctionSection,
    types: TypeSection,
    exports: ExportSection,
}

impl WasmCompiler {
    pub fn new() -> Self {
        Self {
            module: Module::new(),
            codes: CodeSection::new(),
            funcs: FunctionSection::new(),
            types: TypeSection::new(),
            exports: ExportSection::new(),
        }
    }

    pub fn compile(mut self, insts: &[OpCode]) -> Vec<u8> {
        // Scan for max local index
        let mut max_local = -1;
        for op in insts {
            match op {
                OpCode::LoadLocal(idx) | OpCode::StoreLocal(idx) => {
                    if *idx as i32 > max_local {
                        max_local = *idx as i32;
                    }
                }
                _ => {}
            }
        }
        let local_count = (max_local + 1) as u32;

        // Define type: () -> i64 (for main-like script)
        // Type index 0
        self.types.function([], [ValType::I64]);
        
        // Define type: (i64) -> () (for print)
        // Type index 1
        self.types.function([ValType::I64], []);

        // Import "env.print" : (i64) -> ()
        // Function index 0 (imported)
        self.module.section(&self.types);
        
        // We need to define imports BEFORE functions section in module?
        // wasm-encoder order: Type, Import, Function, ...
        // My `self.module.section` calls append sections. I must call them in order.
        
        // Let's restructure.
        let mut imports = wasm_encoder::ImportSection::new();
        imports.import("env", "print", wasm_encoder::EntityType::Function(1)); // Type index 1
        
        // Function index 1 (internal main) uses Type index 0
        self.funcs.function(0);
        
        // Export "main" as Function index 1 (since index 0 is import)
        self.exports.export("main", ExportKind::Func, 1);

        // Define locals
        // All locals are i64 for now
        let mut func = Function::new([(local_count, ValType::I64)]);
        
        for op in insts {
            match op {
                OpCode::PushInt(v) => {
                    func.instruction(&Instruction::I64Const(*v));
                }
                OpCode::PushBool(b) => {
                    func.instruction(&Instruction::I64Const(if *b { 1 } else { 0 }));
                }
                OpCode::LoadLocal(idx) => {
                    func.instruction(&Instruction::LocalGet(*idx as u32));
                }
                OpCode::StoreLocal(idx) => {
                    func.instruction(&Instruction::LocalSet(*idx as u32));
                }
                OpCode::Add => {
                    func.instruction(&Instruction::I64Add);
                }
                OpCode::Sub => {
                    func.instruction(&Instruction::I64Sub);
                }
                OpCode::Mul => {
                    func.instruction(&Instruction::I64Mul);
                }
                OpCode::Div => {
                    func.instruction(&Instruction::I64DivS);
                }
                OpCode::Eq => {
                    func.instruction(&Instruction::I64Eq);
                }
                OpCode::Neq => {
                    func.instruction(&Instruction::I64Ne);
                }
                OpCode::Lt => {
                    func.instruction(&Instruction::I64LtS);
                }
                OpCode::Gt => {
                    func.instruction(&Instruction::I64GtS);
                }
                OpCode::Le => {
                    func.instruction(&Instruction::I64LeS);
                }
                OpCode::Ge => {
                    func.instruction(&Instruction::I64GeS);
                }
                OpCode::Print => {
                    // Call imported print (index 0)
                    func.instruction(&Instruction::Call(0));
                }
                OpCode::Return => {
                    // MIR Return usually means "return top of stack"
                    // In WASM, we just return.
                    // But if stack is empty (Void), we might need to push a dummy 0 if our signature is -> i64.
                    // For now, assume stack has i64.
                    // func.instruction(&Instruction::Return);
                    // Actually, 'Return' in MIR might be used in middle.
                    // For main body, let's just let it flow.
                }
                OpCode::Pop => {
                    func.instruction(&Instruction::Drop);
                }
                _ => {
                    // println!("Warning: Skipping OpCode {:?} in WASM backend", op);
                }
            }
        }
        
        // End marker for the function body
        func.instruction(&Instruction::End);
        
        self.codes.function(&func);

        // Section order is important!
        // 1. Type
        // 2. Import
        // 3. Function
        // ...
        // 7. Export
        // 10. Code
        
        // self.types is already filled
        // self.module.section(&self.types); // Done inside compile? No, I need to add it to module.
        
        // Create a new module builder or reset self.module?
        // self.module = Module::new(); // Reset
        // Actually I should just append.
        
        let mut module = Module::new();
        module.section(&self.types);
        module.section(&imports);
        module.section(&self.funcs);
        module.section(&self.exports);
        module.section(&self.codes);

        module.finish()
    }
}
