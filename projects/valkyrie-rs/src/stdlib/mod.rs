use crate::mir::value::{EvalError, NativeFunctionWrapper, RuntimeValue};
use std::{collections::HashMap, sync::Arc};

pub fn register(globals: &mut HashMap<String, RuntimeValue>, args: Vec<String>) {
    globals.insert(
        "get_args".to_string(),
        RuntimeValue::NativeFunction {
            name: "get_args".to_string(),
            func: NativeFunctionWrapper(Arc::new(move |_| {
                let list_args: Vec<RuntimeValue> = args.iter().map(|s| RuntimeValue::String(s.clone())).collect();
                Ok(RuntimeValue::List(Arc::new(std::sync::Mutex::new(list_args))))
            })),
        },
    );

    globals.insert(
        "get_env".to_string(),
        RuntimeValue::NativeFunction {
            name: "get_env".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                if args.len() != 1 {
                    return Err(EvalError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                match &args[0] {
                    RuntimeValue::String(key) => {
                        match std::env::var(key) {
                            Ok(val) => Ok(RuntimeValue::String(val)),
                            Err(_) => Ok(RuntimeValue::Void), // Or return error? Void seems safer for "not found"
                        }
                    }
                    _ => Err(EvalError::TypeError { expected: "String".into(), got: format!("{:?}", args[0]) }),
                }
            })),
        },
    );

    globals.insert(
        "cwd".to_string(),
        RuntimeValue::NativeFunction {
            name: "cwd".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                if args.len() != 0 {
                    return Err(EvalError::ArgumentCountMismatch { expected: 0, got: args.len() });
                }
                match std::env::current_dir() {
                    Ok(path) => Ok(RuntimeValue::String(path.to_string_lossy().to_string())),
                    Err(e) => Err(EvalError::Unknown(format!("Failed to get CWD: {}", e))),
                }
            })),
        },
    );

    globals.insert(
        "exit".to_string(),
        RuntimeValue::NativeFunction {
            name: "exit".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                if args.len() != 1 {
                    return Err(EvalError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                match &args[0] {
                    RuntimeValue::Int(code) => {
                        std::process::exit(*code as i32);
                    }
                    _ => Err(EvalError::TypeError { expected: "Int".into(), got: format!("{:?}", args[0]) }),
                }
            })),
        },
    );

    globals.insert(
        "print".to_string(),
        RuntimeValue::NativeFunction {
            name: "print".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                for arg in args {
                    match arg {
                        RuntimeValue::Int(i) => print!("{}", i),
                        RuntimeValue::Float(f) => print!("{}", f),
                        RuntimeValue::Bool(b) => print!("{}", b),
                        RuntimeValue::String(s) => print!("{}", s),
                        RuntimeValue::Void => print!("void"),
                        _ => print!("{:?}", arg),
                    }
                    print!(" ");
                }
                println!();
                Ok(RuntimeValue::Void)
            })),
        },
    );

    globals.insert(
        "println".to_string(),
        RuntimeValue::NativeFunction {
            name: "println".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                for arg in args {
                    match arg {
                        RuntimeValue::Int(i) => print!("{}", i),
                        RuntimeValue::Float(f) => print!("{}", f),
                        RuntimeValue::Bool(b) => print!("{}", b),
                        RuntimeValue::String(s) => print!("{}", s),
                        RuntimeValue::Void => print!("void"),
                        _ => print!("{:?}", arg),
                    }
                    print!(" ");
                }
                println!();
                Ok(RuntimeValue::Void)
            })),
        },
    );

    globals.insert(
        "sqrt".to_string(),
        RuntimeValue::NativeFunction {
            name: "sqrt".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                if args.len() != 1 {
                    return Err(EvalError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                match &args[0] {
                    RuntimeValue::Float(f) => Ok(RuntimeValue::Float(f.sqrt())),
                    RuntimeValue::Int(i) => Ok(RuntimeValue::Float((*i as f64).sqrt())),
                    _ => Err(EvalError::TypeError { expected: "Number".into(), got: format!("{:?}", args[0]) }),
                }
            })),
        },
    );

    globals.insert(
        "len".to_string(),
        RuntimeValue::NativeFunction {
            name: "len".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                if args.len() != 1 {
                    return Err(EvalError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                match &args[0] {
                    RuntimeValue::String(s) => Ok(RuntimeValue::Int(s.len() as i64)),
                    RuntimeValue::List(l) => Ok(RuntimeValue::Int(l.lock().unwrap().len() as i64)),
                    _ => Err(EvalError::TypeError { expected: "String or List".into(), got: format!("{:?}", args[0]) }),
                }
            })),
        },
    );

    globals.insert(
        "read_file".to_string(),
        RuntimeValue::NativeFunction {
            name: "read_file".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                if args.len() != 1 {
                    return Err(EvalError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                match &args[0] {
                    RuntimeValue::String(path) => match std::fs::read_to_string(path) {
                        Ok(content) => Ok(RuntimeValue::String(content)),
                        Err(err) => Err(EvalError::Unknown(format!("IO Error: {}", err))),
                    },
                    _ => Err(EvalError::TypeError { expected: "String".into(), got: format!("{:?}", args[0]) }),
                }
            })),
        },
    );

    globals.insert(
        "write_file".to_string(),
        RuntimeValue::NativeFunction {
            name: "write_file".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                if args.len() != 2 {
                    return Err(EvalError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                match (&args[0], &args[1]) {
                    (RuntimeValue::String(path), RuntimeValue::String(content)) => {
                         match std::fs::write(path, content) {
                            Ok(_) => Ok(RuntimeValue::Void),
                            Err(err) => Err(EvalError::Unknown(format!("IO Error: {}", err))),
                        }
                    }
                    _ => Err(EvalError::TypeError { expected: "String, String".into(), got: format!("{:?}, {:?}", args[0], args[1]) }),
                }
            })),
        },
    );

    globals.insert(
        "write_file_bytes".to_string(),
        RuntimeValue::NativeFunction {
            name: "write_file_bytes".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                if args.len() != 2 {
                    return Err(EvalError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let path = match &args[0] {
                    RuntimeValue::String(s) => s,
                    _ => return Err(EvalError::TypeError { expected: "String".into(), got: format!("{:?}", args[0]) }),
                };
                
                let bytes = match &args[1] {
                    RuntimeValue::List(l) => {
                        let list = l.lock().unwrap();
                        let mut vec = Vec::new();
                        for item in list.iter() {
                            match item {
                                RuntimeValue::Int(i) => vec.push(*i as u8),
                                _ => return Err(EvalError::TypeError { expected: "Int (byte)".into(), got: format!("{:?}", item) }),
                            }
                        }
                        vec
                    }
                    _ => return Err(EvalError::TypeError { expected: "List<Int>".into(), got: format!("{:?}", args[1]) }),
                };
                
                match std::fs::write(path, bytes) {
                    Ok(_) => Ok(RuntimeValue::Void),
                    Err(err) => Err(EvalError::Unknown(format!("IO Error: {}", err))),
                }
            })),
        },
    );

    globals.insert(
        "int".to_string(),
        RuntimeValue::NativeFunction {
            name: "int".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                if args.len() != 1 {
                    return Err(EvalError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                match &args[0] {
                    RuntimeValue::String(s) => match s.parse::<i64>() {
                        Ok(i) => Ok(RuntimeValue::Int(i)),
                        Err(_) => Err(EvalError::Unknown(format!("Invalid integer string: {}", s))),
                    },
                    RuntimeValue::Int(i) => Ok(RuntimeValue::Int(*i)),
                    RuntimeValue::Float(f) => Ok(RuntimeValue::Int(*f as i64)),
                    _ => Err(EvalError::TypeError { expected: "String or Number".into(), got: format!("{:?}", args[0]) }),
                }
            })),
        },
    );

    globals.insert(
        "typeof".to_string(),
        RuntimeValue::NativeFunction {
            name: "typeof".to_string(),
            func: NativeFunctionWrapper(Arc::new(|args| {
                if args.len() != 1 {
                    return Err(EvalError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                let type_name = match &args[0] {
                    RuntimeValue::Int(_) => "int",
                    RuntimeValue::Float(_) => "float",
                    RuntimeValue::Bool(_) => "bool",
                    RuntimeValue::String(_) => "string",
                    RuntimeValue::List(_) => "list",
                    RuntimeValue::Object(_) => "object",
                    RuntimeValue::Class { .. } => "class",
                    RuntimeValue::Trait { .. } => "trait",
                    RuntimeValue::MirFunction { .. } | RuntimeValue::NativeFunction { .. } => "function",
                    RuntimeValue::Void => "void",
                    _ => "unknown",
                };
                Ok(RuntimeValue::String(type_name.to_string()))
            })),
        },
    );

    globals.insert("null".to_string(), RuntimeValue::Void);
    globals.insert("void".to_string(), RuntimeValue::Void);
}
