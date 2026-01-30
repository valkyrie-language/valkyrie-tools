use crate::ast::{parser::ParseError, Expression, LiteralValue, Position, ProgramNode, Span, Statement, TypeExpression};
use crate::mir::interpreter::{eval_expression, Environment};
use crate::mir::value::RuntimeValue;
use std::path::PathBuf;
use std::collections::{HashSet, HashMap};

pub struct MacroExpander {
    base_dir: PathBuf,
    loaded_modules: HashSet<PathBuf>,
    user_macros: HashMap<String, (Vec<String>, Statement)>,
    compile_time_env: Environment,
    depth: usize,
}

fn run_async<F: std::future::Future>(f: F) -> F::Output {
    futures::executor::block_on(f)
}

impl MacroExpander {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            loaded_modules: HashSet::new(),
            user_macros: HashMap::new(),
            compile_time_env: Environment::new(),
            depth: 0,
        }
    }

    pub fn expand_program(&mut self, mut program: ProgramNode) -> ProgramNode {
        self.depth += 1;
        if self.depth > 100 {
            panic!("Stack overflow detected in macro expansion!");
        }

        // 1. Collect user-defined macros and functions from the program
        self.collect_definitions(&program.statements);

        // 2. Expand all statements
        let mut statements = Vec::new();
        for stmt in program.statements {
            statements.extend(self.expand_statement(stmt));
        }
        self.depth -= 1;
        ProgramNode { statements, span: program.span }
    }

    fn expand_block(&mut self, stmt: Statement) -> Statement {
        let span = stmt.span();
        let stmts = self.expand_statement(stmt);
        if stmts.len() == 1 {
            if let Statement::Block { .. } = &stmts[0] {
                return stmts[0].clone();
            }
        }
        Statement::Block { statements: stmts, span }
    }

    fn collect_definitions(&mut self, statements: &[Statement]) {
        for stmt in statements {
            match stmt {
                Statement::Macro { name, params, body, .. } => {
                    let param_names = params.iter().map(|(n, _)| n.clone()).collect();
                    self.user_macros.insert(name.clone(), (param_names, *body.clone()));
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
                        self.compile_time_env.define(name.clone(), func);
                    }
                }
                Statement::Annotation { name, target, .. } => {
                    if name == "const_fn" {
                        if let Statement::Function { name: f_name, params, body, is_async, is_generator, .. } = &**target {
                            if let Some(b) = body {
                                let func = RuntimeValue::Function {
                                    name: f_name.clone(),
                                    params: params.iter().map(|(n, _)| n.clone()).collect(),
                                    body: b.clone(),
                                    owner: None,
                                    is_async: *is_async,
                                    is_generator: *is_generator,
                                };
                                self.compile_time_env.define(f_name.clone(), func);
                            }
                        }
                    }
                    // Also collect macros defined within annotations if any
                }
                _ => {}
            }
        }
    }

    fn expand_statement(&mut self, stmt: Statement) -> Vec<Statement> {
        match stmt {
            Statement::Using { path, span } => {
                let mut stmts = Vec::new();
                if let Some(file_path) = self.resolve_module(&path) {
                    if !self.loaded_modules.contains(&file_path) {
                        self.loaded_modules.insert(file_path.clone());
                        if let Ok(content) = std::fs::read_to_string(&file_path) {
                            let lexer = crate::ast::lexer::Lexer::new(&content);
                            let mut parser = crate::ast::parser::Parser::new(lexer);
                            match parser.parse_program() {
                                Ok(prog) => {
                                    let old_base = self.base_dir.clone();
                                    if let Some(parent) = file_path.parent() {
                                        self.base_dir = parent.to_path_buf();
                                    }
                                    let expanded = self.expand_program(prog);
                                    self.base_dir = old_base;
                                    stmts.extend(expanded.statements);
                                }
                                Err(e) => {
                                    eprintln!("Error parsing module {:?}: {:?}", file_path, e);
                                }
                            }
                        } else {
                            eprintln!("Error reading module {:?}", file_path);
                        }
                    }
                }
                stmts.push(Statement::Using { path, span });
                stmts
            }
            Statement::Macro { .. } => {
                // Macros are removed after expansion
                vec![]
            }
            Statement::Expression { expression, span } => {
                if let Expression::MacroCall { name, args, span: _ } = &expression {
                    if name == "include" {
                        if let Some(first_arg) = args.first() {
                            if let Expression::Literal { value: LiteralValue::String(path_str), .. } = first_arg {
                                let path = self.base_dir.join(path_str);
                                if let Ok(src) = std::fs::read_to_string(&path) {
                                    let lexer = crate::ast::lexer::Lexer::new(&src);
                                    let mut parser = crate::ast::parser::Parser::new(lexer);
                                    match parser.parse_program() {
                                        Ok(program) => {
                                            let old_base = self.base_dir.clone();
                                            if let Some(parent) = path.parent() {
                                                self.base_dir = parent.to_path_buf();
                                            }
                                            let expanded = self.expand_program(program);
                                            self.base_dir = old_base;
                                            return expanded.statements;
                                        }
                                        Err(e) => {
                                            eprintln!("Error parsing included file {:?}: {:?}", path, e);
                                            return vec![];
                                        }
                                    }
                                } else {
                                    eprintln!("Error reading included file {:?}", path);
                                    return vec![];
                                }
                            }
                        }
                    }
                }
                vec![Statement::Expression { expression: self.expand_expression(expression), span }]
            }
            Statement::Block { statements, span } => {
                let mut expanded_stmts = Vec::new();
                for s in statements {
                    expanded_stmts.extend(self.expand_statement(s));
                }
                vec![Statement::Block { statements: expanded_stmts, span }]
            }
            Statement::Let { name, type_hint, value, span } => {
                vec![Statement::Let { name, type_hint, value: self.expand_expression(value), span }]
            }
            Statement::If { condition, then_branch, else_branch, span } => {
                let then_block = self.expand_block(*then_branch);
                let else_block = else_branch.map(|b| Box::new(self.expand_block(*b)));
                vec![Statement::If {
                    condition: self.expand_expression(condition),
                    then_branch: Box::new(then_block),
                    else_branch: else_block,
                    span,
                }]
            }
            Statement::Return { value, span } => {
                vec![Statement::Return { value: value.map(|v| self.expand_expression(v)), span }]
            }
            Statement::Function { name, generics, params, return_type, body, is_async, is_generator, span } => {
                vec![Statement::Function {
                    name,
                    generics,
                    params,
                    return_type,
                    body: body.map(|b| Box::new(self.expand_block(*b))),
                    is_async,
                    is_generator,
                    span,
                }]
            }
            Statement::Class { name, generics, parents, traits, fields, methods, span } => {
                let mut expanded_methods = Vec::new();
                for m in methods {
                    expanded_methods.extend(self.expand_statement(m));
                }
                vec![Statement::Class { name, generics, parents, traits, fields, methods: expanded_methods, span }]
            }
            Statement::Trait { name, generics, parents, methods, span } => {
                let mut expanded_methods = Vec::new();
                for m in methods {
                    expanded_methods.extend(self.expand_statement(m));
                }
                vec![Statement::Trait { name, generics, parents, methods: expanded_methods, span }]
            }
            Statement::Imply { target, generics, trait_target, methods, span } => {
                let mut expanded_methods = Vec::new();
                for m in methods {
                    expanded_methods.extend(self.expand_statement(m));
                }
                vec![Statement::Imply { target, generics, trait_target, methods: expanded_methods, span }]
            }
            Statement::Annotation { name, args, target, span } => {
                if name == "evaluate" {
                    if let Statement::Expression { expression, span: e_span } = &*target {
                        let result = run_async(eval_expression(expression, &mut self.compile_time_env));
                        match result {
                            Ok(val) => {
                                let lit_expr = self.runtime_value_to_expression(val, *e_span);
                                return vec![Statement::Expression { expression: lit_expr, span: *e_span }];
                            }
                            Err(e) => {
                                eprintln!("Compile-time evaluation error: {:?}", e);
                            }
                        }
                    }
                }
                
                if name == "const_fn" {
                    // Mark function for compile-time execution
                    let expanded_target = self.expand_statement(*target);
                    for stmt in &expanded_target {
                        if let Statement::Function { name, params, body, is_async, is_generator, .. } = stmt {
                            if let Some(b) = body {
                                let func = RuntimeValue::Function {
                                    name: name.clone(),
                                    params: params.iter().map(|(n, _)| n.clone()).collect(),
                                    body: b.clone(),
                                    owner: None,
                                    is_async: *is_async,
                                    is_generator: *is_generator,
                                };
                                self.compile_time_env.define(name.clone(), func);
                            }
                        }
                    }
                    return expanded_target;
                }
                
                // If it's not @evaluate or evaluation failed, check if it's an attribute macro
                if let Some((param_names, body)) = self.user_macros.get(&name).cloned() {
                    let mut arg_map = HashMap::new();
                    // Match provided arguments
                    for (i, param_name) in param_names.iter().take(args.len()).enumerate() {
                        arg_map.insert(param_name.clone(), args[i].clone());
                    }
                    
                    // If there's one more parameter, it's the target
                    if param_names.len() == args.len() + 1 {
                        let target_expr = Expression::Block { body: target.clone(), span: target.span() };
                        arg_map.insert(param_names.last().unwrap().clone(), target_expr);
                    } else if param_names.is_empty() && args.is_empty() {
                        // Special case: no params macro used as annotation
                        // Maybe it just wraps the target or replaces it
                    }

                    let substituted = self.substitute_macro_args(body, &arg_map);
                    let expanded = self.expand_statement(substituted);
                    // If macro returns a lambda expression, we might need to convert it to a function statement
                    // but for attribute macros wrapping functions, they usually return the lambda directly
                    // which is then treated as an expression statement.
                    return expanded;
                }

                let expanded_args = args.into_iter().map(|a| self.expand_expression(a)).collect();
                let expanded_target = self.expand_block(*target);
                vec![Statement::Annotation { name, args: expanded_args, target: Box::new(expanded_target), span }]
            }
            _ => vec![stmt],
        }
    }

    fn expand_expression(&mut self, expr: Expression) -> Expression {
        match expr {
            Expression::MacroCall { name, args, span } => {
                // Check built-in macros
                if name == "evaluate" {
                    if let Some(first_arg) = args.first() {
                        let result = run_async(eval_expression(first_arg, &mut self.compile_time_env));
                        match result {
                            Ok(val) => {
                                return self.runtime_value_to_expression(val, span);
                            }
                            Err(e) => {
                                eprintln!("Compile-time evaluation error: {:?}", e);
                            }
                        }
                    }
                }
                if name == "stringify" {
                    let s = args.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>().join(", ");
                    return Expression::Literal { value: LiteralValue::String(s), span };
                }
                if name == "location.line_number" {
                    return Expression::Literal { value: LiteralValue::Int(span.start.line as i64), span };
                }
                if name == "concat_number" {
                    if args.len() == 2 {
                        let arg1 = self.expand_expression(args[0].clone());
                        let arg2 = self.expand_expression(args[1].clone());
                        if let (
                            Expression::Literal { value: LiteralValue::Int(v1), .. },
                            Expression::Literal { value: LiteralValue::Int(v2), .. },
                        ) = (&arg1, &arg2)
                        {
                            let combined = format!("{}{}", v1, v2);
                            if let Ok(i) = combined.parse::<i64>() {
                                return Expression::Literal { value: LiteralValue::Int(i), span };
                            }
                        }
                    }
                }

                // Check user-defined macros
                if let Some((param_names, body)) = self.user_macros.get(&name).cloned() {
                    if param_names.len() == args.len() {
                        let mut arg_map = HashMap::new();
                        for (i, param_name) in param_names.into_iter().enumerate() {
                            arg_map.insert(param_name, args[i].clone());
                        }
                        
                        let substituted_body = self.substitute_macro_args(body, &arg_map);
                        let expanded_stmts = self.expand_statement(substituted_body);
                        
                        if expanded_stmts.len() == 1 {
                            match &expanded_stmts[0] {
                                Statement::Expression { expression, .. } => {
                                    return expression.clone();
                                }
                                Statement::Return { value: Some(expr), .. } => {
                                    return expr.clone();
                                }
                                _ => {}
                            }
                        }
                        
                        return Expression::Block { 
                            body: Box::new(Statement::Block { 
                                statements: expanded_stmts, 
                                span 
                            }), 
                            span 
                        };
                    }
                }

                let expanded_args = args.into_iter().map(|a| self.expand_expression(a)).collect();
                Expression::MacroCall { name, args: expanded_args, span }
            }
            Expression::BinaryOp { left, op, right, span } => Expression::BinaryOp {
                left: Box::new(self.expand_expression(*left)),
                op,
                right: Box::new(self.expand_expression(*right)),
                span,
            },
            Expression::Call { callee, args, span } => {
                let expanded_args = args.into_iter().map(|a| self.expand_expression(a)).collect();
                Expression::Call { callee: Box::new(self.expand_expression(*callee)), args: expanded_args, span }
            }
            Expression::Get { object, name, span } => {
                Expression::Get { object: Box::new(self.expand_expression(*object)), name, span }
            }
            Expression::New { class, args, closure, span } => {
                let expanded_args = args.into_iter().map(|a| self.expand_expression(a)).collect();
                let expanded_closure = closure.map(|c| Box::new(self.expand_block(*c)));
                Expression::New { class, args: expanded_args, closure: expanded_closure, span }
            }
            Expression::Match { scrutinee, arms, else_arm, span } => {
                let s = Box::new(self.expand_expression(*scrutinee));
                let mut new_arms = Vec::new();
                for (pat, e) in arms {
                    new_arms.push((pat, self.expand_expression(e)));
                }
                let e_arm = else_arm.map(|e| Box::new(self.expand_expression(*e)));
                Expression::Match { scrutinee: s, arms: new_arms, else_arm: e_arm, span }
            }
            Expression::Await { future, span } => Expression::Await { future: Box::new(self.expand_expression(*future)), span },
            Expression::List { elements, span } => {
                let expanded = elements.into_iter().map(|e| self.expand_expression(e)).collect();
                Expression::List { elements: expanded, span }
            }
            Expression::Index { target, index, is_zero_based, span } => {
                Expression::Index {
                    target: Box::new(self.expand_expression(*target)),
                    index: Box::new(self.expand_expression(*index)),
                    is_zero_based,
                    span,
                }
            }
            Expression::UnaryOp { op, operand, span } => {
                Expression::UnaryOp { op, operand: Box::new(self.expand_expression(*operand)), span }
            }
            Expression::Block { body, span } => {
                Expression::Block { body: Box::new(self.expand_block(*body)), span }
            }
            _ => expr,
        }
    }

    fn substitute_macro_args(&self, stmt: Statement, args: &HashMap<String, Expression>) -> Statement {
        match stmt {
            Statement::Expression { expression, span } => {
                if let Expression::Identifier { name, .. } = &expression {
                    if let Some(arg_expr) = args.get(name) {
                        if let Expression::Block { body, .. } = arg_expr {
                            return *body.clone();
                        }
                    }
                }
                Statement::Expression { expression: self.substitute_expr_args(expression, args), span }
            }
            Statement::Return { value, span } => {
                if let Some(Expression::Identifier { name, .. }) = &value {
                    if let Some(arg_expr) = args.get(name) {
                        if let Expression::Block { body, .. } = arg_expr {
                            return *body.clone();
                        }
                    }
                }
                Statement::Return { value: value.map(|v| self.substitute_expr_args(v, args)), span }
            }
            Statement::Block { statements, span } => {
                let mut new_stmts = Vec::new();
                for s in statements {
                    new_stmts.push(self.substitute_macro_args(s, args));
                }
                Statement::Block { statements: new_stmts, span }
            }
            Statement::Let { name, type_hint, value, span } => {
                Statement::Let { name, type_hint, value: self.substitute_expr_args(value, args), span }
            }
            Statement::If { condition, then_branch, else_branch, span } => {
                Statement::If {
                    condition: self.substitute_expr_args(condition, args),
                    then_branch: Box::new(self.substitute_macro_args(*then_branch, args)),
                    else_branch: else_branch.map(|b| Box::new(self.substitute_macro_args(*b, args))),
                    span,
                }
            }
            Statement::Function { name, generics, params, return_type, body, is_async, is_generator, span } => {
                Statement::Function {
                    name,
                    generics,
                    params,
                    return_type,
                    body: body.map(|b| Box::new(self.substitute_macro_args(*b, args))),
                    is_async,
                    is_generator,
                    span,
                }
            }
            Statement::Annotation { name, args: attr_args, target, span } => {
                Statement::Annotation {
                    name,
                    args: attr_args.into_iter().map(|a| self.substitute_expr_args(a, args)).collect(),
                    target: Box::new(self.substitute_macro_args(*target, args)),
                    span,
                }
            }
            // Add more cases as needed, or keep as is if not supported in macros
            _ => stmt,
        }
    }

    fn substitute_expr_args(&self, expr: Expression, args: &HashMap<String, Expression>) -> Expression {
        match expr {
            Expression::Identifier { name, span } => {
                if let Some(arg_expr) = args.get(&name) {
                    let mut new_expr = arg_expr.clone();
                    // Keep the original span if it's just an identifier being replaced?
                    // Actually, usually we want the span of the call site argument.
                    return new_expr;
                }
                Expression::Identifier { name, span }
            }
            Expression::BinaryOp { left, op, right, span } => Expression::BinaryOp {
                left: Box::new(self.substitute_expr_args(*left, args)),
                op,
                right: Box::new(self.substitute_expr_args(*right, args)),
                span,
            },
            Expression::UnaryOp { op, operand, span } => Expression::UnaryOp {
                op,
                operand: Box::new(self.substitute_expr_args(*operand, args)),
                span,
            },
            Expression::Call { callee, args: call_args, span } => {
                let new_args = call_args.into_iter().map(|a| self.substitute_expr_args(a, args)).collect();
                Expression::Call { callee: Box::new(self.substitute_expr_args(*callee, args)), args: new_args, span }
            }
            Expression::MacroCall { name, args: macro_args, span } => {
                let new_args = macro_args.into_iter().map(|a| self.substitute_expr_args(a, args)).collect();
                Expression::MacroCall { name, args: new_args, span }
            }
            Expression::Block { body, span } => {
                Expression::Block { body: Box::new(self.substitute_macro_args(*body, args)), span }
            }
            _ => expr,
        }
    }

    fn runtime_value_to_expression(&self, val: RuntimeValue, span: Span) -> Expression {
        match val {
            RuntimeValue::Int(v) => Expression::Literal { value: LiteralValue::Int(v), span },
            RuntimeValue::Bool(v) => Expression::Literal { value: LiteralValue::Bool(v), span },
            RuntimeValue::Float(v) => Expression::Literal { value: LiteralValue::Float(v.to_bits()), span },
            RuntimeValue::String(v) => Expression::Literal { value: LiteralValue::String(v), span },
            _ => {
                // For non-literals, return a dummy or error?
                // For now, return a string representation
                Expression::Literal { value: LiteralValue::String(format!("{:?}", val)), span }
            }
        }
    }

    fn resolve_module(&self, path: &[String]) -> Option<PathBuf> {
        if path.is_empty() { return None; }
        
        let mut base = self.base_dir.clone();
        let mut segments = path.iter();
        let first = segments.next()?;
        
        if first == "package" {
             // Search for 'library' relative to base
             let mut current = base.clone();
             let mut found = false;
             for _ in 0..5 {
                 if current.join("library").exists() {
                     base = current.join("library");
                     found = true;
                     break;
                 }
                 if let Some(p) = current.parent() {
                     current = p.to_path_buf();
                 } else {
                     break;
                 }
             }
             if !found {
                 // Fallback to ../../library (assuming running from vcc.exe)
                 // This handles the specific case of vcc.exe being run from project root,
                 // and base_dir being binary/vcc.
                 // ../../library should resolve to projects/valkyrie-vk/library
                 // BUT `base_dir` is absolute path?
                 // If absolute, `parent` works.
             }
        } else if first == "binary" {
             let mut current = base.clone();
             let mut found = false;
             for _ in 0..5 {
                 if current.join("binary").exists() {
                     base = current.join("binary");
                     found = true;
                     break;
                 }
                 if let Some(p) = current.parent() {
                     current = p.to_path_buf();
                 } else {
                     break;
                 }
             }
        } else if first == "std" {
             // Search for valkyrie-std
             let mut current = base.clone();
             let mut found = false;
             for _ in 0..5 {
                 // println!("Checking for valkyrie-std in {:?}", current);
                 if current.join("valkyrie-std").exists() {
                     base = current.join("valkyrie-std").join("library");
                     found = true;
                     break;
                 }
                 if let Some(p) = current.parent() {
                     current = p.to_path_buf();
                 } else {
                     break;
                 }
             }
             if !found {
                 eprintln!("Could not find valkyrie-std starting from {:?}", self.base_dir);
             }
        } else {
             base = base.join(first);
        }
        
        for seg in segments {
            base = base.join(seg);
        }
        
        let p1 = base.with_extension("vk");
        if p1.exists() { 
            if let Ok(canon) = p1.canonicalize() {
                return Some(canon);
            }
            return Some(p1); 
        }
        
        let p2 = base.join("index.vk");
        if p2.exists() { 
            if let Ok(canon) = p2.canonicalize() {
                return Some(canon);
            }
            return Some(p2); 
        }
        
        // Also try library/seg/index.vk style if we are in root?
        // But we handled base already.
        
        None
    }
}
