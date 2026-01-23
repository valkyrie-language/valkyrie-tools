use crate::{
    ast::{BinaryOperator, Expression, LiteralValue, MatchPattern, ProgramNode, Statement, TypeExpression, UnaryOperator},
    mir::value::{AsyncFuture, EvalError, GeneratorStream, ObjectData, RuntimeValue},
};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, RuntimeValue>>,
    namespaces: Vec<Vec<String>>,
    usings: Vec<HashMap<String, String>>,
}

impl Environment {
    pub fn new() -> Self {
        Self { scopes: vec![HashMap::new()], namespaces: vec![Vec::new()], usings: vec![HashMap::new()] }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.namespaces.push(self.namespaces.last().cloned().unwrap_or_default());
        self.usings.push(self.usings.last().cloned().unwrap_or_default());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
        self.namespaces.pop();
        self.usings.pop();
    }

    pub fn set_namespace(&mut self, path: Vec<String>) {
        if let Some(ns) = self.namespaces.last_mut() {
            *ns = path;
        }
    }

    pub fn add_using(&mut self, path: Vec<String>) {
        let Some(alias) = path.last().cloned()
        else {
            return;
        };
        let full = path.join("::");
        if let Some(m) = self.usings.last_mut() {
            m.insert(alias, full);
        }
    }

    fn current_namespace(&self) -> &[String] {
        self.namespaces.last().map(|v| v.as_slice()).unwrap_or(&[])
    }

    fn qualify_in_namespace(&self, name: &str) -> Option<String> {
        let ns = self.current_namespace();
        if ns.is_empty() {
            None
        }
        else {
            Some(format!("{}::{}", ns.join("::"), name))
        }
    }

    fn lookup(&self, name: &str) -> Option<RuntimeValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    fn resolve_value(&self, name: &str) -> Option<RuntimeValue> {
        if let Some(v) = self.lookup(name) {
            return Some(v);
        }
        if name.contains("::") {
            return None;
        }
        if let Some(q) = self.qualify_in_namespace(name) {
            if let Some(v) = self.lookup(&q) {
                return Some(v);
            }
        }
        if let Some(m) = self.usings.last() {
            if let Some(full) = m.get(name) {
                if let Some(v) = self.lookup(full) {
                    return Some(v);
                }
            }
        }
        None
    }

    pub fn define(&mut self, name: String, value: RuntimeValue) {
        let qualified = if self.scopes.len() == 1 && !name.contains("::") {
            self.qualify_in_namespace(&name).unwrap_or(name)
        }
        else {
            name
        };

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(qualified, value);
        }
    }

    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        self.resolve_value(name)
    }
}

fn collect_trait_methods(env: &Environment, trait_name: &str) -> Result<Vec<String>, EvalError> {
    let trait_val = env.get(trait_name).ok_or_else(|| EvalError::UndefinedIdentifier(trait_name.to_string()))?;
    if let RuntimeValue::Trait { parents, methods, .. } = trait_val {
        let mut all_methods = methods.clone();
        for p in parents {
            let p_methods = collect_trait_methods(env, &p)?;
            for pm in p_methods {
                if !all_methods.contains(&pm) {
                    all_methods.push(pm);
                }
            }
        }
        Ok(all_methods)
    }
    else {
        Err(EvalError::TypeError { expected: "Trait".into(), got: format!("{:?}", trait_val) })
    }
}

fn resolve_overload(method: RuntimeValue, arg_count: usize) -> Option<RuntimeValue> {
    match method {
        RuntimeValue::Function { ref params, .. } => {
            // Check if instance method (has self) or static
            // This context is tricky because we don't know if we injected self yet.
            // But we can check raw params count.
            // If it's an instance method, params[0] is self.
            // arg_count usually doesn't include self if called via dot?
            // Actually, let's assume arg_count is the number of arguments provided at call site.
            // If the function has 'self', it expects arg_count + 1 params.
            // If it doesn't, it expects arg_count.

            let expected = if !params.is_empty() && params[0] == "self" { params.len() - 1 } else { params.len() };

            if expected == arg_count {
                Some(method.clone())
            }
            else {
                None
            }
        }
        RuntimeValue::OverloadedFunction { variants, .. } => {
            for v in variants {
                if let Some(m) = resolve_overload(v, arg_count) {
                    return Some(m);
                }
            }
            None
        }
        _ => None,
    }
}

fn val_to_string(v: &RuntimeValue) -> String {
    match v {
        RuntimeValue::Int(i) => i.to_string(),
        RuntimeValue::Float(f) => f.to_string(),
        RuntimeValue::Bool(b) => b.to_string(),
        RuntimeValue::String(s) => s.clone(),
        RuntimeValue::Void => "void".to_string(),
        RuntimeValue::Object(_) => "Object".to_string(),
        RuntimeValue::Class { name, .. } => format!("Class {}", name),
        RuntimeValue::Trait { name, .. } => format!("Trait {}", name),
        RuntimeValue::Function { name, .. } => format!("Function {}", name),
        RuntimeValue::MirFunction { name, .. } => format!("MirFunction {}", name),
        RuntimeValue::OverloadedFunction { name, .. } => format!("OverloadedFunction {}", name),
        RuntimeValue::Macro { name, .. } => format!("Macro {}", name),
        RuntimeValue::Super { .. } => "super".to_string(),
        RuntimeValue::Future(_) => "Future".to_string(),
        RuntimeValue::Generator(_) => "Generator".to_string(),
        RuntimeValue::YieldSender(_) => "YieldSender".to_string(),
        RuntimeValue::Ast(expr) => format!("{:?}", expr),
        RuntimeValue::NativeFunction { name, .. } => format!("NativeFunction {}", name),
        RuntimeValue::List(list_arc) => {
            let elements = list_arc.lock().unwrap();
            let elems: Vec<String> = elements.iter().map(|e| val_to_string(e)).collect();
            format!("[{}]", elems.join(", "))
        }
    }
}

pub async fn eval_program(program: &ProgramNode) -> Result<RuntimeValue, EvalError> {
    let mut env = Environment::new();
    let mut last_val = RuntimeValue::Void;
    for stmt in &program.statements {
        last_val = eval_statement(stmt, &mut env).await?;
    }
    Ok(last_val)
}

// Helper to bridge Statement evaluation returning Result
#[async_recursion::async_recursion]
pub async fn eval_statement(stmt: &Statement, env: &mut Environment) -> Result<RuntimeValue, EvalError> {
    match stmt {
        Statement::Namespace { path, .. } => {
            env.set_namespace(path.clone());
            Ok(RuntimeValue::Void)
        }
        Statement::Using { path, .. } => {
            env.add_using(path.clone());
            Ok(RuntimeValue::Void)
        }
        Statement::Block { statements, .. } => {
            env.push_scope();
            let mut result = RuntimeValue::Void;
            let mut error = None;
            for s in statements {
                match eval_statement(s, env).await {
                    Ok(v) => result = v,
                    Err(e) => {
                        error = Some(e);
                        break;
                    }
                }
            }
            env.pop_scope();
            if let Some(e) = error {
                Err(e)
            }
            else {
                Ok(result)
            }
        }
        Statement::Expression { expression, .. } => {
            eval_expression(expression, env).await
        }
        Statement::Let { name, value, .. } => {
            let val = eval_expression(value, env).await?;
            env.define(name.clone(), val);
            Ok(RuntimeValue::Void)
        }
        Statement::If { condition, then_branch, else_branch, .. } => {
            let cond = eval_expression(condition, env).await?;
            if let RuntimeValue::Bool(b) = cond {
                if b {
                    eval_statement(then_branch, env).await
                }
                else if let Some(else_b) = else_branch {
                    eval_statement(else_b, env).await
                }
                else {
                    Ok(RuntimeValue::Void)
                }
            }
            else {
                Err(EvalError::TypeError { expected: "Bool".into(), got: format!("{:?}", cond) })
            }
        }
        Statement::Return { value, .. } => {
            let val = if let Some(v) = value { eval_expression(v, env).await? } else { RuntimeValue::Void };
            Err(EvalError::Return(val))
        }
        Statement::Yield { value, .. } => {
            let val = if let Some(v) = value { eval_expression(v, env).await? } else { RuntimeValue::Void };
            if let Some(RuntimeValue::YieldSender(sender)) = env.get("$yield") {
                sender.send(val).await.map_err(|_| EvalError::Return(RuntimeValue::Void))?;
                Ok(RuntimeValue::Void)
            }
            else {
                Err(EvalError::TypeError { expected: "Generator Context".into(), got: "None".into() })
            }
        }
        Statement::While { condition, body, .. } => {
            loop {
                let cond_val = eval_expression(condition, env).await?;
                match cond_val {
                    RuntimeValue::Bool(true) => match eval_statement(body, env).await {
                        Ok(_) => {}
                        Err(EvalError::Return(v)) => return Err(EvalError::Return(v)),
                        Err(e) => return Err(e),
                    },
                    RuntimeValue::Bool(false) => break,
                    _ => return Err(EvalError::TypeError { expected: "Bool".into(), got: format!("{:?}", cond_val) }),
                }
            }
            Ok(RuntimeValue::Void)
        }
        Statement::Class { name, parents, traits, fields, methods, .. } => {
            let mut method_map = HashMap::new();
            for m in methods {
                if let Statement::Function { name: m_name, params, body, is_async, is_generator, .. } = m {
                    if let Some(b) = body {
                        let func = RuntimeValue::Function {
                            name: m_name.clone(),
                            params: params.iter().map(|(n, _)| n.clone()).collect(),
                            body: b.clone(),
                            owner: Some(name.clone()),
                            is_async: *is_async,
                            is_generator: *is_generator,
                        };

                        if let Some(existing) = method_map.get_mut(m_name) {
                            if let RuntimeValue::OverloadedFunction { variants, .. } = existing {
                                variants.push(func);
                            }
                            else if let RuntimeValue::Function { .. } = existing {
                                let old_func = existing.clone();
                                *existing =
                                    RuntimeValue::OverloadedFunction { name: m_name.clone(), variants: vec![old_func, func] };
                            }
                        }
                        else {
                            method_map.insert(m_name.clone(), func);
                        }
                    }
                }
            }

            let mut parent_names_with_alias = Vec::new();
            let mut parent_names_only = Vec::new();
            for (alias, p) in parents {
                let mut current = p;
                while let TypeExpression::Generic { base, .. } = current {
                    current = base;
                }
                if let TypeExpression::Name { name: p_name, .. } = current {
                    parent_names_with_alias.push((alias.clone(), p_name.clone()));
                    parent_names_only.push(p_name.clone());
                }
            }

            for t in traits {
                let mut current = t;
                while let TypeExpression::Generic { base, .. } = current {
                    current = base;
                }
                if let TypeExpression::Name { name: t_name, .. } = current {
                    parent_names_with_alias.push((None, t_name.clone()));
                    parent_names_only.push(t_name.clone());
                }
            }

            // Compute MRO (C3 Linearization)
            let mut mro = vec![name.clone()];
            if !parent_names_only.is_empty() {
                let mut parent_mros = Vec::new();
                for p_name in &parent_names_only {
                    if let Some(RuntimeValue::Class { mro: p_mro, .. }) = env.get(p_name) {
                        parent_mros.push(p_mro.clone());
                    }
                    else {
                        return Err(EvalError::UndefinedIdentifier(p_name.clone()));
                    }
                }

                let merged = merge_mro(parent_mros, parent_names_only);
                mro.extend(merged);
            }

            let class_val = RuntimeValue::Class {
                name: name.clone(),
                parents: parent_names_with_alias,
                mro,
                fields: fields.iter().map(|(n, _)| (n.clone(), "any".to_string())).collect(),
                methods: method_map,
            };
            env.define(name.clone(), class_val);
            Ok(RuntimeValue::Void)
        }
        Statement::Trait { name, parents, methods, .. } => {
            let mut parent_names = Vec::new();
            for p in parents {
                let mut current = p;
                while let TypeExpression::Generic { base, .. } = current {
                    current = base;
                }
                if let TypeExpression::Name { name: p_name, .. } = current {
                    parent_names.push(p_name.clone());
                }
            }

            let mut method_names = Vec::new();
            for m in methods {
                if let Statement::Function { name: m_name, .. } = m {
                    method_names.push(m_name.clone());
                }
            }
            env.define(name.clone(), RuntimeValue::Trait { name: name.clone(), parents: parent_names, methods: method_names });
            Ok(RuntimeValue::Void)
        }
        Statement::Imply { target, trait_target, methods, .. } => {
            let target_name = {
                let mut current = target;
                while let TypeExpression::Generic { base, .. } = current {
                    current = base;
                }
                if let TypeExpression::Name { name, .. } = current {
                    name
                }
                else {
                    return Err(EvalError::TypeError { expected: "ClassName".into(), got: "ComplexType".into() });
                }
            };

            // Check trait if specified
            if let Some(t_expr) = trait_target {
                let t_name = {
                    let mut current = t_expr;
                    while let TypeExpression::Generic { base, .. } = current {
                        current = base;
                    }
                    if let TypeExpression::Name { name, .. } = current {
                        name
                    }
                    else {
                        return Err(EvalError::TypeError { expected: "TraitName".into(), got: "ComplexType".into() });
                    }
                };

                let required_methods = collect_trait_methods(env, t_name)?;

                // Verify all trait methods are implemented
                let impl_methods: Vec<String> = methods
                    .iter()
                    .filter_map(|m| {
                        if let Statement::Function { name, .. } = m {
                            Some(name.clone())
                        }
                        else {
                            None
                        }
                    })
                    .collect();

                for tm in required_methods {
                    if !impl_methods.contains(&tm) {
                        return Err(EvalError::TypeError {
                            expected: format!("Method implementation for {}", tm),
                            got: "Missing".into(),
                        });
                    }
                }
            }

            let mut class_val = env.get(target_name).ok_or_else(|| EvalError::UndefinedIdentifier(target_name.clone()))?;

            if let RuntimeValue::Class { methods: ref mut class_methods, .. } = &mut class_val {
                for m in methods {
                    if let Statement::Function { name: m_name, params, body, is_async, is_generator, .. } = m {
                        if let Some(b) = body {
                            let func = RuntimeValue::Function {
                                name: m_name.clone(),
                                params: params.iter().map(|(n, _)| n.clone()).collect(),
                                body: b.clone(),
                                owner: Some(target_name.clone()),
                                is_async: *is_async,
                                is_generator: *is_generator,
                            };

                            if let Some(existing) = class_methods.get_mut(m_name) {
                                if let RuntimeValue::OverloadedFunction { variants, .. } = existing {
                                    variants.push(func);
                                }
                                else if let RuntimeValue::Function { .. } = existing {
                                    let old_func = existing.clone();
                                    *existing = RuntimeValue::OverloadedFunction {
                                        name: m_name.clone(),
                                        variants: vec![old_func, func],
                                    };
                                }
                            }
                            else {
                                class_methods.insert(m_name.clone(), func);
                            }
                        }
                    }
                }
                env.define(target_name.clone(), class_val);
                Ok(RuntimeValue::Void)
            }
            else {
                Err(EvalError::TypeError { expected: "Class".into(), got: format!("{:?}", class_val) })
            }
        }
        Statement::Function { name, params, body, is_async, is_generator, .. } => {
            if let Some(b) = body {
                let func = RuntimeValue::Function {
                    name: name.clone(),
                    params: params.iter().map(|(n, _)| n.clone()).collect(),
                    body: b.clone(),
                    owner: None,
                    is_async: *is_async,
                    is_generator: *is_generator,
                };
                env.define(name.clone(), func);
            }
            Ok(RuntimeValue::Void)
        }
        Statement::Macro { name, params, body, .. } => {
            let macro_val = RuntimeValue::Macro {
                name: name.clone(),
                params: params.iter().map(|(n, _)| n.clone()).collect(),
                body: body.clone(),
            };
            env.define(name.clone(), macro_val);
            Ok(RuntimeValue::Void)
        }
        Statement::Annotation { target, .. } => {
            // TODO: Implement annotation expansion (this should happen before runtime)
            // For now, just execute the target statement
            eval_statement(target, env).await
        }
        Statement::Expression { expression, .. } => eval_expression(expression, env).await,
    }
}

#[async_recursion::async_recursion]
pub async fn eval_expression(expr: &Expression, env: &mut Environment) -> Result<RuntimeValue, EvalError> {
    match expr {
        Expression::Literal { value, .. } => match value {
            LiteralValue::Int(v) => Ok(RuntimeValue::Int(*v)),
            LiteralValue::Bool(v) => Ok(RuntimeValue::Bool(*v)),
            LiteralValue::Float(v) => Ok(RuntimeValue::Float(f64::from_bits(*v))),
            LiteralValue::String(v) => Ok(RuntimeValue::String(v.clone())),
        },
        Expression::Identifier { name, .. } => env.get(name).ok_or_else(|| EvalError::UndefinedIdentifier(name.clone())),
        Expression::UnaryOp { op, operand, .. } => {
            let val = eval_expression(operand, env).await?;
            match op {
                UnaryOperator::Not => match val {
                    RuntimeValue::Bool(b) => Ok(RuntimeValue::Bool(!b)),
                    _ => Err(EvalError::TypeError { expected: "Bool".into(), got: val_to_string(&val) }),
                },
                UnaryOperator::Neg => match val {
                    RuntimeValue::Int(i) => Ok(RuntimeValue::Int(-i)),
                    RuntimeValue::Float(f) => Ok(RuntimeValue::Float(-f)),
                    _ => Err(EvalError::TypeError { expected: "Number".into(), got: val_to_string(&val) }),
                },
            }
        }
        Expression::Lambda { .. } => Err(EvalError::Unknown("Lambda not supported in AST interpreter yet".into())),
        Expression::Await { future, .. } => {
            let fut_val = eval_expression(future, env).await?;
            if let RuntimeValue::Future(f) = fut_val {
                let mut guard = f.0.lock().await;
                match guard.as_mut().await {
                    Ok(v) => Ok(v),
                    Err(EvalError::Return(v)) => Ok(v),
                    Err(e) => Err(e),
                }
            }
            else {
                Err(EvalError::TypeError { expected: "Future".into(), got: val_to_string(&fut_val) })
            }
        }
        Expression::BinaryOp { left, op, right, .. } => {
            if let BinaryOperator::Assign = op {
                let r_val = eval_expression(right, env).await?;
                match &**left {
                    Expression::Identifier { name, .. } => {
                        if env.get(name).is_some() {
                            env.define(name.clone(), r_val.clone());
                            return Ok(r_val);
                        }
                        else {
                            return Err(EvalError::UndefinedIdentifier(name.clone()));
                        }
                    }
                    Expression::Get { object, name, .. } => {
                        let obj = eval_expression(object, env).await?;
                        if let RuntimeValue::Object(data) = obj {
                            data.lock().unwrap().fields.insert(name.clone(), r_val.clone());
                            return Ok(r_val);
                        }
                        else if let RuntimeValue::Class { .. } = obj {
                            // Static field assignment?
                            // Currently Class structure doesn't support mutable static fields in this runtime model easily
                            // unless we wrap Class in Mutex or use a RefCell.
                            // But RuntimeValue::Class is just an enum variant.
                            // Modifying it in place requires it to be a reference or inside a Mutex.
                            // For now, let's say static fields are not mutable or not supported yet.
                            return Err(EvalError::TypeError {
                                expected: "Object".into(),
                                got: "Class (Static fields not supported)".into(),
                            });
                        }
                        else {
                            eprintln!("DEBUG: Assigning to non-object. Obj: {:?}, Field: {}", obj, name);
                            return Err(EvalError::TypeError {
                                expected: "Object".into(),
                                got: format!("{:?} (assigning field '{}')", obj, name),
                            });
                        }
                    }
                    _ => return Err(EvalError::TypeError { expected: "lvalue".into(), got: "rvalue".into() }),
                }
            }

            if let BinaryOperator::And = op {
                 let l = eval_expression(left, env).await?;
                 if let RuntimeValue::Bool(b) = l {
                     if !b { return Ok(RuntimeValue::Bool(false)); }
                     let r = eval_expression(right, env).await?;
                     if let RuntimeValue::Bool(rb) = r {
                         return Ok(RuntimeValue::Bool(rb));
                     } else {
                         return Err(EvalError::TypeError { expected: "Bool".into(), got: val_to_string(&r) });
                     }
                 } else {
                     return Err(EvalError::TypeError { expected: "Bool".into(), got: val_to_string(&l) });
                 }
            }
            if let BinaryOperator::Or = op {
                 let l = eval_expression(left, env).await?;
                 if let RuntimeValue::Bool(b) = l {
                     if b { return Ok(RuntimeValue::Bool(true)); }
                     let r = eval_expression(right, env).await?;
                     if let RuntimeValue::Bool(rb) = r {
                         return Ok(RuntimeValue::Bool(rb));
                     } else {
                         return Err(EvalError::TypeError { expected: "Bool".into(), got: val_to_string(&r) });
                     }
                 } else {
                     return Err(EvalError::TypeError { expected: "Bool".into(), got: val_to_string(&l) });
                 }
            }

            let l = eval_expression(left, env).await?;
            let r = eval_expression(right, env).await?;

            match (l, r) {
                (RuntimeValue::Int(a), RuntimeValue::Int(b)) => match op {
                    BinaryOperator::Add => Ok(RuntimeValue::Int(a + b)),
                    BinaryOperator::Sub => Ok(RuntimeValue::Int(a - b)),
                    BinaryOperator::Mul => Ok(RuntimeValue::Int(a * b)),
                    BinaryOperator::Div => Ok(RuntimeValue::Int(a / b)),
                    BinaryOperator::Rem => Ok(RuntimeValue::Int(a % b)),
                    BinaryOperator::Equal => Ok(RuntimeValue::Bool(a == b)),
                    BinaryOperator::NotEqual => Ok(RuntimeValue::Bool(a != b)),
                    BinaryOperator::Less => Ok(RuntimeValue::Bool(a < b)),
                    BinaryOperator::LessEqual => Ok(RuntimeValue::Bool(a <= b)),
                    BinaryOperator::Greater => Ok(RuntimeValue::Bool(a > b)),
                    BinaryOperator::GreaterEqual => Ok(RuntimeValue::Bool(a >= b)),
                    BinaryOperator::Assign => unreachable!(),
                    BinaryOperator::And | BinaryOperator::Or => unreachable!(),
                    BinaryOperator::BitAnd => Ok(RuntimeValue::Int(a & b)),
                    BinaryOperator::BitOr => Ok(RuntimeValue::Int(a | b)),
                },
                (RuntimeValue::Float(a), RuntimeValue::Float(b)) => match op {
                    BinaryOperator::Add => Ok(RuntimeValue::Float(a + b)),
                    BinaryOperator::Sub => Ok(RuntimeValue::Float(a - b)),
                    BinaryOperator::Mul => Ok(RuntimeValue::Float(a * b)),
                    BinaryOperator::Div => Ok(RuntimeValue::Float(a / b)),
                    BinaryOperator::Rem => Ok(RuntimeValue::Float(a % b)),
                    BinaryOperator::Equal => Ok(RuntimeValue::Bool(a == b)),
                    BinaryOperator::NotEqual => Ok(RuntimeValue::Bool(a != b)),
                    BinaryOperator::Less => Ok(RuntimeValue::Bool(a < b)),
                    BinaryOperator::LessEqual => Ok(RuntimeValue::Bool(a <= b)),
                    BinaryOperator::Greater => Ok(RuntimeValue::Bool(a > b)),
                    BinaryOperator::GreaterEqual => Ok(RuntimeValue::Bool(a >= b)),
                    BinaryOperator::Assign => unreachable!(),
                    BinaryOperator::And | BinaryOperator::Or => unreachable!(),
                    BinaryOperator::BitAnd | BinaryOperator::BitOr => Err(EvalError::TypeError { expected: "Int".into(), got: "Float".into() }),
                },
                (RuntimeValue::Int(a), RuntimeValue::Float(b)) => eval_numeric_binary_op(a as f64, b, op),
                (RuntimeValue::Float(a), RuntimeValue::Int(b)) => eval_numeric_binary_op(a, b as f64, op),
                (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => match op {
                    BinaryOperator::Equal => Ok(RuntimeValue::Bool(a == b)),
                    BinaryOperator::NotEqual => Ok(RuntimeValue::Bool(a != b)),
                    BinaryOperator::And | BinaryOperator::Or => unreachable!(),
                    _ => Err(EvalError::TypeError { expected: "Bool".into(), got: format!("{:?}", op) }),
                },
                (RuntimeValue::String(a), RuntimeValue::String(b)) => match op {
                    BinaryOperator::Equal => Ok(RuntimeValue::Bool(a == b)),
                    BinaryOperator::NotEqual => Ok(RuntimeValue::Bool(a != b)),
                    BinaryOperator::And | BinaryOperator::Or => unreachable!(),
                    _ => Err(EvalError::TypeError { expected: "String".into(), got: format!("{:?}", op) }),
                },
                (l, r) => {
                    Err(EvalError::TypeError { expected: "compatible operands".into(), got: format!("{:?} and {:?}", l, r) })
                }
            }
        }
        Expression::Call { callee, args, .. } => {
            eprintln!("DEBUG: Call {:?}", callee);
            // Handle built-in print function
            if let Expression::Identifier { name, .. } = &**callee {
                if name == "print" {
                    for arg in args {
                        let val = eval_expression(arg, env).await?;
                        print!("{:?} ", val);
                    }
                    println!();
                    return Ok(RuntimeValue::Void);
                }
            }

            // Handle method call (Get -> Call) specially?
            // Or just eval callee.
            // If callee is Get(obj, name), we might need to bind 'self'.

            // Check if callee is a Get expression
            if let crate::ast::Expression::Get { object, name, .. } = &**callee {
                let obj = eval_expression(object, env).await?;

                if let RuntimeValue::Generator(stream) = &obj {
                    if name == "next" {
                        if !args.is_empty() {
                            return Err(EvalError::ArgumentCountMismatch { expected: 0, got: args.len() });
                        }
                        let mut rx = stream.0.lock().await;
                        match rx.recv().await {
                            Some(v) => return Ok(v),
                            None => return Ok(RuntimeValue::Void),
                        }
                    }
                }

                // Look up method on object
                eprintln!("DEBUG: Call method '{}' on {:?}", name, obj);
                if let RuntimeValue::Object(data) = &obj {
                    let class_name = data.lock().unwrap().class.clone();

                    // Method Resolution with Inheritance (MRO)
                    let class_val = env.get(&class_name).ok_or(EvalError::UndefinedIdentifier(class_name.clone()))?;
                    let mut found_method = None;

                    if let RuntimeValue::Class { mro, .. } = class_val {
                        for c_name in mro {
                            let c_val = env.get(&c_name).ok_or(EvalError::UndefinedIdentifier(c_name.clone()))?;
                            if let RuntimeValue::Class { methods, .. } = c_val {
                                if let Some(method) = methods.get(name) {
                                    found_method = Some(method.clone());
                                    break;
                                }
                            }
                        }
                    }
                    else {
                        return Err(EvalError::TypeError { expected: "Class".into(), got: format!("{:?}", class_val) });
                    }

                    if let Some(method) = found_method {
                        // Found method. Now call it.
                        let method_resolved = resolve_overload(method, args.len())
                            .ok_or_else(|| EvalError::ArgumentCountMismatch { expected: 0, got: args.len() })?;

                        if let RuntimeValue::Function { ref params, ref body, is_async, is_generator, .. } = method_resolved {
                            let mut arg_vals = Vec::new();
                            for arg in args {
                                arg_vals.push(eval_expression(arg, env).await?);
                            }

                            if is_generator {
                                let mut captured_env = env.clone();
                                captured_env.push_scope();

                                // Check for self
                                let mut param_iter = params.iter();
                                let mut injected_self = false;

                                if let Some(first_param) = param_iter.next() {
                                    if first_param == "self" {
                                        captured_env.define("self".to_string(), obj.clone());
                                        injected_self = true;

                                        if let RuntimeValue::Function { owner: Some(owner_class), .. } = &method_resolved {
                                            captured_env.define(
                                                "super".to_string(),
                                                RuntimeValue::Super { obj: data.clone(), start_class: owner_class.clone() },
                                            );
                                        }
                                    }
                                }

                                let params_to_bind = if injected_self { &params[1..] } else { &params[..] };

                                if params_to_bind.len() != arg_vals.len() {
                                    return Err(EvalError::ArgumentCountMismatch {
                                        expected: params_to_bind.len(),
                                        got: arg_vals.len(),
                                    });
                                }

                                for (p, v) in params_to_bind.iter().zip(arg_vals.iter()) {
                                    captured_env.define(p.clone(), v.clone());
                                }

                                let (tx, rx) = tokio::sync::mpsc::channel(32);
                                captured_env.define("$yield".to_string(), RuntimeValue::YieldSender(tx));

                                let body_clone = body.clone();
                                tokio::spawn(async move {
                                    let _ = eval_statement(&body_clone, &mut captured_env).await;
                                });
                                return Ok(RuntimeValue::Generator(GeneratorStream(Arc::new(tokio::sync::Mutex::new(rx)))));
                            }

                            if is_async {
                                let mut captured_env = env.clone();
                                captured_env.push_scope();

                                // Check for self
                                let mut param_iter = params.iter();
                                let mut injected_self = false;

                                if let Some(first_param) = param_iter.next() {
                                    if first_param == "self" {
                                        captured_env.define("self".to_string(), obj.clone());
                                        injected_self = true;

                                        if let RuntimeValue::Function { owner: Some(owner_class), .. } = &method_resolved {
                                            captured_env.define(
                                                "super".to_string(),
                                                RuntimeValue::Super { obj: data.clone(), start_class: owner_class.clone() },
                                            );
                                        }
                                    }
                                }

                                let params_to_bind = if injected_self { &params[1..] } else { &params[..] };

                                if params_to_bind.len() != arg_vals.len() {
                                    return Err(EvalError::ArgumentCountMismatch {
                                        expected: params_to_bind.len(),
                                        got: arg_vals.len(),
                                    });
                                }

                                for (p, v) in params_to_bind.iter().zip(arg_vals.iter()) {
                                    captured_env.define(p.clone(), v.clone());
                                }

                                let body_clone = body.clone();
                                let future = Box::pin(async move { eval_statement(&body_clone, &mut captured_env).await });
                                return Ok(RuntimeValue::Future(AsyncFuture(Arc::new(tokio::sync::Mutex::new(future)))));
                            }

                            env.push_scope();
                            // Check for self
                            let mut param_iter = params.iter();
                            let mut injected_self = false;

                            if let Some(first_param) = param_iter.next() {
                                if first_param == "self" {
                                    env.define("self".to_string(), obj.clone());
                                    injected_self = true;

                                    // Inject 'super'
                                    // 'super' depends on the owner class of the method
                                    if let RuntimeValue::Function { owner: Some(owner_class), .. } = &method_resolved {
                                        env.define(
                                            "super".to_string(),
                                            RuntimeValue::Super { obj: data.clone(), start_class: owner_class.clone() },
                                        );
                                    }
                                }
                            }

                            // If method expects self but we called it on object, we injected it.
                            // If method DOES NOT expect self (static method called on instance), we should probably allow it but NOT inject self.
                            // But `param_iter` already skipped "self" if it was there.
                            // If it wasn't there, `param_iter` consumed the first param which is a normal arg.
                            // Wait, `next()` consumes.

                            // Reset iter logic
                            let params_to_bind = if injected_self { &params[1..] } else { &params[..] };

                            if params_to_bind.len() != arg_vals.len() {
                                return Err(EvalError::ArgumentCountMismatch {
                                    expected: params_to_bind.len(),
                                    got: arg_vals.len(),
                                });
                            }

                            for (p, v) in params_to_bind.iter().zip(arg_vals.iter()) {
                                env.define(p.clone(), v.clone());
                            }

                            let result = eval_statement(&body, env).await;
                            env.pop_scope();
                            return match result {
                                Ok(v) => Ok(v),
                                Err(EvalError::Return(v)) => Ok(v),
                                Err(e) => Err(e),
                            };
                        }
                    }
                }
                else if let RuntimeValue::Class { methods, mro, .. } = &obj {
                    // Static method call on Class
                    // Look up method in Class (and parents?)
                    // Static methods should also follow MRO? Usually yes.

                    let mut found_method = None;
                    for c_name in mro {
                        let c_val = env.get(&c_name).ok_or(EvalError::UndefinedIdentifier(c_name.clone()))?;
                        if let RuntimeValue::Class { methods: c_methods, .. } = c_val {
                            if let Some(method) = c_methods.get(name) {
                                found_method = Some(method.clone());
                                break;
                            }
                        }
                    }

                    if let Some(method) = found_method {
                        let method_resolved = resolve_overload(method, args.len())
                            .ok_or_else(|| EvalError::ArgumentCountMismatch { expected: 0, got: args.len() })?;

                        if let RuntimeValue::Function { ref params, ref body, .. } = method_resolved {
                            // Check if it is an instance method (has 'self')
                            if !params.is_empty() && params[0] == "self" {
                                return Err(EvalError::TypeError {
                                    expected: "Static Method".into(),
                                    got: "Instance Method (needs self)".into(),
                                });
                            }

                            let mut arg_vals = Vec::new();
                            for arg in args {
                                arg_vals.push(eval_expression(arg, env).await?);
                            }

                            if params.len() != arg_vals.len() {
                                return Err(EvalError::ArgumentCountMismatch { expected: params.len(), got: arg_vals.len() });
                            }

                            env.push_scope();
                            for (p, v) in params.iter().zip(arg_vals.iter()) {
                                env.define(p.clone(), v.clone());
                            }

                            let result = eval_statement(&body, env).await;
                            env.pop_scope();
                            return match result {
                                Ok(v) => Ok(v),
                                Err(EvalError::Return(v)) => Ok(v),
                                Err(e) => Err(e),
                            };
                        }
                    }
                }
                else if let RuntimeValue::String(s) = &obj {
                    if name == "len" {
                        return Ok(RuntimeValue::Int(s.len() as i64));
                    }
                    else if name == "chars" {
                        let chars: Vec<RuntimeValue> = s.chars().map(|c| RuntimeValue::String(c.to_string())).collect();
                        return Ok(RuntimeValue::List(Arc::new(Mutex::new(chars))));
                    }
                }
                else if let RuntimeValue::List(l) = &obj {
                    if name == "len" {
                        let len = l.lock().unwrap().len();
                        return Ok(RuntimeValue::Int(len as i64));
                    }
                }
            }

            let func = eval_expression(callee, env).await?;
            let mut arg_vals = Vec::new();
            for arg in args {
                arg_vals.push(eval_expression(arg, env).await?);
            }

            match func {
                RuntimeValue::Function { params, body, is_async, is_generator, .. } => {
                    if params.len() != arg_vals.len() {
                        return Err(EvalError::ArgumentCountMismatch { expected: params.len(), got: arg_vals.len() });
                    }

                    if is_generator {
                        let mut captured_env = env.clone();
                        captured_env.push_scope();
                        for (param, val) in params.iter().zip(arg_vals) {
                            captured_env.define(param.clone(), val);
                        }

                        let (tx, rx) = tokio::sync::mpsc::channel(32);
                        captured_env.define("$yield".to_string(), RuntimeValue::YieldSender(tx));

                        let body_clone = body.clone();
                        tokio::spawn(async move {
                            let _ = eval_statement(&body_clone, &mut captured_env).await;
                        });

                        return Ok(RuntimeValue::Generator(GeneratorStream(Arc::new(tokio::sync::Mutex::new(rx)))));
                    }

                    if is_async {
                        let mut captured_env = env.clone();
                        captured_env.push_scope();
                        for (param, val) in params.iter().zip(arg_vals) {
                            captured_env.define(param.clone(), val);
                        }
                        let body_clone = body.clone();
                        let future = Box::pin(async move { eval_statement(&body_clone, &mut captured_env).await });
                        return Ok(RuntimeValue::Future(AsyncFuture(Arc::new(tokio::sync::Mutex::new(future)))));
                    }

                    env.push_scope();
                    for (param, val) in params.iter().zip(arg_vals) {
                        env.define(param.clone(), val);
                    }

                    let result = eval_statement(&body, env).await;
                    env.pop_scope();

                    match result {
                        Ok(v) => Ok(v),
                        Err(EvalError::Return(v)) => Ok(v),
                        Err(e) => Err(e),
                    }
                }
                _ => Err(EvalError::NotAFunction),
            }
        }
        Expression::New { class, args, closure, .. } => {
            // 1. Resolve Class
            let mut current = class;
            while let crate::ast::TypeExpression::Generic { base, .. } = current {
                current = base;
            }
            let class_name = if let crate::ast::TypeExpression::Name { name, .. } = current {
                name.clone()
            }
            else {
                return Err(EvalError::TypeError { expected: "ClassName".into(), got: "ComplexType".into() });
            };

            let class_val = env.get(&class_name).ok_or_else(|| EvalError::UndefinedIdentifier(class_name.clone()))?;

            // 2. Create Object with inherited fields
            // Collect fields from hierarchy using MRO
            let mut all_fields = HashMap::new();
            let mut ctor = None;

            if let RuntimeValue::Class { mro, .. } = &class_val {
                // Traverse MRO (usually Child -> Parents)
                for c_name in mro {
                    let c_val = env.get(c_name).ok_or_else(|| EvalError::UndefinedIdentifier(c_name.clone()))?;
                    if let RuntimeValue::Class { fields, methods, parents, .. } = c_val {
                        // Add fields
                        for (fname, _) in fields {
                            if !all_fields.contains_key(&fname) {
                                all_fields.insert(fname, RuntimeValue::Void);
                            }
                        }

                        // Initialize named super pointers
                        // If class has named parents, we need to create objects for them?
                        // "Multiple classes must use super.class_name variable and super.c variable"
                        // This implies we need to store something in the object to allow `super.c`.
                        // But `super` is not an object, it's a keyword.
                        // Wait, if `super` is a keyword, then `super.c` is property access on super?
                        // If `c` is a named base, maybe we should store the base instance in a field named `c`?
                        // If so, we need to instantiate it. But base is a Class, not an Object.
                        // Inheritance implies IS-A, so `self` IS `c`.
                        // But if we want to access `c` explicitly, maybe we just alias `self`?
                        // "super.c variable" suggests `c` is a field in `super` (or `self`)?
                        // Let's assume `c: C` in `class A(c: C)` means `A` has a field `c` which effectively holds the `C` part.
                        // But since we flatten fields, `self` holds `C`'s fields directly.
                        // So `self.c` might be ambiguous if `c` is a field name or a parent name.
                        // If `c` is a parent name, `self.c` could return `self` casted to `C`?
                        // Or `super.c` returns `self` but with scope adjusted to `C`?
                        // For now, let's implement `super` as a special handling in `eval_expression`.
                        // But we are in `New`.

                        // Look for constructor (new)
                        if ctor.is_none() {
                            if let Some(m) = methods.get("new") {
                                // Check overloading for 'new'
                                // We need to pass args to resolve_overload, but args are available here.
                                // So we just store the potentially overloaded method and resolve later.
                                ctor = Some(m.clone());
                            }
                        }
                    }
                }
            }
            else {
                return Err(EvalError::TypeError { expected: "Class".into(), got: format!("{:?}", class_val) });
            }

            let obj = RuntimeValue::Object(Arc::new(Mutex::new(ObjectData { class: class_name.clone(), fields: all_fields })));

            // 3. Call Constructor (new) if exists
            if let Some(ctor_func) = ctor {
                let ctor_resolved = resolve_overload(ctor_func, args.len())
                    .ok_or_else(|| EvalError::ArgumentCountMismatch { expected: 0, got: args.len() })?;

                if let RuntimeValue::Function { params, body, .. } = ctor_resolved {
                    let mut arg_vals = Vec::new();
                    for arg in args {
                        arg_vals.push(eval_expression(arg, env).await?);
                    }

                    env.push_scope();
                    // Pass arguments
                    if params.len() == arg_vals.len() {
                        for (p, v) in params.iter().zip(arg_vals) {
                            env.define(p.clone(), v);
                        }
                    }
                    else {
                        // Handle implicit 'self' injection if first param is 'self'
                        if !params.is_empty() && params[0] == "self" {
                            env.define("self".to_string(), obj.clone());
                            if params.len() - 1 == arg_vals.len() {
                                for (p, v) in params.iter().skip(1).zip(arg_vals.iter()) {
                                    env.define(p.clone(), v.clone());
                                }
                            }
                            else {
                                return Err(EvalError::ArgumentCountMismatch {
                                    expected: params.len() - 1,
                                    got: arg_vals.len(),
                                });
                            }
                        }
                        else {
                            return Err(EvalError::ArgumentCountMismatch { expected: params.len(), got: arg_vals.len() });
                        }
                    }

                    // Just evaluate body for now.
                    let _ = eval_statement(&body, env).await;
                    env.pop_scope();
                }
            }

            // 4. Run Trailing Closure if exists
            if let Some(c) = closure {
                // Closure likely takes `self` or captures it?
                // Or just runs in a scope where `self` is defined?
                // Or maybe it's `new Class() { |obj| ... }`?
                // User said: `new namepath::ClassName() { 可选尾随闭包，后处理过程 };`
                // "post-processing process".
                // Probably `it` or `self` is available?
                // Let's bind `it` to the object.
                env.push_scope();
                env.define("it".to_string(), obj.clone()); // Kotlin style
                let _ = eval_statement(c, env).await;
                env.pop_scope();
            }

            Ok(obj)
            // } else {
            //     Err(EvalError::TypeError { expected: "Class".into(), got: format!("{:?}", class_val) })
            // }
        }
        Expression::Lambda { params, body, is_async, is_generator, .. } => Ok(RuntimeValue::Function {
            name: "lambda".to_string(),
            params: params.iter().map(|(n, _)| n.clone()).collect(),
            body: body.clone(),
            owner: None,
            is_async: *is_async,
            is_generator: *is_generator,
        }),
        Expression::Match { scrutinee, arms, else_arm, .. } => {
            let v = eval_expression(scrutinee, env).await?;

            for (pat, arm_expr) in arms {
                if match_pattern(&v, pat) {
                    return eval_expression(arm_expr, env).await;
                }
            }

            if let Some(e) = else_arm {
                eval_expression(e, env).await
            }
            else {
                Ok(RuntimeValue::Void)
            }
        }
        Expression::List { elements, .. } => {
            let mut vals = Vec::new();
            for e in elements {
                vals.push(eval_expression(e, env).await?);
            }
            Ok(RuntimeValue::List(std::sync::Arc::new(std::sync::Mutex::new(vals))))
        }
        Expression::Index { target, index, is_zero_based, .. } => {
            let t_val = eval_expression(target, env).await?;
            let i_val = eval_expression(index, env).await?;

            let idx = match i_val {
                RuntimeValue::Int(i) => i,
                _ => return Err(EvalError::TypeError { expected: "Int".into(), got: format!("{:?}", i_val) }),
            };

            let final_idx = if *is_zero_based { idx } else { idx - 1 };
            if final_idx < 0 {
                return Err(EvalError::Unknown(format!("Index out of bounds: {}", final_idx)));
            }

            match t_val {
                RuntimeValue::List(list_arc) => {
                    let elements = list_arc.lock().unwrap();
                    if (final_idx as usize) < elements.len() {
                        Ok(elements[final_idx as usize].clone())
                    }
                    else {
                        Err(EvalError::Unknown(format!("Index out of bounds: {} >= {}", final_idx, elements.len())))
                    }
                }
                _ => Err(EvalError::TypeError { expected: "List".into(), got: format!("{:?}", t_val) }),
            }
        }
        Expression::Block { body, .. } => eval_statement(body, env).await,
        Expression::MacroCall { name, args, .. } => {
            // Built-in macros for interpreter
            if name == "println" || name == "print" {
                for arg in args {
                    let val = eval_expression(arg, env).await?;
                    print!("{}", val_to_string(&val));
                }
                println!();
                Ok(RuntimeValue::Void)
            }
            else if name == "dbg" || name == "debug_log" {
                for arg in args {
                    let val = eval_expression(arg, env).await?;
                    println!("[DEBUG] {:?}", val);
                }
                Ok(RuntimeValue::Void)
            }
            else if name == "stringify" {
                if args.is_empty() {
                    return Ok(RuntimeValue::String("".to_string()));
                }
                // Return string representation of the first argument AST
                // Note: We do NOT evaluate the argument
                let s = format!("{:?}", args[0]);
                Ok(RuntimeValue::String(s))
            }
            else if let Some(macro_val) = env.get(name) {
                // User-defined macro execution (simulated as runtime function)
                if let RuntimeValue::Macro { params, body, .. } = macro_val {
                    let mut arg_vals = Vec::new();
                    // For user macros, pass ASTs instead of evaluating
                    // This aligns with "macros are functions of AST nodes"
                    for arg in args {
                        arg_vals.push(RuntimeValue::Ast(Box::new(arg.clone())));
                    }

                    if params.len() != arg_vals.len() {
                        return Err(EvalError::ArgumentCountMismatch { expected: params.len(), got: arg_vals.len() });
                    }

                    env.push_scope();
                    for (param, val) in params.iter().zip(arg_vals) {
                        env.define(param.clone(), val);
                    }

                    let result = eval_statement(&body, env).await;
                    env.pop_scope();

                    match result {
                        Ok(v) => Ok(v),
                        Err(EvalError::Return(v)) => Ok(v),
                        Err(e) => Err(e),
                    }
                }
                else {
                    Err(EvalError::UndefinedIdentifier(format!("macro @{}", name)))
                }
            }
            else {
                Err(EvalError::UndefinedIdentifier(format!("macro @{}", name)))
            }
        }
        Expression::Get { object, name, .. } => {
            let obj = eval_expression(object, env).await?;
            if name == "starts_with" {
                 println!("DEBUG: Get {} on {:?}", name, obj);
            }
            match obj {
                RuntimeValue::Super { obj, start_class } => {
                    // super.field or super.method()
                    // 1. Check if name is a parent of start_class (for manual resolution super.ParentName)
                    // 2. Check if name is a field in obj (flattened)
                    // 3. If name is method, resolve in start_class's parents (MRO after start_class)

                    // User requirement: super.class_name variable and super.c variable
                    // This implies name could be a class name or alias.
                    // If name matches a parent of start_class:
                    // Return a new Super proxy for THAT parent.

                    let mut parent_found = None;
                    if let Ok(current_class_val) =
                        env.get(&start_class).ok_or(EvalError::UndefinedIdentifier(start_class.clone()))
                    {
                        if let RuntimeValue::Class { parents, .. } = current_class_val {
                            for (alias, p_name) in parents {
                                if p_name == *name || alias.as_ref() == Some(name) {
                                    parent_found = Some(p_name.clone());
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(p_name) = parent_found {
                        return Ok(RuntimeValue::Super { obj: obj.clone(), start_class: p_name });
                    }

                    // If not a parent name, check for field or method in SUPER context.
                    // Method lookup should start AFTER start_class in MRO.
                    // But wait, `super.method()` usually means calling method on super.
                    // If `super` is `Super { start_class: Child }`, then `super.method()` should look in Parents of Child.

                    // Check method in MRO
                    let class_val = env.get(&start_class).ok_or(EvalError::UndefinedIdentifier(start_class.clone()))?;
                    if let RuntimeValue::Class { mro, .. } = class_val {
                        // Find start_class in MRO, start looking after it?
                        // No, mro includes start_class at head.
                        // But we want to call PARENT method.
                        // Actually, if we have multiple inheritance, `super` is ambiguous without qualification?
                        // Or `super` follows MRO order.
                        // If `start_class` is Child, we skip Child in MRO.

                        let mut found_method = None;
                        let mut skip = true; // Skip until we pass start_class
                                             // Wait, `mro` is linearized list of ANCESTORS of `start_class`?
                                             // No, `mro` in `Class` is the MRO of that class.
                                             // So `mro[0]` is `start_class`.
                                             // We want to search `mro[1..]`.

                        for c_name in mro.iter().skip(1) {
                            let c_val = env.get(c_name).ok_or(EvalError::UndefinedIdentifier(c_name.clone()))?;
                            if let RuntimeValue::Class { methods, .. } = c_val {
                                if let Some(method) = methods.get(name) {
                                    found_method = Some(method.clone());
                                    break;
                                }
                            }
                        }

                        if let Some(method) = found_method {
                            // When calling this method, `self` is still `obj`.
                            // But what if this method calls `super`?
                            // It will use its own `owner` which is the class where it's defined.
                            // So `super` chaining works.
                            return Ok(method);
                        }
                    }

                    // If not method, check field in obj
                    let fields = &obj.lock().unwrap().fields;
                    if let Some(val) = fields.get(name) {
                        return Ok(val.clone());
                    }

                    Err(EvalError::UndefinedIdentifier(format!("super member {}", name)))
                }
                RuntimeValue::Object(data) => {
                    let fields = &data.lock().unwrap().fields;
                    if let Some(val) = fields.get(name) {
                        return Ok(val.clone());
                    }
                    Err(EvalError::UndefinedIdentifier(name.clone()))
                }
                RuntimeValue::String(s) => {
                    if name == "len" {
                        Ok(RuntimeValue::Int(s.len() as i64))
                    }
                    else if name == "chars" {
                        let chars: Vec<RuntimeValue> = s.chars().map(|c| RuntimeValue::String(c.to_string())).collect();
                        Ok(RuntimeValue::List(std::sync::Arc::new(std::sync::Mutex::new(chars))))
                    }
                    else if name == "starts_with" {
                        let s_clone = s.clone();
                        Ok(RuntimeValue::NativeFunction {
                            name: "String.starts_with".to_string(), 
                            func: crate::mir::value::NativeFunctionWrapper(std::sync::Arc::new(move |args| {
                                if args.len() != 1 {
                                    return Err(EvalError::ArgumentCountMismatch { expected: 1, got: args.len() });
                                }
                                let prefix = match &args[0] {
                                    RuntimeValue::String(p) => p,
                                    _ => return Err(EvalError::TypeError { expected: "String".into(), got: "Other".into() }),
                                };
                                Ok(RuntimeValue::Bool(s_clone.starts_with(prefix)))
                            }))
                        })
                    }
                    else {
                        Err(EvalError::UndefinedIdentifier(format!("String member {}", name)))
                    }
                }
                RuntimeValue::List(l) => {
                    if name == "len" {
                        let len = l.lock().unwrap().len();
                        Ok(RuntimeValue::Int(len as i64))
                    }
                    else {
                        Err(EvalError::UndefinedIdentifier(format!("List member {}", name)))
                    }
                }
                _ => Err(EvalError::TypeError {
                    expected: "Object".into(),
                    got: format!("{:?} (accessing field '{}')", obj, name),
                }),
            }
        }
    }
}

// Helper to resolve super
// super.c.trait_method()
// If we have `super` object, `super.c` should return the parent part?
// But we flattened the object.
// Maybe `super` is a special object that has fields pointing to parent instances?
// Or we just rely on `super` being the object itself, and `c` being a field.
// If `class A(c: C)`, then `A` has a field `c` of type `C`.
// So `self.c` works. `super.c`? `super` usually refers to parent *class* methods.
// But the user said: `class A(ClassName, c: C)`.
// `ClassName` is an unnamed parent. `c` is a named parent.
// If `c` is a field, then `self.c` works.
// If the user wants `super.c`, maybe `super` in `A` exposes `c`?
// If `super` evaluates to `self` (but with different method resolution), then `super.c` is `self.c`.
// So standard field access should work if we store named parents as fields.

fn match_pattern(value: &RuntimeValue, pat: &MatchPattern) -> bool {
    match pat {
        MatchPattern::Wildcard => true,
        MatchPattern::Literal(lit) => match (value, lit) {
            (RuntimeValue::Int(v), LiteralValue::Int(p)) => v == p,
            (RuntimeValue::Bool(v), LiteralValue::Bool(p)) => v == p,
            (RuntimeValue::String(v), LiteralValue::String(p)) => v == p,
            (RuntimeValue::Float(v), LiteralValue::Float(p)) => v.to_bits() == *p,
            _ => false,
        },
    }
}

fn merge_mro(mut parent_mros: Vec<Vec<String>>, mut parent_names: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    loop {
        // Remove empty lists
        parent_mros.retain(|m| !m.is_empty());
        if parent_mros.is_empty() && parent_names.is_empty() {
            return result;
        }

        let mut candidate = None;

        // Try to find a candidate in the head of the lists
        // A candidate is valid if it is not in the tail of any other list
        for (i, mro_list) in parent_mros.iter().enumerate() {
            let head = &mro_list[0];
            let mut valid = true;
            for (j, other_list) in parent_mros.iter().enumerate() {
                if i != j {
                    // Check if head is in tail of other_list
                    if other_list.iter().skip(1).any(|x| x == head) {
                        valid = false;
                        break;
                    }
                }
            }
            if valid {
                candidate = Some(head.clone());
                break;
            }
        }

        // If no candidate from lists, try parents list itself?
        // Actually, parent_names are also part of the merge constraint in C3: merge(L(B1), ..., [B1, B2...])
        // So we should treat parent_names as another list to merge.
        // Let's add it to parent_mros at start.
        // But `parent_mros` is Vec<Vec<String>>.

        if candidate.is_none() {
            // Check if head of parent_names is valid?
            if !parent_names.is_empty() {
                let head = &parent_names[0];
                let mut valid = true;
                for mro_list in &parent_mros {
                    if mro_list.iter().skip(1).any(|x| x == head) {
                        valid = false;
                        break;
                    }
                }
                if valid {
                    candidate = Some(head.clone());
                }
            }
        }

        if let Some(c) = candidate {
            result.push(c.clone());
            // Remove c from heads of all lists
            for mro_list in parent_mros.iter_mut() {
                if !mro_list.is_empty() && mro_list[0] == c {
                    mro_list.remove(0);
                }
            }
            if !parent_names.is_empty() && parent_names[0] == c {
                parent_names.remove(0);
            }
        }
        else {
            // C3 linearization failed (inconsistent hierarchy)
            // For now, just panic or return what we have (best effort)
            // Or pick the first available head to break cycle?
            if !parent_mros.is_empty() {
                let head = parent_mros[0].remove(0);
                result.push(head);
            }
            else if !parent_names.is_empty() {
                let head = parent_names.remove(0);
                result.push(head);
            }
            else {
                break result;
            }
        }
    }
}

fn eval_numeric_binary_op(a: f64, b: f64, op: &BinaryOperator) -> Result<RuntimeValue, EvalError> {
    match op {
        BinaryOperator::Add => Ok(RuntimeValue::Float(a + b)),
        BinaryOperator::Sub => Ok(RuntimeValue::Float(a - b)),
        BinaryOperator::Mul => Ok(RuntimeValue::Float(a * b)),
        BinaryOperator::Div => Ok(RuntimeValue::Float(a / b)),
        BinaryOperator::Rem => Ok(RuntimeValue::Float(a % b)),
        BinaryOperator::Equal => Ok(RuntimeValue::Bool(a == b)),
        BinaryOperator::NotEqual => Ok(RuntimeValue::Bool(a != b)),
        BinaryOperator::Less => Ok(RuntimeValue::Bool(a < b)),
        BinaryOperator::LessEqual => Ok(RuntimeValue::Bool(a <= b)),
        BinaryOperator::Greater => Ok(RuntimeValue::Bool(a > b)),
        BinaryOperator::GreaterEqual => Ok(RuntimeValue::Bool(a >= b)),
        BinaryOperator::Assign => Err(EvalError::TypeError { expected: "Operator".into(), got: "Assign".into() }),
        BinaryOperator::And | BinaryOperator::Or => unreachable!(),
        BinaryOperator::BitAnd | BinaryOperator::BitOr => Err(EvalError::TypeError { expected: "Int".into(), got: "Float".into() }),
    }
}
