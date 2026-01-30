use crate::{
    ast::{BinaryOperator, Expression, LiteralValue, MatchPattern, ProgramNode, Span, Statement, UnaryOperator},
    mir::{
        bytecode::{ClassMethod, OpCode},
        debug::DebugInfo,
    },
};
use std::{collections::HashMap, sync::Arc};

pub struct HirLowering {
    instructions: Vec<OpCode>,
    locals: HashMap<String, usize>, // name -> index
    next_local: usize,
    scopes: Vec<Vec<(String, Option<usize>)>>, // To handle scoping: Undo log
    namespace: Vec<String>,
    aliases: HashMap<String, String>,
    debug_info: DebugInfo,
    current_span: Option<Span>,
    pub promote_top_level_let_to_global: bool,
    depth: usize,
}

impl HirLowering {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            locals: HashMap::new(),
            next_local: 0,
            scopes: Vec::new(),
            namespace: Vec::new(),
            aliases: HashMap::new(),
            debug_info: DebugInfo::new(),
            current_span: None,
            promote_top_level_let_to_global: false,
            depth: 0,
        }
    }

    pub fn with_aliases(aliases: HashMap<String, String>) -> Self {
        Self {
            instructions: Vec::new(),
            locals: HashMap::new(),
            next_local: 0,
            scopes: Vec::new(),
            namespace: Vec::new(),
            aliases,
            debug_info: DebugInfo::new(),
            current_span: None,
            promote_top_level_let_to_global: false,
            depth: 0,
        }
    }

    pub fn compile(mut self, program: ProgramNode) -> (Vec<OpCode>, DebugInfo, HashMap<String, String>) {
        let len = program.statements.len();
        for (i, stmt) in program.statements.into_iter().enumerate() {
            let is_last = i == len - 1;
            self.compile_statement(stmt, is_last);
        }
        self.emit(OpCode::Return);
        (self.instructions, self.debug_info, self.aliases)
    }

    fn resolve_class_name(&self, name: &str) -> String {
        if let Some(full) = self.aliases.get(name) {
            full.clone()
        }
        else if !self.namespace.is_empty() && !name.contains("::") {
            format!("{}::{}", self.namespace.join("::"), name)
        }
        else {
            name.to_string()
        }
    }

    fn compile_statement(&mut self, stmt: Statement, push_result: bool) {
        self.depth += 1;
        
        if self.depth > 5000 {
            panic!("Stack overflow in HirLowering::compile_statement");
        }
        
        let span = stmt.span();
        let old_span = self.current_span;
        self.current_span = Some(span);

        match stmt {
            Statement::Namespace { path, .. } => {
                self.namespace = path;
                if push_result {
                    self.emit(OpCode::PushVoid);
                }
            }
            Statement::Using { path, .. } => {
                if let Some(alias) = path.last() {
                    let full_name = path.join("::");
                    self.aliases.insert(alias.clone(), full_name);
                }
                if push_result {
                    self.emit(OpCode::PushVoid);
                }
            }
            Statement::Expression { expression, .. } => {
                self.compile_expression(expression);
                if !push_result {
                    self.emit(OpCode::Pop);
                }
            }
            Statement::Let { name, value, .. } => {
                self.compile_expression(value);
                if self.promote_top_level_let_to_global {
                    self.emit(OpCode::StoreGlobal(name));
                } else {
                    let index = self.declare_local(name);
                    self.emit(OpCode::StoreLocal(index));
                }
                if push_result {
                    self.emit(OpCode::PushVoid);
                }
            }
            Statement::Return { value, .. } => {
                if let Some(v) = value {
                    self.compile_expression(v);
                }
                else {
                    self.emit(OpCode::PushVoid);
                }
                self.emit(OpCode::Return);
            }
            Statement::Block { statements, .. } => {
                let old_namespace = self.namespace.clone();
                let old_aliases = self.aliases.clone();

                let len = statements.len();
                if len == 0 && push_result {
                    self.emit(OpCode::PushVoid);
                }
                for (i, s) in statements.into_iter().enumerate() {
                    let is_last = i == len - 1;
                    self.compile_statement(s, push_result && is_last);
                }

                self.namespace = old_namespace;
                self.aliases = old_aliases;
            }
            Statement::If { mut condition, mut then_branch, mut else_branch, .. } => {
                let mut jump_end_indices = Vec::new();
                
                loop {
                    self.compile_expression(condition);
                    let jump_to_else_idx = self.instructions.len();
                    self.emit(OpCode::JumpIfFalse(0)); 

                    self.compile_statement(*then_branch, push_result);

                    let jump_to_end_idx = self.instructions.len();
                    self.emit(OpCode::Jump(0)); 
                    jump_end_indices.push(jump_to_end_idx);

                    let else_start = self.instructions.len();
                    self.instructions[jump_to_else_idx] = OpCode::JumpIfFalse(else_start);

                    if let Some(else_b) = else_branch {
                        // Check if else_b is strictly an If, or a Block containing a single If
                        let is_candidate = match &*else_b {
                            Statement::If { .. } => true,
                            Statement::Block { statements, .. } if statements.len() == 1 => {
                                matches!(statements[0], Statement::If { .. })
                            }
                            _ => false,
                        };

                        if is_candidate {
                            match *else_b {
                                Statement::If { condition: c, then_branch: t, else_branch: e, .. } => {
                                    condition = c;
                                    then_branch = t;
                                    else_branch = e;
                                    continue;
                                }
                                Statement::Block { mut statements, .. } => {
                                    if let Statement::If { condition: c, then_branch: t, else_branch: e, .. } = statements.remove(0) {
                                        condition = c;
                                        then_branch = t;
                                        else_branch = e;
                                        continue;
                                    } else {
                                        unreachable!("Guaranteed by is_candidate check");
                                    }
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            // Not an If chain, just compile the else block
                            self.compile_statement(*else_b, push_result);
                            break;
                        }
                    } else {
                        if push_result {
                             self.emit(OpCode::PushVoid);
                        }
                        break;
                    }
                }
                
                let end = self.instructions.len();
                for idx in jump_end_indices {
                    self.instructions[idx] = OpCode::Jump(end);
                }
            }
            Statement::While { condition, body, .. } => {
                let start_idx = self.instructions.len();
                self.compile_expression(condition);

                let jump_to_end_idx = self.instructions.len();
                self.emit(OpCode::JumpIfFalse(0)); // Placeholder

                self.compile_statement(*body, false);
                self.emit(OpCode::Jump(start_idx));

                let end_idx = self.instructions.len();
                self.instructions[jump_to_end_idx] = OpCode::JumpIfFalse(end_idx);

                if push_result {
                    self.emit(OpCode::PushVoid);
                }
            }
            Statement::Function { name, params, body, is_async, is_generator, .. } => {
                let full_name =
                    if self.namespace.is_empty() { name.clone() } else { format!("{}::{}", self.namespace.join("::"), name) };

                let mut function_compiler = HirLowering::new();
                function_compiler.namespace = self.namespace.clone();
                function_compiler.aliases = self.aliases.clone();
                for (p_name, _) in &params {
                    function_compiler.declare_local(p_name.clone());
                }
                if let Some(body_stmt) = body {
                    function_compiler.compile_statement(*body_stmt, true);
                }
                else {
                    function_compiler.emit(OpCode::PushVoid);
                    function_compiler.emit(OpCode::Return);
                }

                if function_compiler.instructions.last() != Some(&OpCode::Return) {
                    function_compiler.emit(OpCode::Return);
                }

                let body_ops = Arc::new(function_compiler.instructions);
                let body_debug = function_compiler.debug_info;
                let param_names = params.into_iter().map(|(n, _)| n).collect();

                self.emit(OpCode::DeclareFunction {
                    name: full_name,
                    params: param_names,
                    body: body_ops,
                    debug_info: body_debug,
                    is_async: is_async,
                    is_generator: is_generator,
                });

                if push_result {
                    self.emit(OpCode::PushVoid);
                }
            }
            Statement::Yield { value, .. } => {
                if let Some(v) = value {
                    self.compile_expression(v);
                }
                else {
                    self.emit(OpCode::PushVoid);
                }
                self.emit(OpCode::Yield);
                // Yield returns result of yield expression (value sent back)
                if !push_result {
                    self.emit(OpCode::Pop);
                }
            }
            Statement::Annotation { target, .. } => {
                self.compile_statement(*target, push_result);
            }
            Statement::Trait { name, methods, .. } => {
                let full_name =
                    if self.namespace.is_empty() { name.clone() } else { format!("{}::{}", self.namespace.join("::"), name) };

                let mut method_names = Vec::new();
                for m in methods {
                    if let Statement::Function { name: m_name, .. } = m {
                        method_names.push(m_name);
                    }
                }

                self.emit(OpCode::DeclareTrait { name: full_name, methods: method_names });

                if push_result {
                    self.emit(OpCode::PushVoid);
                }
            }
            Statement::Class { name, parents, traits, fields, methods, .. } => {
                let full_name =
                    if self.namespace.is_empty() { name.clone() } else { format!("{}::{}", self.namespace.join("::"), name) };

                let mut parent_names = Vec::new();
                for (_, p_type) in parents {
                    let mut current = &p_type;
                    while let crate::ast::TypeExpression::Generic { base, .. } = current {
                        current = base;
                    }
                    if let crate::ast::TypeExpression::Name { name: p_name, .. } = current {
                        parent_names.push(p_name.clone());
                    }
                }
                for t_type in traits {
                    let mut current = &t_type;
                    while let crate::ast::TypeExpression::Generic { base, .. } = current {
                        current = base;
                    }
                    if let crate::ast::TypeExpression::Name { name: t_name, .. } = current {
                        parent_names.push(t_name.clone());
                    }
                }

                let field_names = fields.into_iter().map(|(n, _)| n).collect();

                let mut class_methods = Vec::new();
                for m in methods {
                    if let Statement::Function { name: m_name, params, body, is_async, is_generator, .. } = m {
                        let mut func_compiler = HirLowering::new();
                        func_compiler.namespace = self.namespace.clone();
                        func_compiler.aliases = self.aliases.clone();
                        for (p_name, _) in &params {
                            func_compiler.declare_local(p_name.clone());
                        }
                        if let Some(b) = body {
                            func_compiler.compile_statement(*b, true);
                        }
                        else {
                            func_compiler.emit(OpCode::PushVoid);
                            func_compiler.emit(OpCode::Return);
                        }
                        if func_compiler.instructions.last() != Some(&OpCode::Return) {
                            func_compiler.emit(OpCode::Return);
                        }

                        class_methods.push(ClassMethod {
                            name: m_name,
                            params: params.into_iter().map(|(n, _)| n).collect(),
                            body: Arc::new(func_compiler.instructions),
                            debug_info: func_compiler.debug_info,
                            is_async,
                            is_generator,
                        });
                    }
                }

                self.emit(OpCode::DeclareClass {
                    name: full_name,
                    parents: parent_names,
                    fields: field_names,
                    methods: class_methods,
                });
                if push_result {
                    self.emit(OpCode::PushVoid);
                }
            }
            Statement::Imply { target, trait_target, methods, .. } => {
                let mut current = &target;
                while let crate::ast::TypeExpression::Generic { base, .. } = current {
                    current = base;
                }
                let class_name = if let crate::ast::TypeExpression::Name { name, .. } = current {
                    self.resolve_class_name(name)
                }
                else {
                    panic!("Complex target in imply not supported");
                };

                let trait_name = if let Some(t) = trait_target {
                    let mut current = &t;
                    while let crate::ast::TypeExpression::Generic { base, .. } = current {
                        current = base;
                    }
                    if let crate::ast::TypeExpression::Name { name, .. } = current { Some(name.clone()) } else { None }
                } else { None };

                let mut class_methods = Vec::new();
                for m in methods {
                    if let Statement::Function { name: m_name, params, body, is_async, is_generator, .. } = m {
                        let mut func_compiler = HirLowering::new();
                        func_compiler.namespace = self.namespace.clone();
                        func_compiler.aliases = self.aliases.clone();
                        for (p_name, _) in &params {
                            func_compiler.declare_local(p_name.clone());
                        }
                        if let Some(b) = body {
                            func_compiler.compile_statement(*b, true);
                        }
                        else {
                            func_compiler.emit(OpCode::PushVoid);
                            func_compiler.emit(OpCode::Return);
                        }
                        if func_compiler.instructions.last() != Some(&OpCode::Return) {
                            func_compiler.emit(OpCode::Return);
                        }

                        class_methods.push(ClassMethod {
                            name: m_name,
                            params: params.into_iter().map(|(n, _)| n).collect(),
                            body: Arc::new(func_compiler.instructions),
                            debug_info: func_compiler.debug_info,
                            is_async,
                            is_generator,
                        });
                    }
                }

                self.emit(OpCode::ExtendClass { name: class_name, trait_name, methods: class_methods });
                if push_result {
                    self.emit(OpCode::PushVoid);
                }
            }
            _ => {
                // Unimplemented statements
                if push_result {
                    self.emit(OpCode::PushVoid);
                }
            }
        }

        self.current_span = old_span;
        self.depth -= 1;
    }

    fn compile_expression(&mut self, expr: Expression) {
        let span = expr.span();
        let old_span = self.current_span;
        self.current_span = Some(span);

        match expr {
            Expression::Literal { value, .. } => match value {
                LiteralValue::Int(v) => self.emit(OpCode::PushInt(v)),
                LiteralValue::Float(v) => self.emit(OpCode::PushFloat(f64::from_bits(v))),
                LiteralValue::Bool(v) => self.emit(OpCode::PushBool(v)),
                LiteralValue::String(v) => self.emit(OpCode::PushString(v)),
            },
            Expression::Identifier { name, .. } => {
                if let Some(&index) = self.locals.get(&name) {
                    self.emit(OpCode::LoadLocal(index));
                }
                else {
                    let resolved = self.aliases.get(&name).cloned().unwrap_or(name);
                    self.emit(OpCode::LoadGlobal(resolved));
                }
            }
            Expression::BinaryOp { left, op, right, .. } => {
                if let BinaryOperator::Assign = op {
                    match *left {
                        Expression::Identifier { name, .. } => {
                            self.compile_expression(*right);
                            if let Some(&index) = self.locals.get(&name) {
                                self.emit(OpCode::StoreLocal(index));
                            }
                            else {
                                self.emit(OpCode::StoreGlobal(name));
                            }
                            self.emit(OpCode::PushVoid);
                        }
                        Expression::Get { object, name, .. } => {
                            self.compile_expression(*object);
                            self.compile_expression(*right);
                            self.emit(OpCode::SetField(name));
                        }
                        Expression::Index { target, index, is_zero_based, .. } => {
                             self.compile_expression(*target);
                             self.compile_expression(*index);
                             if !is_zero_based {
                                 self.emit(OpCode::PushInt(1));
                                 self.emit(OpCode::Sub);
                             }
                             self.compile_expression(*right);
                             self.emit(OpCode::CallMethod("set".to_string(), 2));
                        }
                        _ => panic!("Invalid assignment target"),
                    }
                }
                else {
                    match op {
                        BinaryOperator::And => {
                            // Short-circuiting AND
                            self.compile_expression(*left);
                            // Stack: [left]
                            let jump_false = self.instructions.len();
                            self.emit(OpCode::JumpIfFalse(0)); // If false, keep false on stack and jump
                            // If true:
                            self.emit(OpCode::Pop); // Pop left (true)
                            self.compile_expression(*right); // Push right
                            
                            let end = self.instructions.len();
                            self.instructions[jump_false] = OpCode::JumpIfFalse(end);
                        }
                        BinaryOperator::Or => {
                             // Short-circuiting OR
                             self.compile_expression(*left);
                             // Stack: [left]
                             let jump_true = self.instructions.len();
                             self.emit(OpCode::JumpIfTrue(0)); // If true, keep true and jump
                             // If false:
                             self.emit(OpCode::Pop);
                             self.compile_expression(*right);
                             
                             let end = self.instructions.len();
                             self.instructions[jump_true] = OpCode::JumpIfTrue(end);
                        }
                        _ => {
                            // Flatten left-recursive BinaryOps to avoid stack overflow
                            let mut stack = Vec::new();
                            let mut current_left = left;
                            let mut current_op = op;
                            let mut current_right = right;

                            loop {
                                let mut is_target = false;
                                if let Expression::BinaryOp { op: o, .. } = &*current_left {
                                     if *o != BinaryOperator::Assign && *o != BinaryOperator::And && *o != BinaryOperator::Or {
                                         is_target = true;
                                     }
                                }

                                if is_target {
                                     if let Expression::BinaryOp { left: l, op: o, right: r, .. } = *current_left {
                                         stack.push((current_op, current_right));
                                         current_left = l;
                                         current_op = o;
                                         current_right = r;
                                         continue;
                                     }
                                }
                                break;
                            }

                            stack.push((current_op, current_right));

                            self.compile_expression(*current_left);

                            while let Some((op, right)) = stack.pop() {
                                self.compile_expression(*right);
                                match op {
                                    BinaryOperator::Add => self.emit(OpCode::Add),
                                    BinaryOperator::Sub => self.emit(OpCode::Sub),
                                    BinaryOperator::Mul => self.emit(OpCode::Mul),
                                    BinaryOperator::Div => self.emit(OpCode::Div),
                                    BinaryOperator::Rem => self.emit(OpCode::Rem),
                                    BinaryOperator::Equal => self.emit(OpCode::Eq),
                                    BinaryOperator::NotEqual => self.emit(OpCode::Neq),
                                    BinaryOperator::Less => self.emit(OpCode::Lt),
                                    BinaryOperator::LessEqual => self.emit(OpCode::Le),
                                    BinaryOperator::Greater => self.emit(OpCode::Gt),
                                    BinaryOperator::GreaterEqual => self.emit(OpCode::Ge),
                                    BinaryOperator::BitAnd => self.emit(OpCode::BitAnd),
                                    BinaryOperator::BitOr => self.emit(OpCode::BitOr),
                                    _ => panic!("Unsupported binary op"),
                                }
                            }
                        }
                    }
                }
            }
            Expression::UnaryOp { op, operand, .. } => {
                self.compile_expression(*operand);
                match op {
                     UnaryOperator::Not => self.emit(OpCode::Not),
                     UnaryOperator::Neg => self.emit(OpCode::Neg),
                }
            }
            Expression::Call { callee, args, .. } => {
                if let Expression::Get { object, name, .. } = *callee {
                    self.compile_expression(*object);
                    let arg_count = args.len();
                    for arg in args {
                        self.compile_expression(arg);
                    }
                    self.emit(OpCode::CallMethod(name, arg_count));
                }
                else {
                    // Evaluate args first
                    let arg_count = args.len();
                    for arg in args {
                        self.compile_expression(arg);
                    }

                    match *callee {
                        Expression::Identifier { name, .. } => {
                            if let Some(&index) = self.locals.get(&name) {
                                self.emit(OpCode::LoadLocal(index));
                                self.emit(OpCode::CallStack(arg_count));
                            }
                            else {
                                let resolved = self.aliases.get(&name).cloned().unwrap_or(name);
                                self.emit(OpCode::Call(resolved, arg_count));
                            }
                        }
                        _ => {
                            self.compile_expression(*callee);
                            self.emit(OpCode::CallStack(arg_count));
                        }
                    }
                }
            }
            Expression::Await { future, .. } => {
                self.compile_expression(*future);
                self.emit(OpCode::Await);
            }
            Expression::Get { object, name, .. } => {
                self.compile_expression(*object);
                self.emit(OpCode::GetField(name));
            }
            Expression::New { class, args, closure: _closure, .. } => {
                let arg_count = args.len();
                for arg in args {
                    self.compile_expression(arg);
                }
                let class_name = if let crate::ast::TypeExpression::Name { name, .. } = class {
                    self.resolve_class_name(&name)
                }
                else {
                    panic!("Complex type in new not supported");
                };
                self.emit(OpCode::New(class_name, arg_count));
            }
            Expression::Match { scrutinee, arms, else_arm, .. } => {
                self.compile_expression(*scrutinee);

                let mut jump_to_end_indices = Vec::new();

                for (pattern, body) in arms {
                    match pattern {
                        MatchPattern::Literal(lit) => {
                            self.emit(OpCode::Dup);
                            match lit {
                                LiteralValue::Int(v) => self.emit(OpCode::PushInt(v)),
                                LiteralValue::Float(v) => self.emit(OpCode::PushFloat(f64::from_bits(v))),
                                LiteralValue::Bool(v) => self.emit(OpCode::PushBool(v)),
                                LiteralValue::String(v) => self.emit(OpCode::PushString(v)),
                            }
                            self.emit(OpCode::Eq);

                            let jump_next_idx = self.instructions.len();
                            self.emit(OpCode::JumpIfFalse(0)); // Placeholder

                            // Match found
                            self.emit(OpCode::Pop); // Pop original val
                            self.compile_expression(body);

                            jump_to_end_indices.push(self.instructions.len());
                            self.emit(OpCode::Jump(0)); // Placeholder

                            // Patch jump_next
                            let next_arm_start = self.instructions.len();
                            self.instructions[jump_next_idx] = OpCode::JumpIfFalse(next_arm_start);
                        }
                        MatchPattern::Wildcard => {
                            self.emit(OpCode::Pop);
                            self.compile_expression(body);
                            jump_to_end_indices.push(self.instructions.len());
                            self.emit(OpCode::Jump(0));
                        }
                    }
                }

                if let Some(else_expr) = else_arm {
                    self.emit(OpCode::Pop);
                    self.compile_expression(*else_expr);
                }
                else {
                    self.emit(OpCode::Pop);
                    self.emit(OpCode::PushVoid);
                }

                let end_idx = self.instructions.len();
                for idx in jump_to_end_indices {
                    self.instructions[idx] = OpCode::Jump(end_idx);
                }
            }
            Expression::Lambda { params, body, is_async, is_generator, .. } => {
                let mut func_compiler = HirLowering::new();
                func_compiler.namespace = self.namespace.clone();
                func_compiler.aliases = self.aliases.clone();
                for (p_name, _) in &params {
                    func_compiler.declare_local(p_name.clone());
                }
                func_compiler.compile_statement(*body, false);

                if func_compiler.instructions.last() != Some(&OpCode::Return) {
                    func_compiler.emit(OpCode::PushVoid);
                    func_compiler.emit(OpCode::Return);
                }

                let body_ops = Arc::new(func_compiler.instructions);
                let body_debug = func_compiler.debug_info;
                let param_names = params.into_iter().map(|(n, _)| n).collect();

                self.emit(OpCode::Lambda {
                    params: param_names,
                    body: body_ops,
                    debug_info: body_debug,
                    is_async,
                    is_generator,
                });
            }
            Expression::List { elements, .. } => {
                let count = elements.len();
                for e in elements {
                    self.compile_expression(e);
                }
                self.emit(OpCode::NewList(count));
            }
            Expression::Index { target, index, is_zero_based, .. } => {
                self.compile_expression(*target);
                self.compile_expression(*index);
                if !is_zero_based {
                    self.emit(OpCode::PushInt(1));
                    self.emit(OpCode::Sub);
                }
                self.emit(OpCode::GetIndex);
            }
            Expression::MacroCall { name, args, span: _ } => {
                if name == "println" || name == "print" {
                    for arg in args {
                        self.compile_expression(arg);
                        self.emit(OpCode::Print);
                    }
                    self.emit(OpCode::PushVoid);
                }
                else if name == "dbg" {
                    if let Some(arg) = args.first() {
                        self.compile_expression(arg.clone());
                        self.emit(OpCode::Dup);
                        self.emit(OpCode::Print);
                    }
                    else {
                        self.emit(OpCode::PushVoid);
                    }
                }
                else if name == "stringify" {
                    if let Some(arg) = args.first() {
                        let s = format!("{:?}", arg);
                        self.emit(OpCode::PushString(s));
                    }
                    else {
                        self.emit(OpCode::PushString("".to_string()));
                    }
                }
                else {
                    panic!("Unknown macro or unimplemented user macro: {}", name);
                }
            }
            Expression::Block { body, .. } => {
                self.compile_statement(*body, true);
            }
            _ => panic!("Unimplemented expression compilation"),
        }

        self.current_span = old_span;
    }

    fn emit(&mut self, op: OpCode) {
        let idx = self.instructions.len();
        self.instructions.push(op);
        if let Some(span) = self.current_span {
            self.debug_info.insert(idx, span);
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn leave_scope(&mut self) {
        if let Some(changes) = self.scopes.pop() {
            for (name, old_idx) in changes.into_iter().rev() {
                if let Some(idx) = old_idx {
                    self.locals.insert(name, idx);
                } else {
                    self.locals.remove(&name);
                }
            }
        }
    }

    fn declare_local(&mut self, name: String) -> usize {
        let index = self.next_local;

        // Log change if we are in a scope
        if let Some(scope) = self.scopes.last_mut() {
            let old_idx = self.locals.get(&name).cloned();
            scope.push((name.clone(), old_idx));
        }

        self.locals.insert(name, index);
        self.next_local += 1;
        index
    }
}
