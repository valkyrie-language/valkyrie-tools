use crate::mir::{
    bytecode::OpCode,
    debug::DebugInfo,
    value::{EvalError, GeneratorStream, RuntimeValue},
};
use async_recursion::async_recursion;
use std::{collections::HashMap, sync::Arc};

struct CallFrame {
    instructions: Arc<Vec<OpCode>>,
    ip: usize,
    locals: Vec<RuntimeValue>,
    debug_info: DebugInfo,
    bp: usize,
}

pub struct VM {
    stack: Vec<RuntimeValue>,
    globals: HashMap<String, RuntimeValue>,
    frames: Vec<CallFrame>,
    yield_sender: Option<tokio::sync::mpsc::Sender<RuntimeValue>>,
}

impl VM {
    pub fn new(args: Vec<String>) -> Self {
        let mut globals = HashMap::new();
        crate::stdlib::register(&mut globals, args);

        Self { stack: Vec::new(), globals, frames: Vec::new(), yield_sender: None }
    }

    pub async fn run(&mut self, instructions: Arc<Vec<OpCode>>, debug_info: DebugInfo) -> Result<RuntimeValue, EvalError> {
        // Initial frame
        self.frames.push(CallFrame { instructions, ip: 0, locals: vec![RuntimeValue::Void; 256], debug_info, bp: 0 });

        while !self.frames.is_empty() {
            // Get instruction and advance IP, then drop borrow of frames so we can mutate self in match
            let op = {
                let frame = self.frames.last_mut().unwrap();
                if frame.ip >= frame.instructions.len() {
                    // End of function, implicit return void if stack empty or pop result
                    self.frames.pop();
                    if self.frames.is_empty() {
                        return Ok(self.stack.pop().unwrap_or(RuntimeValue::Void));
                    }
                    continue;
                }
                let op = frame.instructions[frame.ip].clone();
                frame.ip += 1;
                op
            };

            match &op {
                // OpCode::PushInt(v) => self.stack.push(RuntimeValue::Int(*v)),
                _ => {
                      let ip = self.frames.last().unwrap().ip;
                      // println!("DEBUG: IP={} OP={:?} StackDepth={}", ip - 1, op, self.stack.len());
                }
            }

            match &op {
                OpCode::PushInt(v) => self.stack.push(RuntimeValue::Int(*v)),
                OpCode::PushFloat(v) => self.stack.push(RuntimeValue::Float(*v)),
                OpCode::PushBool(v) => self.stack.push(RuntimeValue::Bool(*v)),
                OpCode::PushString(v) => self.stack.push(RuntimeValue::String(v.clone())),
                OpCode::PushVoid => self.stack.push(RuntimeValue::Void),
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::Dup => {
                    let val = self.stack.last().ok_or(EvalError::Unknown("Stack underflow".into()))?.clone();
                    self.stack.push(val);
                }

                OpCode::LoadLocal(idx) => {
                    let val = self.frames.last().unwrap().locals[*idx].clone();
                    self.stack.push(val);
                }
                OpCode::StoreLocal(idx) => {
                    let val = self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    self.frames.last_mut().unwrap().locals[*idx] = val;
                }
                OpCode::LoadGlobal(name) => {
                    if let Some(val) = self.globals.get(name) {
                        self.stack.push(val.clone());
                    }
                    else {
                        return Err(EvalError::Unknown(format!("Unknown global: {}", name)));
                    }
                }
                OpCode::StoreGlobal(name) => {
                    let val = self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    self.globals.insert(name.clone(), val);
                }

                OpCode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => self.stack.push(RuntimeValue::Int(a + b)),
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => self.stack.push(RuntimeValue::Float(a + b)),
                        (RuntimeValue::String(a), RuntimeValue::String(b)) => self.stack.push(RuntimeValue::String(a + &b)),
                        _ => return Err(EvalError::TypeError { expected: "Number or String".into(), got: "Other".into() }),
                    }
                }
                OpCode::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => self.stack.push(RuntimeValue::Int(a - b)),
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => self.stack.push(RuntimeValue::Float(a - b)),
                        _ => return Err(EvalError::TypeError { expected: "Number".into(), got: "Other".into() }),
                    }
                }
                OpCode::Mul => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => self.stack.push(RuntimeValue::Int(a * b)),
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => self.stack.push(RuntimeValue::Float(a * b)),
                        _ => return Err(EvalError::TypeError { expected: "Number".into(), got: "Other".into() }),
                    }
                }
                OpCode::Div => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => self.stack.push(RuntimeValue::Int(a / b)),
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => self.stack.push(RuntimeValue::Float(a / b)),
                        _ => return Err(EvalError::TypeError { expected: "Number".into(), got: "Other".into() }),
                    }
                }
                OpCode::Rem => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => self.stack.push(RuntimeValue::Int(a % b)),
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => self.stack.push(RuntimeValue::Float(a % b)),
                        _ => return Err(EvalError::TypeError { expected: "Number".into(), got: "Other".into() }),
                    }
                }
                OpCode::Jump(target) => {
                    self.frames.last_mut().unwrap().ip = *target;
                }
                OpCode::JumpIfFalse(target) => {
                    let val = self.stack.last().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    // println!("DEBUG: JumpIfFalse val={:?}", val);
                    match val {
                        RuntimeValue::Bool(b) => {
                            if !*b {
                                self.frames.last_mut().unwrap().ip = *target;
                            }
                        }
                        _ => return Err(EvalError::TypeError { expected: "Bool".into(), got: format!("{:?}", val) }),
                    }
                }
                OpCode::JumpIfTrue(target) => {
                    let val = self.stack.last().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    match val {
                        RuntimeValue::Bool(b) => {
                            if *b {
                                self.frames.last_mut().unwrap().ip = *target;
                            }
                        }
                        _ => return Err(EvalError::TypeError { expected: "Bool".into(), got: "Other".into() }),
                    }
                }
                OpCode::Eq => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(RuntimeValue::Bool(a == b));
                }
                OpCode::Neq => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(RuntimeValue::Bool(a != b));
                }
                OpCode::Lt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => self.stack.push(RuntimeValue::Bool(a < b)),
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => self.stack.push(RuntimeValue::Bool(a < b)),
                        _ => return Err(EvalError::TypeError { expected: "Number".into(), got: "Other".into() }),
                    }
                }
                OpCode::Le => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => self.stack.push(RuntimeValue::Bool(a <= b)),
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => self.stack.push(RuntimeValue::Bool(a <= b)),
                        _ => return Err(EvalError::TypeError { expected: "Number".into(), got: "Other".into() }),
                    }
                }
                OpCode::Gt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => self.stack.push(RuntimeValue::Bool(a > b)),
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => self.stack.push(RuntimeValue::Bool(a > b)),
                        _ => return Err(EvalError::TypeError { expected: "Number".into(), got: "Other".into() }),
                    }
                }
                OpCode::Ge => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => self.stack.push(RuntimeValue::Bool(a >= b)),
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => self.stack.push(RuntimeValue::Bool(a >= b)),
                        _ => return Err(EvalError::TypeError { expected: "Number".into(), got: "Other".into() }),
                    }
                }
                OpCode::BitAnd => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => self.stack.push(RuntimeValue::Int(a & b)),
                        (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => self.stack.push(RuntimeValue::Bool(a & b)),
                        _ => return Err(EvalError::TypeError { expected: "Int or Bool".into(), got: "Other".into() }),
                    }
                }
                OpCode::BitOr => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => self.stack.push(RuntimeValue::Int(a | b)),
                        (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => self.stack.push(RuntimeValue::Bool(a | b)),
                        _ => return Err(EvalError::TypeError { expected: "Int or Bool".into(), got: "Other".into() }),
                    }
                }
                OpCode::Not => {
                    let val = self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    match val {
                        RuntimeValue::Bool(b) => self.stack.push(RuntimeValue::Bool(!b)),
                        _ => return Err(EvalError::TypeError { expected: "Bool".into(), got: "Other".into() }),
                    }
                }
                OpCode::Neg => {
                    let val = self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    match val {
                        RuntimeValue::Int(i) => self.stack.push(RuntimeValue::Int(-i)),
                        RuntimeValue::Float(f) => self.stack.push(RuntimeValue::Float(-f)),
                        _ => return Err(EvalError::TypeError { expected: "Number".into(), got: "Other".into() }),
                    }
                }
                OpCode::Return => {
                    let frame = self.frames.pop().unwrap(); // We know frames is not empty
                    
                    if self.frames.is_empty() {
                        return Ok(self.stack.pop().unwrap_or(RuntimeValue::Void));
                    }
                    
                    // Cleanup stack
                    // Retain return value if stack is not empty (stack might be empty if void return)
                    // Usually Return means top of stack is return value
                    // But if stack size > bp, we have garbage.
                    // If stack size < bp, underflow?
                    
                    let ret_val = self.stack.pop().unwrap_or(RuntimeValue::Void);
                    self.stack.truncate(frame.bp);
                    self.stack.push(ret_val);
                }
                OpCode::Yield => {
                    let val = self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    if let Some(tx) = &self.yield_sender {
                        tx.send(val).await.map_err(|_| EvalError::Unknown("Yield failed".into()))?;
                        self.stack.push(RuntimeValue::Void);
                    }
                    else {
                        return Err(EvalError::Unknown("Yield outside generator".into()));
                    }
                }
                OpCode::Await => {
                    let val = self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    if let RuntimeValue::Future(f) = val {
                        let mut fut = f.0.lock().await;
                        let result = fut.as_mut().await?;
                        self.stack.push(result);
                    }
                    else {
                        return Err(EvalError::TypeError { expected: "Future".into(), got: "Other".into() });
                    }
                }
                OpCode::DeclareFunction { name, params, body, debug_info, is_async, is_generator } => {
                    let func = RuntimeValue::MirFunction {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                        debug_info: debug_info.clone(),
                        owner: None,
                        is_async: *is_async,
                        is_generator: *is_generator,
                    };
                    self.globals.insert(name.clone(), func);
                }
                OpCode::Lambda { params, body, debug_info, is_async, is_generator } => {
                    let func = RuntimeValue::MirFunction {
                        name: "lambda".to_string(),
                        params: params.clone(),
                        body: body.clone(),
                        debug_info: debug_info.clone(),
                        owner: None,
                        is_async: *is_async,
                        is_generator: *is_generator,
                    };
                    self.stack.push(func);
                }
                OpCode::Print => {
                    let val = self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    match val {
                        RuntimeValue::Int(i) => println!("{}", i),
                        RuntimeValue::Float(f) => println!("{}", f),
                        RuntimeValue::Bool(b) => println!("{}", b),
                        RuntimeValue::String(s) => println!("{}", s),
                        RuntimeValue::Void => println!("void"),
                        _ => println!("{:?}", val),
                    }
                }
                OpCode::DeclareClass { name, parents, fields, methods } => {
                    let mut method_map = HashMap::new();
                    for m in methods {
                        let new_func = RuntimeValue::MirFunction {
                            name: m.name.clone(),
                            params: m.params.clone(),
                            body: m.body.clone(),
                            debug_info: m.debug_info.clone(),
                            owner: Some(name.clone()),
                            is_async: m.is_async,
                            is_generator: m.is_generator,
                        };

                        if let Some(existing) = method_map.get(&m.name) {
                            let mut variants = Vec::new();
                            match existing {
                                RuntimeValue::MirFunction { .. } => {
                                    variants.push(existing.clone());
                                    variants.push(new_func);
                                }
                                RuntimeValue::OverloadedFunction { variants: existing_variants, .. } => {
                                    variants.extend(existing_variants.clone());
                                    variants.push(new_func);
                                }
                                _ => {
                                    variants.push(new_func);
                                }
                            }
                            method_map
                                .insert(m.name.clone(), RuntimeValue::OverloadedFunction { name: m.name.clone(), variants });
                        }
                        else {
                            method_map.insert(m.name.clone(), new_func);
                        }
                    }

                    let mut mro = vec![name.clone()];
                    // Naive MRO: just append parents' MROs
                    for p_name in parents {
                        if let Some(RuntimeValue::Class { mro: p_mro, .. }) = self.globals.get(p_name) {
                            for m in p_mro {
                                if !mro.contains(m) {
                                    mro.push(m.clone());
                                }
                            }
                        }
                        else {
                            if !mro.contains(p_name) {
                                mro.push(p_name.clone());
                            }
                        }
                    }

                    let mut all_fields = HashMap::new();
                    // Collect fields from MRO (reverse order: base -> derived)
                    for c_name in mro.iter().rev() {
                        if c_name == name {
                            // Add own fields
                            for f in fields {
                                all_fields.insert(f.clone(), "void".to_string());
                            }
                        }
                        else {
                            // Add parent fields
                            if let Some(RuntimeValue::Class { fields: p_fields, .. }) = self.globals.get(c_name) {
                                for (fname, ftype) in p_fields {
                                    all_fields.insert(fname.clone(), ftype.clone());
                                }
                            }
                        }
                    }

                    let fields_mapped = all_fields.into_iter().collect();

                    let class_val = RuntimeValue::Class {
                        name: name.clone(),
                        parents: parents.iter().map(|p| (None, p.clone())).collect(),
                        mro,
                        fields: fields_mapped,
                        methods: method_map,
                    };
                    self.globals.insert(name.clone(), class_val);
                }

                OpCode::DeclareTrait { name, methods } => {
                    self.globals.insert(
                        name.clone(),
                        RuntimeValue::Trait { name: name.clone(), parents: Vec::new(), methods: methods.clone() },
                    );
                }
                OpCode::ExtendClass { name, trait_name, methods } => {
                    // Logic to check trait requirements could be here, but skipping for now or partially implemented
                    // Check if class exists
                    if let Some(RuntimeValue::Class { methods: class_methods, .. }) = self.globals.get_mut(name) {
                        for m in methods {
                            let new_func = RuntimeValue::MirFunction {
                                name: m.name.clone(),
                                params: m.params.clone(),
                                body: m.body.clone(),
                                debug_info: m.debug_info.clone(),
                                owner: Some(name.clone()),
                                is_async: m.is_async,
                                is_generator: m.is_generator,
                            };

                            if let Some(existing) = class_methods.get(&m.name) {
                                let mut variants = Vec::new();
                                match existing {
                                    RuntimeValue::MirFunction { .. } => {
                                        variants.push(existing.clone());
                                        variants.push(new_func);
                                    }
                                    RuntimeValue::OverloadedFunction { variants: existing_variants, .. } => {
                                        variants.extend(existing_variants.clone());
                                        variants.push(new_func);
                                    }
                                    _ => {
                                        variants.push(new_func);
                                    }
                                }
                                class_methods.insert(
                                    m.name.clone(),
                                    RuntimeValue::OverloadedFunction { name: m.name.clone(), variants },
                                );
                            }
                            else {
                                class_methods.insert(m.name.clone(), new_func);
                            }
                        }
                    }
                    else {
                        return Err(EvalError::Unknown(format!("Unknown class for extension: {}", name)));
                    }
                }
                OpCode::Call(name, arg_count) => {
                    if let Some(func) = self.globals.get(name) {
                        self.call_value(func.clone(), *arg_count).await?;
                    }
                    else {
                        return Err(EvalError::Unknown(format!("Unknown function: {}", name)));
                    }
                }
                OpCode::CallStack(arg_count) => {
                    let callee = self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    self.call_value(callee, *arg_count).await?;
                }
                OpCode::CallMethod(name, arg_count) => {
                    self.call_method(name, *arg_count).await?;
                }

                OpCode::GetField(name) => {
                    let obj = self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    if let RuntimeValue::Object(data) = obj {
                        let d = data.lock().unwrap();
                        if let Some(val) = d.fields.get(name) {
                            self.stack.push(val.clone());
                        }
                        else {
                            return Err(EvalError::Unknown(format!("Unknown field: {}", name)));
                        }
                    }
                    else {
                        return Err(EvalError::TypeError { expected: "Object".into(), got: "Other".into() });
                    }
                }
                OpCode::SetField(name) => {
                    let val = self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    let obj = self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?;
                    
                    // println!("DEBUG: SetField {} on {:?}", name, obj);
                    // println!("DEBUG: SetField {} on {:?}", name, obj);
                    
                    if let RuntimeValue::Object(data) = obj {
                        let mut d = data.lock().unwrap();
                        d.fields.insert(name.clone(), val);
                        self.stack.push(RuntimeValue::Void);
                    }
                    else {
                        return Err(EvalError::TypeError { expected: "Object".into(), got: "Other".into() });
                    }
                }
                OpCode::New(class_name, arg_count) => {
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(self.stack.pop().unwrap());
                    }
                    args.reverse();

                    if let Some(RuntimeValue::Class { fields, .. }) = self.globals.get(class_name) {
                        let mut obj_fields = HashMap::new();
                        for (fname, _) in fields {
                            obj_fields.insert(fname.clone(), RuntimeValue::Void);
                        }

                        let obj_data = crate::mir::value::ObjectData { class: class_name.clone(), fields: obj_fields };
                        let obj = RuntimeValue::Object(std::sync::Arc::new(std::sync::Mutex::new(obj_data)));

                        self.stack.push(obj.clone()); // The object to return

                        let has_init = if let Some(RuntimeValue::Class { mro, .. }) = self.globals.get(class_name) {
                            let mut found = false;
                            for c_name in mro {
                                if let Some(RuntimeValue::Class { methods, .. }) = self.globals.get(c_name) {
                                    if methods.contains_key("constructor") {
                                        found = true;
                                        break;
                                    } else {
                                        println!("DEBUG: constructor not found in class {}. Methods: {:?}", c_name, methods.keys());
                                    }
                                }
                            }
                            found
                        }
                        else {
                            false
                        };

                        if has_init {
                            let args_len = args.len();
                            self.stack.push(obj.clone()); // 'self' for constructor
                            for arg in args {
                                self.stack.push(arg);
                            }

                            // Inject Shim Frame to pop constructor result
                            let shim_code = std::sync::Arc::new(vec![OpCode::Pop, OpCode::Return]);
                            
                            // We want to return the Object which is below 'self'.
                            // Stack: [..., Object, Self, Args...]
                            // Object is at `self.stack.len() - args.len() - 2`.
                            let base_bp = self.stack.len() - args_len - 2; 
                            
                            self.frames.push(CallFrame {
                                instructions: shim_code,
                                ip: 0,
                                locals: Vec::new(),
                                debug_info: DebugInfo::new(),
                                bp: base_bp,
                            });

                            self.call_method("constructor", *arg_count).await?;
                            // self.stack.pop(); // Do NOT pop here, shim will do it
                        }
                        else {
                            if *arg_count > 0 {
                                return Err(EvalError::ArgumentCountMismatch { expected: 0, got: *arg_count });
                            }
                        }
                    }
                    else {
                        println!("DEBUG: Unknown class '{}'. Available globals: {:?}", class_name, self.globals.keys());
                        return Err(EvalError::Unknown(format!("Unknown class: {}", class_name)));
                    }
                }
                OpCode::NewList(count) => {
                    let mut elements = Vec::new();
                    for _ in 0..*count {
                        elements.push(self.stack.pop().unwrap());
                    }
                    elements.reverse();
                    self.stack.push(RuntimeValue::List(std::sync::Arc::new(std::sync::Mutex::new(elements))));
                }
                OpCode::GetIndex => {
                    let index = self.stack.pop().unwrap();
                    let target = self.stack.pop().unwrap();
                    
                    // println!("DEBUG: GetIndex target={:?}, index={:?}", target, index);

                    if let RuntimeValue::List(list_arc) = target {
                        let list = list_arc.lock().unwrap();
                        if let RuntimeValue::Int(idx) = index {
                            let i = if idx < 0 { list.len() as i64 + idx } else { idx };
                            if i >= 0 && i < list.len() as i64 {
                                self.stack.push(list[i as usize].clone());
                            }
                            else {
                                return Err(EvalError::Unknown(format!("Index out of bounds: {}", idx)));
                            }
                        }
                        else {
                            return Err(EvalError::TypeError { expected: "Int".into(), got: format!("{:?}", index) });
                        }
                    }
                    else if let RuntimeValue::Object(data) = target {
                         let d = data.lock().unwrap();
                         if let RuntimeValue::String(key) = index {
                             if let Some(val) = d.fields.get(&key) {
                                 self.stack.push(val.clone());
                             } else {
                                 self.stack.push(RuntimeValue::Void); // Return void/null if not found, or error?
                                 // For map behavior, returning null/void is often better than crashing
                             }
                         } else {
                             return Err(EvalError::TypeError { expected: "String".into(), got: format!("{:?}", index) });
                         }
                    }
                    else {
                        return Err(EvalError::TypeError { expected: "List or Object".into(), got: format!("{:?}", target) });
                    }
                }
                OpCode::Label(_) => {}
            }
        }

        Ok(RuntimeValue::Void)
    }

    #[async_recursion]
    async fn call_value(&mut self, callee: RuntimeValue, arg_count: usize) -> Result<(), EvalError> {
        match callee {
            RuntimeValue::NativeFunction { func, .. } => {
                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?);
                }
                args.reverse();
                let result = (func.0)(args)?;
                self.stack.push(result);
                Ok(())
            }
            RuntimeValue::MirFunction { ref params, body, debug_info, is_async, is_generator, .. } => {
                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.stack.pop().ok_or(EvalError::Unknown("Stack underflow".into()))?);
                }
                args.reverse();

                let mut new_locals = vec![RuntimeValue::Void; 256];
                for (i, arg) in args.into_iter().enumerate() {
                    if i < new_locals.len() {
                        new_locals[i] = arg;
                    }
                }

                if is_generator {
                    let (tx, rx) = tokio::sync::mpsc::channel(32);
                    let mut vm = VM::new(Vec::new()); // TODO: Pass args?
                    vm.globals = self.globals.clone();

                    tokio::spawn(async move {
                        let _ = vm.run(body, debug_info).await;
                    });

                    self.stack.push(RuntimeValue::Generator(crate::mir::value::GeneratorStream(std::sync::Arc::new(
                        tokio::sync::Mutex::new(rx),
                    ))));
                }
                else if is_async {
                    let mut vm = VM::new(Vec::new()); // TODO: Pass args?
                    vm.globals = self.globals.clone();

                    let fut = async move { vm.run(body, debug_info).await };
                    self.stack.push(RuntimeValue::Future(crate::mir::value::AsyncFuture(std::sync::Arc::new(
                        tokio::sync::Mutex::new(Box::pin(fut)),
                    ))));
                }
                else {
                    let bp = self.stack.len();
                    self.frames.push(CallFrame { instructions: body, ip: 0, locals: new_locals, debug_info, bp });
                }
                
                Ok(())
            }
            RuntimeValue::OverloadedFunction { ref variants, .. } => {
                let resolved = resolve_overload(callee.clone(), arg_count);
                if let Some(m) = resolved {
                    self.call_value(m, arg_count).await
                }
                else {
                    Err(EvalError::Unknown("No matching overload".into()))
                }
            }
            _ => Err(EvalError::TypeError { expected: "Function".into(), got: format!("{:?}", callee) }),
        }
    }

    async fn call_method(&mut self, name: &str, arg_count: usize) -> Result<(), EvalError> {
        // Peek object from stack (it's at stack.len() - 1 - arg_count)
        let stack_len = self.stack.len();
        if stack_len < arg_count + 1 {
            return Err(EvalError::Unknown("Stack underflow in method call".into()));
        }
        let obj_idx = stack_len - 1 - arg_count;
        let obj = self.stack[obj_idx].clone();

        // Handle primitive methods
        match &obj {
            RuntimeValue::String(s) => {
                if name == "len" {
                    if arg_count != 0 {
                        return Err(EvalError::ArgumentCountMismatch { expected: 0, got: arg_count });
                    }
                    // Pop obj (and args if any, but arg_count is 0)
                    let _ = self.stack.pop();
                    self.stack.push(RuntimeValue::Int(s.len() as i64));
                    return Ok(());
                }
                else if name == "chars" {
                    if arg_count != 0 {
                        return Err(EvalError::ArgumentCountMismatch { expected: 0, got: arg_count });
                    }
                    let _ = self.stack.pop();
                    let chars: Vec<RuntimeValue> = s.chars().map(|c| RuntimeValue::String(c.to_string())).collect();
                    self.stack.push(RuntimeValue::List(std::sync::Arc::new(std::sync::Mutex::new(chars))));
                    return Ok(());
                }
                else if name == "starts_with" {
                    if arg_count != 1 {
                        return Err(EvalError::ArgumentCountMismatch { expected: 1, got: arg_count });
                    }
                    let prefix_val = self.stack.pop().unwrap();
                    let _ = self.stack.pop(); // obj
                    
                    if let RuntimeValue::String(prefix) = prefix_val {
                        self.stack.push(RuntimeValue::Bool(s.starts_with(&prefix)));
                        return Ok(());
                    } else {
                         return Err(EvalError::TypeError { expected: "String".into(), got: format!("{:?}", prefix_val) });
                    }
                }
            }
            RuntimeValue::List(l) => {
                if name == "len" {
                    if arg_count != 0 {
                        return Err(EvalError::ArgumentCountMismatch { expected: 0, got: arg_count });
                    }
                    let _ = self.stack.pop();
                    let len = l.lock().unwrap().len();
                    self.stack.push(RuntimeValue::Int(len as i64));
                    return Ok(());
                }
                if name == "push" {
                    if arg_count != 1 {
                        return Err(EvalError::ArgumentCountMismatch { expected: 1, got: arg_count });
                    }
                    let item = self.stack.pop().unwrap();
                    let _ = self.stack.pop(); // obj
                    l.lock().unwrap().push(item);
                    self.stack.push(RuntimeValue::Void);
                    return Ok(());
                }
                if name == "pop" {
                    if arg_count != 0 {
                        return Err(EvalError::ArgumentCountMismatch { expected: 0, got: arg_count });
                    }
                    let _ = self.stack.pop(); // obj
                    let val = l.lock().unwrap().pop().unwrap_or(RuntimeValue::Void);
                    self.stack.push(val);
                    return Ok(());
                }
                if name == "set" {
                    if arg_count != 2 {
                        return Err(EvalError::ArgumentCountMismatch { expected: 2, got: arg_count });
                    }
                    let val = self.stack.pop().unwrap();
                    let index_val = self.stack.pop().unwrap();
                    let _ = self.stack.pop(); // obj

                    let mut list = l.lock().unwrap();
                    if let RuntimeValue::Int(idx) = index_val {
                        let i = if idx < 0 { list.len() as i64 + idx } else { idx };
                        if i >= 0 && i < list.len() as i64 {
                            list[i as usize] = val;
                            self.stack.push(RuntimeValue::Void);
                            return Ok(());
                        }
                        else {
                            return Err(EvalError::Unknown(format!("Index out of bounds: {}", idx)));
                        }
                    }
                    else {
                        return Err(EvalError::TypeError { expected: "Int".into(), got: format!("{:?}", index_val) });
                    }
                }
            }
            RuntimeValue::Object(data) => {
                if name == "set" {
                     if arg_count != 2 {
                         return Err(EvalError::ArgumentCountMismatch { expected: 2, got: arg_count });
                     }
                     let val = self.stack.pop().unwrap();
                     let key_val = self.stack.pop().unwrap();
                     let _ = self.stack.pop(); // obj

                     if let RuntimeValue::String(key) = key_val {
                         let mut d = data.lock().unwrap();
                         d.fields.insert(key, val);
                         self.stack.push(RuntimeValue::Void);
                         return Ok(());
                     } else {
                         return Err(EvalError::TypeError { expected: "String".into(), got: "Other".into() });
                     }
                }
            }
            _ => {}
        }

        // Resolve method for Objects
        let method = match obj {
            RuntimeValue::Object(data) => {
                let d = data.lock().unwrap();
                let class_name = &d.class;

                // Find class
                if let Some(RuntimeValue::Class { mro, .. }) = self.globals.get(class_name) {
                    let mut found_method = None;
                    for c_name in mro {
                        if let Some(RuntimeValue::Class { methods, .. }) = self.globals.get(c_name) {
                            if let Some(m) = methods.get(name) {
                                found_method = Some(m.clone());
                                break;
                            }
                        }
                    }
                    found_method
                }
                else {
                    None
                }
            }
            _ => return Err(EvalError::TypeError { expected: "Object".into(), got: format!("{:?}", obj) }),
        };

        if let Some(m) = method {
            // We need to check if we should pass 'self'.
            // The object is already on the stack at the correct position for 'self' if the method expects it.
            // But call_value expects 'arg_count' args to pop.
            // If we call with 'arg_count + 1', it will pop 'obj' + args.
            // This is correct for instance methods.
            // But what if it's a static method called on object? (Not typical but possible)
            // For now assume all methods called via call_method are instance methods requiring self.

            self.call_value(m, arg_count + 1).await
        }
        else {
            Err(EvalError::Unknown(format!("Method not found: {}", name)))
        }
    }
}

// Helper needed for resolve_overload
fn resolve_overload(method: RuntimeValue, arg_count: usize) -> Option<RuntimeValue> {
    match method {
        RuntimeValue::Function { ref params, .. } | RuntimeValue::MirFunction { ref params, .. } => {
            // Check param count.
            // Note: For MirFunction, params include 'self' if it's an instance method.
            // But here arg_count might or might not include self depending on context.
            // In call_value, arg_count is what we are passing.
            // If we are passing self, arg_count includes it.
            if params.len() == arg_count {
                Some(method.clone())
            }
            else {
                None
            }
        }
        RuntimeValue::OverloadedFunction { ref variants, .. } => {
            for v in variants {
                if let Some(m) = resolve_overload(v.clone(), arg_count) {
                    return Some(m);
                }
            }
            None
        }
        _ => None,
    }
}
