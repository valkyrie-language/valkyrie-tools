pub mod lowering;
pub mod optimizer;

use crate::ast::{BinaryOperator, LiteralValue, MatchPattern, Span, UnaryOperator};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HirBindingId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirBindingKind {
    Let,
    Param,
    Function,
    Class,
    Trait,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirBinding {
    pub name: String,
    pub kind: HirBindingKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirBindingTable {
    pub bindings: Vec<HirBinding>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirProgram {
    pub statements: Vec<HirStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStatement {
    Namespace {
        path: Vec<String>,
        span: Span,
    },
    Using {
        path: Vec<String>,
        span: Span,
    },
    Block {
        statements: Vec<HirStatement>,
        span: Span,
    },
    Let {
        name: String,
        binding: Option<HirBindingId>,
        ty: HirType,
        value: HirExpression,
        span: Span,
    },
    If {
        condition: HirExpression,
        then_branch: Box<HirStatement>,
        else_branch: Option<Box<HirStatement>>,
        span: Span,
    },
    Return {
        value: Option<HirExpression>,
        span: Span,
    },
    While {
        condition: HirExpression,
        body: Box<HirStatement>,
        span: Span,
    },
    Class {
        name: String,
        binding: Option<HirBindingId>,
        parents: Vec<(Option<String>, HirType)>,
        traits: Vec<HirType>,
        fields: Vec<(String, HirType)>,
        methods: Vec<HirStatement>,
        span: Span,
    },
    Trait {
        name: String,
        binding: Option<HirBindingId>,
        parents: Vec<HirType>,
        methods: Vec<HirStatement>,
        span: Span,
    },
    Imply {
        target: HirType,
        trait_target: Option<HirType>,
        methods: Vec<HirStatement>,
        span: Span,
    },
    Function {
        name: String,
        binding: Option<HirBindingId>,
        params: Vec<HirParam>,
        return_type: HirType,
        body: Option<Box<HirStatement>>,
        span: Span,
    },
    Expression {
        expression: HirExpression,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirExpression {
    Identifier {
        name: String,
        binding: Option<HirBindingId>,
        ty: HirType,
        span: Span,
    },
    Literal {
        value: LiteralValue,
        ty: HirType,
        span: Span,
    },
    BinaryOp {
        left: Box<HirExpression>,
        op: BinaryOperator,
        right: Box<HirExpression>,
        ty: HirType,
        span: Span,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<HirExpression>,
        ty: HirType,
        span: Span,
    },
    Call {
        callee: Box<HirExpression>,
        args: Vec<HirExpression>,
        ty: HirType,
        span: Span,
    },
    Get {
        object: Box<HirExpression>,
        name: String,
        ty: HirType,
        span: Span,
    },
    New {
        class: HirType,
        args: Vec<HirExpression>,
        closure: Option<Box<HirStatement>>,
        ty: HirType,
        span: Span,
    },
    Match {
        scrutinee: Box<HirExpression>,
        arms: Vec<(MatchPattern, HirExpression)>,
        else_arm: Option<Box<HirExpression>>,
        ty: HirType,
        span: Span,
    },
    Lambda {
        params: Vec<HirParam>,
        body: Box<HirStatement>,
        ty: HirType,
        span: Span,
    },
    List {
        elements: Vec<HirExpression>,
        ty: HirType,
        span: Span,
    },
    Index {
        target: Box<HirExpression>,
        index: Box<HirExpression>,
        is_zero_based: bool,
        ty: HirType,
        span: Span,
    },
    Block {
        body: Box<HirStatement>,
        ty: HirType,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirParam {
    pub name: String,
    pub binding: Option<HirBindingId>,
    pub ty: HirType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirType {
    Unknown,
    Int,
    Float,
    Bool,
    String,
    Void,
    Function { params: Vec<HirType>, ret: Box<HirType> },
    Class(String),
    List(Box<HirType>),
}

pub fn resolve_symbols(program: &mut HirProgram) -> HirBindingTable {
    struct Resolver {
        scopes: Vec<HashMap<String, HirBindingId>>,
        bindings: Vec<HirBinding>,
        namespaces: Vec<Vec<String>>,
        usings: Vec<HashMap<String, String>>,
    }

    impl Resolver {
        fn new() -> Self {
            Self {
                scopes: vec![HashMap::new()],
                bindings: Vec::new(),
                namespaces: vec![Vec::new()],
                usings: vec![HashMap::new()],
            }
        }

        fn push_scope(&mut self) {
            self.scopes.push(HashMap::new());
            self.namespaces.push(self.namespaces.last().cloned().unwrap_or_default());
            self.usings.push(self.usings.last().cloned().unwrap_or_default());
        }

        fn pop_scope(&mut self) {
            self.scopes.pop();
            self.namespaces.pop();
            self.usings.pop();
        }

        fn current_namespace(&self) -> &[String] {
            self.namespaces.last().map(|v| v.as_slice()).unwrap_or(&[])
        }

        fn set_namespace(&mut self, path: Vec<String>) {
            if let Some(ns) = self.namespaces.last_mut() {
                *ns = path;
            }
        }

        fn add_using(&mut self, path: Vec<String>) {
            let Some(alias) = path.last().cloned()
            else {
                return;
            };
            let full = path.join("::");
            if let Some(m) = self.usings.last_mut() {
                m.insert(alias, full);
            }
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

        fn resolve_name(&self, name: &str) -> Option<HirBindingId> {
            if let Some(id) = self.lookup(name) {
                return Some(id);
            }
            if name.contains("::") {
                return None;
            }
            if let Some(q) = self.qualify_in_namespace(name) {
                if let Some(id) = self.lookup(&q) {
                    return Some(id);
                }
            }
            if let Some(m) = self.usings.last() {
                if let Some(full) = m.get(name) {
                    if let Some(id) = self.lookup(full) {
                        return Some(id);
                    }
                }
            }
            None
        }

        fn define(&mut self, name: String, kind: HirBindingKind, span: Span) -> HirBindingId {
            let id = HirBindingId(self.bindings.len() as u32);
            self.bindings.push(HirBinding { name: name.clone(), kind, span });
            if let Some(scope) = self.scopes.last_mut() {
                scope.insert(name, id);
            }
            id
        }

        fn lookup(&self, name: &str) -> Option<HirBindingId> {
            for scope in self.scopes.iter().rev() {
                if let Some(id) = scope.get(name) {
                    return Some(*id);
                }
            }
            None
        }

        fn resolve_program(&mut self, program: &mut HirProgram) {
            for stmt in &mut program.statements {
                self.resolve_statement(stmt);
            }
        }

        fn resolve_statement(&mut self, stmt: &mut HirStatement) {
            match stmt {
                HirStatement::Namespace { path, .. } => {
                    self.set_namespace(path.clone());
                }
                HirStatement::Using { path, .. } => {
                    self.add_using(path.clone());
                }
                HirStatement::Block { statements, .. } => {
                    self.push_scope();
                    for s in statements {
                        self.resolve_statement(s);
                    }
                    self.pop_scope();
                }
                HirStatement::Let { name, binding, value, span, .. } => {
                    self.resolve_expression(value);
                    let qualified = if self.scopes.len() == 1 {
                        self.qualify_in_namespace(name).unwrap_or_else(|| name.clone())
                    }
                    else {
                        name.clone()
                    };
                    let id = self.define(qualified, HirBindingKind::Let, *span);
                    *binding = Some(id);
                }
                HirStatement::If { condition, then_branch, else_branch, .. } => {
                    self.resolve_expression(condition);
                    self.resolve_statement(then_branch);
                    if let Some(else_b) = else_branch {
                        self.resolve_statement(else_b);
                    }
                }
                HirStatement::While { condition, body, .. } => {
                    self.resolve_expression(condition);
                    self.resolve_statement(body);
                }
                HirStatement::Return { value, .. } => {
                    if let Some(v) = value {
                        self.resolve_expression(v);
                    }
                }
                HirStatement::Class { name, binding, methods, span, .. } => {
                    let qualified = if self.scopes.len() == 1 {
                        self.qualify_in_namespace(name).unwrap_or_else(|| name.clone())
                    }
                    else {
                        name.clone()
                    };
                    let id = self.define(qualified, HirBindingKind::Class, *span);
                    *binding = Some(id);

                    self.push_scope(); // Class scope
                    for m in methods {
                        self.resolve_statement(m);
                    }
                    self.pop_scope();
                }
                HirStatement::Trait { name, binding, methods, span, .. } => {
                    let qualified = if self.scopes.len() == 1 {
                        self.qualify_in_namespace(name).unwrap_or_else(|| name.clone())
                    }
                    else {
                        name.clone()
                    };
                    let id = self.define(qualified, HirBindingKind::Trait, *span);
                    *binding = Some(id);

                    self.push_scope(); // Trait scope
                    for m in methods {
                        self.resolve_statement(m);
                    }
                    self.pop_scope();
                }
                HirStatement::Imply { methods, .. } => {
                    self.push_scope(); // Imply scope
                    for m in methods {
                        self.resolve_statement(m);
                    }
                    self.pop_scope();
                }
                HirStatement::Function { name, binding, params, body, span, .. } => {
                    let qualified = if self.scopes.len() == 1 {
                        self.qualify_in_namespace(name).unwrap_or_else(|| name.clone())
                    }
                    else {
                        name.clone()
                    };
                    let func_id = self.define(qualified, HirBindingKind::Function, *span);
                    *binding = Some(func_id);

                    self.push_scope();
                    for p in params {
                        let id = self.define(p.name.clone(), HirBindingKind::Param, *span);
                        p.binding = Some(id);
                    }
                    if let Some(b) = body {
                        self.resolve_statement(b);
                    }
                    self.pop_scope();
                }
                HirStatement::Expression { expression, .. } => {
                    self.resolve_expression(expression);
                }
            }
        }

        fn resolve_expression(&mut self, expr: &mut HirExpression) {
            match expr {
                HirExpression::Identifier { name, binding, .. } => {
                    *binding = self.resolve_name(name);
                }
                HirExpression::Literal { .. } => {}
                HirExpression::BinaryOp { left, right, .. } => {
                    self.resolve_expression(left);
                    self.resolve_expression(right);
                }
                HirExpression::Call { callee, args, .. } => {
                    self.resolve_expression(callee);
                    for a in args {
                        self.resolve_expression(a);
                    }
                }
                HirExpression::Get { object, .. } => {
                    self.resolve_expression(object);
                }
                HirExpression::New { args, closure, .. } => {
                    for a in args {
                        self.resolve_expression(a);
                    }
                    if let Some(c) = closure {
                        self.resolve_statement(c);
                    }
                }
                HirExpression::Match { scrutinee, arms, else_arm, .. } => {
                    self.resolve_expression(scrutinee);
                    for (_, e) in arms.iter_mut() {
                        self.resolve_expression(e);
                    }
                    if let Some(e) = else_arm.as_deref_mut() {
                        self.resolve_expression(e);
                    }
                }
                HirExpression::List { elements, .. } => {
                    for e in elements {
                        self.resolve_expression(e);
                    }
                }
                HirExpression::Index { target, index, .. } => {
                    self.resolve_expression(target);
                    self.resolve_expression(index);
                }
                HirExpression::Block { body, .. } => {
                    self.resolve_statement(body);
                }
                HirExpression::UnaryOp { operand, .. } => {
                    self.resolve_expression(operand);
                }
                HirExpression::Lambda { params, body, span, .. } => {
                    self.push_scope();
                    for p in params {
                        let id = self.define(p.name.clone(), HirBindingKind::Param, *span);
                        p.binding = Some(id);
                    }
                    self.resolve_statement(body);
                    self.pop_scope();
                }
            }
        }
    }

    let mut resolver = Resolver::new();
    resolver.resolve_program(program);
    HirBindingTable { bindings: resolver.bindings }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirTypeError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for HirTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {:?}", self.message, self.span)
    }
}

impl std::error::Error for HirTypeError {}

pub fn type_check(program: &mut HirProgram, bindings: &HirBindingTable) -> Result<(), HirTypeError> {
    struct Checker {
        binding_types: Vec<HirType>,
        return_stack: Vec<HirType>,
    }

    impl Checker {
        fn new(binding_count: usize) -> Self {
            Self { binding_types: vec![HirType::Unknown; binding_count], return_stack: Vec::new() }
        }

        fn get_binding_type(&self, id: HirBindingId) -> HirType {
            self.binding_types[id.0 as usize].clone()
        }

        fn set_binding_type(&mut self, id: HirBindingId, ty: HirType) {
            self.binding_types[id.0 as usize] = ty;
        }

        fn unify(&self, a: &HirType, b: &HirType) -> Option<HirType> {
            match (a, b) {
                (HirType::Unknown, t) | (t, HirType::Unknown) => Some(t.clone()),
                (x, y) if x == y => Some(x.clone()),
                _ => None,
            }
        }

        fn ensure_bool(&self, ty: &HirType) -> bool {
            matches!(ty, HirType::Bool | HirType::Unknown)
        }

        fn ensure_numeric(&self, ty: &HirType) -> bool {
            matches!(ty, HirType::Int | HirType::Float | HirType::Unknown)
        }

        fn type_error(&self, message: impl Into<String>, span: Span) -> HirTypeError {
            HirTypeError { message: message.into(), span }
        }

        fn check_program(&mut self, program: &mut HirProgram) -> Result<(), HirTypeError> {
            for stmt in &mut program.statements {
                self.check_statement(stmt)?;
            }
            Ok(())
        }

        fn check_statement(&mut self, stmt: &mut HirStatement) -> Result<(), HirTypeError> {
            match stmt {
                HirStatement::Namespace { .. } => Ok(()),
                HirStatement::Using { .. } => Ok(()),
                HirStatement::Block { statements, .. } => {
                    for s in statements {
                        self.check_statement(s)?;
                    }
                    Ok(())
                }
                HirStatement::Let { binding, ty, value, span, .. } => {
                    let val_ty = self.infer_expression(value)?;
                    let merged = self
                        .unify(ty, &val_ty)
                        .ok_or_else(|| self.type_error(format!("type mismatch: {:?} vs {:?}", ty, val_ty), *span))?;
                    *ty = merged.clone();
                    if let Some(id) = *binding {
                        self.set_binding_type(id, merged);
                    }
                    Ok(())
                }
                HirStatement::If { condition, then_branch, else_branch, span } => {
                    let cond_ty = self.infer_expression(condition)?;
                    if !self.ensure_bool(&cond_ty) {
                        return Err(self.type_error(format!("if condition must be Bool, got {:?}", cond_ty), *span));
                    }
                    self.check_statement(then_branch)?;
                    if let Some(else_b) = else_branch {
                        self.check_statement(else_b)?;
                    }
                    Ok(())
                }
                HirStatement::While { condition, body, span } => {
                    let cond_ty = self.infer_expression(condition)?;
                    if !self.ensure_bool(&cond_ty) {
                        return Err(self.type_error(format!("while condition must be Bool, got {:?}", cond_ty), *span));
                    }
                    self.check_statement(body)?;
                    Ok(())
                }
                HirStatement::Return { value, span } => {
                    if self.return_stack.is_empty() {
                        return Err(self.type_error("return outside of function", *span));
                    };

                    let idx = self.return_stack.len() - 1;
                    let expected = self.return_stack[idx].clone();

                    let actual = if let Some(v) = value { self.infer_expression(v)? } else { HirType::Void };
                    let merged = self.unify(&expected, &actual).ok_or_else(|| {
                        self.type_error(format!("return type mismatch: {:?} vs {:?}", expected, actual), *span)
                    })?;
                    self.return_stack[idx] = merged;
                    Ok(())
                }
                HirStatement::Class { binding, methods, .. } => {
                    if let Some(id) = *binding {
                        self.set_binding_type(id, HirType::Unknown); // Placeholder
                    }
                    for m in methods {
                        self.check_statement(m)?;
                    }
                    Ok(())
                }
                HirStatement::Trait { binding, methods, .. } => {
                    if let Some(id) = *binding {
                        self.set_binding_type(id, HirType::Unknown); // Placeholder
                    }
                    for m in methods {
                        self.check_statement(m)?;
                    }
                    Ok(())
                }
                HirStatement::Imply { methods, .. } => {
                    for m in methods {
                        self.check_statement(m)?;
                    }
                    Ok(())
                }
                HirStatement::Function { binding, params, return_type, body, span, .. } => {
                    if let Some(id) = *binding {
                        let func_ty = HirType::Function {
                            params: params.iter().map(|p| p.ty.clone()).collect(),
                            ret: Box::new(return_type.clone()),
                        };
                        self.set_binding_type(id, func_ty);
                    }

                    for p in params.iter() {
                        if let Some(id) = p.binding {
                            self.set_binding_type(id, p.ty.clone());
                        }
                    }

                    if let Some(body_stmt) = body {
                        self.return_stack.push(return_type.clone());
                        self.check_statement(body_stmt)?;
                        let inferred = self.return_stack.pop().unwrap_or(HirType::Void);
                        let merged = self.unify(return_type, &inferred).ok_or_else(|| {
                            self.type_error(format!("return type mismatch: {:?} vs {:?}", return_type, inferred), *span)
                        })?;
                        *return_type = merged;
                    }

                    if let Some(id) = *binding {
                        let func_ty = HirType::Function {
                            params: params.iter().map(|p| p.ty.clone()).collect(),
                            ret: Box::new(return_type.clone()),
                        };
                        self.set_binding_type(id, func_ty);
                    }
                    Ok(())
                }
                HirStatement::Expression { expression, .. } => {
                    let _ = self.infer_expression(expression)?;
                    Ok(())
                }
            }
        }

        fn infer_expression(&mut self, expr: &mut HirExpression) -> Result<HirType, HirTypeError> {
            match expr {
                HirExpression::Identifier { name, binding, ty, span } => {
                    let Some(id) = *binding
                    else {
                        return Err(self.type_error(format!("undefined identifier: {}", name), *span));
                    };
                    let t = self.get_binding_type(id);
                    *ty = t.clone();
                    Ok(t)
                }
                HirExpression::Literal { ty, .. } => Ok(ty.clone()),
                HirExpression::BinaryOp { left, op, right, ty, span } => {
                    let l = self.infer_expression(left)?;
                    let r = self.infer_expression(right)?;
                    let out = match op {
                        BinaryOperator::Add
                        | BinaryOperator::Sub
                        | BinaryOperator::Mul
                        | BinaryOperator::Div
                        | BinaryOperator::Rem
                        | BinaryOperator::BitAnd
                        | BinaryOperator::BitOr => {
                            if !self.ensure_numeric(&l) || !self.ensure_numeric(&r) {
                                return Err(self.type_error(
                                    format!("arithmetic/bitwise operands must be numeric, got {:?} and {:?}", l, r),
                                    *span,
                                ));
                            }
                            match (&l, &r) {
                                (HirType::Float, _) | (_, HirType::Float) => {
                                    if matches!(op, BinaryOperator::BitAnd | BinaryOperator::BitOr) {
                                          return Err(self.type_error("bitwise operations not supported on Float", *span));
                                     }
                                    HirType::Float
                                }
                                (HirType::Int, HirType::Int) => HirType::Int,
                                (HirType::Unknown, t) | (t, HirType::Unknown) => t.clone(),
                                _ => HirType::Unknown,
                            }
                        }
                        BinaryOperator::Equal | BinaryOperator::NotEqual => {
                            let merged = self.unify(&l, &r).ok_or_else(|| {
                                self.type_error(
                                    format!("equality operands must have same type, got {:?} and {:?}", l, r),
                                    *span,
                                )
                            })?;
                            if !matches!(
                                merged,
                                HirType::Int | HirType::Float | HirType::Bool | HirType::String | HirType::Unknown
                            ) {
                                return Err(self.type_error(format!("unsupported equality type: {:?}", merged), *span));
                            }
                            HirType::Bool
                        }
                        BinaryOperator::Less
                        | BinaryOperator::LessEqual
                        | BinaryOperator::Greater
                        | BinaryOperator::GreaterEqual => {
                            if !self.ensure_numeric(&l) || !self.ensure_numeric(&r) {
                                return Err(self.type_error(
                                    format!("comparison operands must be numeric, got {:?} and {:?}", l, r),
                                    *span,
                                ));
                            }
                            HirType::Bool
                        }
                        BinaryOperator::And | BinaryOperator::Or => {
                            if !self.ensure_bool(&l) || !self.ensure_bool(&r) {
                                return Err(self.type_error(
                                    format!("logical operands must be Bool, got {:?} and {:?}", l, r),
                                    *span,
                                ));
                            }
                            HirType::Bool
                        }
                        BinaryOperator::Assign => {
                            // Check lvalue type vs rvalue type
                            let merged = self.unify(&l, &r).ok_or_else(|| {
                                self.type_error(format!("assignment type mismatch: {:?} vs {:?}", l, r), *span)
                            })?;
                            merged
                        }
                    };
                    *ty = out.clone();
                    Ok(out)
                }
                HirExpression::UnaryOp { op, operand, ty, span } => {
                    let operand_ty = self.infer_expression(operand)?;
                    let out = match op {
                        UnaryOperator::Not => {
                            if !self.ensure_bool(&operand_ty) {
                                return Err(self.type_error(format!("! operand must be Bool, got {:?}", operand_ty), *span));
                            }
                            HirType::Bool
                        }
                        UnaryOperator::Neg => {
                            if !self.ensure_numeric(&operand_ty) {
                                return Err(self.type_error(format!("- operand must be numeric, got {:?}", operand_ty), *span));
                            }
                            operand_ty
                        }
                    };
                    *ty = out.clone();
                    Ok(out)
                }
                HirExpression::Lambda { params: _, body, ty, .. } => {
                     // TODO: Infer lambda params and body
                     self.check_statement(body)?;
                     let func_ty = HirType::Function { params: vec![], ret: Box::new(HirType::Unknown) };
                     *ty = func_ty.clone();
                     Ok(func_ty)
                }
                HirExpression::Call { callee, args, ty, span } => {
                    if let HirExpression::Identifier { name, ty: callee_ty, .. } = &mut **callee {
                        if name == "print" {
                            for a in args.iter_mut() {
                                let _ = self.infer_expression(a)?;
                            }
                            *callee_ty = HirType::Function { params: vec![HirType::Unknown], ret: Box::new(HirType::Void) };
                            *ty = HirType::Void;
                            return Ok(HirType::Void);
                        }
                    }

                    let callee_ty = self.infer_expression(callee)?;
                    let mut arg_tys = Vec::new();
                    for a in args.iter_mut() {
                        arg_tys.push(self.infer_expression(a)?);
                    }

                    let out = match callee_ty {
                        HirType::Function { params, ret } => {
                            if params.len() != arg_tys.len() {
                                return Err(self.type_error(
                                    format!("argument count mismatch: expected {}, got {}", params.len(), arg_tys.len()),
                                    *span,
                                ));
                            }
                            for (p, a) in params.iter().zip(arg_tys.iter()) {
                                if self.unify(p, a).is_none() {
                                    return Err(self
                                        .type_error(format!("argument type mismatch: expected {:?}, got {:?}", p, a), *span));
                                }
                            }
                            *ret
                        }
                        HirType::Unknown => HirType::Unknown,
                        other => {
                            return Err(self.type_error(format!("callee is not a function: {:?}", other), *span));
                        }
                    };
                    *ty = out.clone();
                    Ok(out)
                }
                HirExpression::Get { object, name, ty, span } => {
                    let _obj_ty = self.infer_expression(object)?;
                    // TODO: Check if field/method exists on obj_ty
                    *ty = HirType::Unknown;
                    Ok(HirType::Unknown)
                }
                HirExpression::Match { scrutinee, arms, else_arm, ty, span } => {
                    let scrutinee_ty = self.infer_expression(scrutinee)?;

                    let mut arm_types = Vec::new();
                    for (pat, arm_expr) in arms.iter_mut() {
                        if let MatchPattern::Literal(lit) = pat {
                            let pat_ty = match lit {
                                LiteralValue::Int(_) => HirType::Int,
                                LiteralValue::Float(_) => HirType::Float,
                                LiteralValue::Bool(_) => HirType::Bool,
                                LiteralValue::String(_) => HirType::String,
                            };
                            if self.unify(&scrutinee_ty, &pat_ty).is_none() {
                                return Err(self.type_error(
                                    format!("match pattern type mismatch: {:?} vs {:?}", scrutinee_ty, pat_ty),
                                    *span,
                                ));
                            }
                        }

                        arm_types.push(self.infer_expression(arm_expr)?);
                    }

                    let mut out_ty = HirType::Unknown;
                    for t in arm_types {
                        out_ty = self.unify(&out_ty, &t).unwrap_or(HirType::Unknown);
                    }

                    if let Some(e) = else_arm.as_deref_mut() {
                        let t = self.infer_expression(e)?;
                        out_ty = self.unify(&out_ty, &t).unwrap_or(HirType::Unknown);
                    }
                    else {
                        out_ty = self.unify(&out_ty, &HirType::Void).unwrap_or(out_ty);
                    }

                    *ty = out_ty.clone();
                    Ok(out_ty)
                }
                HirExpression::New { args, closure, ty, .. } => {
                    for arg in args {
                        self.infer_expression(arg)?;
                    }
                    if let Some(c) = closure {
                        // TODO: Check closure
                    }
                    *ty = HirType::Unknown;
                    Ok(HirType::Unknown)
                }
                HirExpression::List { elements, ty, span } => {
                    let mut elem_ty = HirType::Unknown;
                    if let Some(first) = elements.first_mut() {
                        elem_ty = self.infer_expression(first)?;
                    }
                    for e in elements.iter_mut().skip(1) {
                        let t = self.infer_expression(e)?;
                        elem_ty = self.unify(&elem_ty, &t).ok_or_else(|| {
                            self.type_error(format!("List element type mismatch: {:?} vs {:?}", elem_ty, t), *span)
                        })?;
                    }
                    let list_ty = HirType::List(Box::new(elem_ty));
                    *ty = list_ty.clone();
                    Ok(list_ty)
                }
                HirExpression::Index { target, index, ty, span, .. } => {
                    let target_ty = self.infer_expression(target)?;
                    let index_ty = self.infer_expression(index)?;

                    if index_ty != HirType::Int && index_ty != HirType::Unknown {
                        return Err(self.type_error(format!("Index must be Int, got {:?}", index_ty), *span));
                    }

                    match target_ty {
                        HirType::List(elem_ty) => {
                            *ty = *elem_ty.clone();
                            Ok(*elem_ty)
                        }
                        HirType::Unknown => {
                            *ty = HirType::Unknown;
                            Ok(HirType::Unknown)
                        }
                        _ => Err(self.type_error(format!("Cannot index non-list type: {:?}", target_ty), *span)),
                    }
                }
                HirExpression::Block { body, ty, .. } => {
                    self.check_statement(body)?;
                    *ty = HirType::Unknown;
                    Ok(HirType::Unknown)
                }
            }
        }
    }

    let mut checker = Checker::new(bindings.bindings.len());
    checker.check_program(program)
}
