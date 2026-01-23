use crate::ast::{parser::ParseError, Expression, LiteralValue, Position, ProgramNode, Span, Statement};
use std::path::PathBuf;
use std::collections::HashSet;

pub struct MacroExpander {
    base_dir: PathBuf,
    loaded_modules: HashSet<PathBuf>,
    depth: usize,
}

impl MacroExpander {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            loaded_modules: HashSet::new(),
            depth: 0,
        }
    }

    pub fn expand_program(&mut self, program: ProgramNode) -> ProgramNode {
        self.depth += 1;
        if self.depth > 100 {
            panic!("Stack overflow detected in macro expansion!");
        }
        let mut statements = Vec::new();
        for stmt in program.statements {
            statements.extend(self.expand_statement(stmt));
        }
        self.depth -= 1;
        ProgramNode { statements, span: program.span }
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
                let expanded_args = args.into_iter().map(|a| self.expand_expression(a)).collect();
                let expanded_target = self.expand_block(*target);
                vec![Statement::Annotation { name, args: expanded_args, target: Box::new(expanded_target), span }]
            }
            _ => vec![stmt],
        }
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

    fn expand_expression(&mut self, expr: Expression) -> Expression {
        match expr {
            Expression::MacroCall { name, args, span } => {
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
                        ) = (arg1, arg2)
                        {
                            let s1 = v1.to_string();
                            let s2 = v2.to_string();
                            let combined = format!("{}{}", s1, s2);
                            if let Ok(i) = combined.parse::<i64>() {
                                return Expression::Literal { value: LiteralValue::Int(i), span };
                            }
                        }
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
            _ => expr,
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
