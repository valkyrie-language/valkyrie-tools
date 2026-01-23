use crate::{
    ast::{Expression, LiteralValue, MatchPattern, ProgramNode, Statement, TypeExpression},
    hir::{HirExpression, HirParam, HirProgram, HirStatement, HirType},
};

pub struct AstLowering {}

impl Default for AstLowering {
    fn default() -> Self {
        Self::new()
    }
}

impl AstLowering {
    pub fn new() -> Self {
        Self {}
    }

    pub fn lower_program(&mut self, program: ProgramNode) -> HirProgram {
        let mut statements = Vec::new();
        for stmt in program.statements {
            statements.push(self.lower_statement(stmt));
        }
        HirProgram { statements }
    }

    fn lower_statement(&mut self, stmt: Statement) -> HirStatement {
        match stmt {
            Statement::Namespace { path, span } => HirStatement::Namespace { path, span },
            Statement::Using { path, span } => HirStatement::Using { path, span },
            Statement::Block { statements, span } => {
                let mut hir_stmts = Vec::new();
                for s in statements {
                    hir_stmts.push(self.lower_statement(s));
                }
                HirStatement::Block { statements: hir_stmts, span }
            }
            Statement::Let { name, type_hint, value, span } => {
                let ty = if let Some(hint) = type_hint { self.lower_type(hint) } else { HirType::Unknown };
                let hir_value = self.lower_expression(value);
                HirStatement::Let { name, binding: None, ty, value: hir_value, span }
            }
            Statement::If { condition, then_branch, else_branch, span } => {
                let cond = self.lower_expression(condition);
                let then_b = Box::new(self.lower_statement(*then_branch));
                let else_b = else_branch.map(|b| Box::new(self.lower_statement(*b)));
                HirStatement::If { condition: cond, then_branch: then_b, else_branch: else_b, span }
            }
            Statement::Return { value, span } => {
                let val = value.map(|v| self.lower_expression(v));
                HirStatement::Return { value: val, span }
            }
            Statement::While { condition, body, span } => {
                let cond = self.lower_expression(condition);
                let b = Box::new(self.lower_statement(*body));
                HirStatement::While { condition: cond, body: b, span }
            }
            Statement::Class { name, parents, traits, fields, methods, span, .. } => {
                let hir_parents = parents.into_iter().map(|(alias, p)| (alias, self.lower_type(p))).collect();
                let hir_traits = traits.into_iter().map(|t| self.lower_type(t)).collect();
                let mut hir_fields = Vec::new();
                for (fname, ftype) in fields {
                    hir_fields.push((fname, self.lower_type(ftype)));
                }
                let mut hir_methods = Vec::new();
                for m in methods {
                    hir_methods.push(self.lower_statement(m));
                }
                HirStatement::Class {
                    name,
                    binding: None,
                    parents: hir_parents,
                    traits: hir_traits,
                    fields: hir_fields,
                    methods: hir_methods,
                    span,
                }
            }
            Statement::Trait { name, parents, methods, span, .. } => {
                let hir_parents = parents.into_iter().map(|p| self.lower_type(p)).collect();
                let mut hir_methods = Vec::new();
                for m in methods {
                    hir_methods.push(self.lower_statement(m));
                }
                HirStatement::Trait { name, binding: None, parents: hir_parents, methods: hir_methods, span }
            }
            Statement::Imply { target, trait_target, methods, span, .. } => {
                let hir_target = self.lower_type(target);
                let hir_trait = trait_target.map(|t| self.lower_type(t));
                let mut hir_methods = Vec::new();
                for m in methods {
                    hir_methods.push(self.lower_statement(m));
                }
                HirStatement::Imply { target: hir_target, trait_target: hir_trait, methods: hir_methods, span }
            }
            Statement::Function { name, params, return_type, body, span, .. } => {
                let mut hir_params = Vec::new();
                for (p_name, p_type) in params {
                    let ty = if let Some(t) = p_type { self.lower_type(t) } else { HirType::Unknown };
                    hir_params.push(HirParam { name: p_name, binding: None, ty });
                }

                let ret_ty = if let Some(rt) = return_type { self.lower_type(rt) } else { HirType::Unknown };

                let hir_body = body.map(|b| Box::new(self.lower_statement(*b)));

                HirStatement::Function { name, binding: None, params: hir_params, return_type: ret_ty, body: hir_body, span }
            }
            Statement::Macro { span, .. } => HirStatement::Block { statements: vec![], span },
            Statement::Annotation { target, .. } => self.lower_statement(*target),
            Statement::Yield { span, .. } => HirStatement::Block { statements: vec![], span },
            Statement::Expression { expression, span } => {
                HirStatement::Expression { expression: self.lower_expression(expression), span }
            }
        }
    }

    fn lower_expression(&mut self, expr: Expression) -> HirExpression {
        match expr {
            Expression::Identifier { name, span } => {
                HirExpression::Identifier { name, binding: None, ty: HirType::Unknown, span }
            }
            Expression::Literal { value, span } => {
                let ty = match value {
                    LiteralValue::Int(_) => HirType::Int,
                    LiteralValue::Float(_) => HirType::Float,
                    LiteralValue::Bool(_) => HirType::Bool,
                    LiteralValue::String(_) => HirType::String,
                };
                HirExpression::Literal { value, ty, span }
            }
            Expression::BinaryOp { left, op, right, span } => {
                let l = Box::new(self.lower_expression(*left));
                let r = Box::new(self.lower_expression(*right));
                HirExpression::BinaryOp { left: l, op, right: r, ty: HirType::Unknown, span }
            }
            Expression::UnaryOp { op, operand, span } => {
                let o = Box::new(self.lower_expression(*operand));
                HirExpression::UnaryOp { op, operand: o, ty: HirType::Unknown, span }
            }
            Expression::Call { callee, args, span } => {
                let c = Box::new(self.lower_expression(*callee));
                let mut a = Vec::new();
                for arg in args {
                    a.push(self.lower_expression(arg));
                }
                HirExpression::Call { callee: c, args: a, ty: HirType::Unknown, span }
            }
            Expression::Get { object, name, span } => {
                let obj = Box::new(self.lower_expression(*object));
                HirExpression::Get { object: obj, name, ty: HirType::Unknown, span }
            }
            Expression::New { class, args, closure, span } => {
                let hir_class = self.lower_type(class);
                let mut hir_args = Vec::new();
                for arg in args {
                    hir_args.push(self.lower_expression(arg));
                }
                let hir_closure = closure.map(|c| Box::new(self.lower_statement(*c)));
                HirExpression::New { class: hir_class, args: hir_args, closure: hir_closure, ty: HirType::Unknown, span }
            }
            Expression::Match { scrutinee, arms, else_arm, span } => {
                let scrutinee = Box::new(self.lower_expression(*scrutinee));
                let mut lowered_arms: Vec<(MatchPattern, HirExpression)> = Vec::new();
                for (pat, expr) in arms {
                    lowered_arms.push((pat, self.lower_expression(expr)));
                }
                let else_arm = else_arm.map(|e| Box::new(self.lower_expression(*e)));
                HirExpression::Match { scrutinee, arms: lowered_arms, else_arm, ty: HirType::Unknown, span }
            }
            Expression::Await { future, .. } => self.lower_expression(*future),
            Expression::MacroCall { name, span, .. } => {
                // If a macro call reaches here, it means it wasn't expanded.
                // This is likely an error for built-in macros, but for user-defined ones we might not have expanded them yet.
                // However, user requested "correct logic", meaning type inference relies on expansion.
                // If it's not expanded, we can't infer its type "correctly" without "cheating".
                // So we should probably treat it as Unknown or Any.
                // Or panic if we assume all valid macros are expanded.

                // Let's assume for now that if it's here, it's a runtime macro or unexpanded one.
                // We return Unknown type, and let runtime handle it (or fail).
                // But we NO LONGER cheat for stringify.
                HirExpression::Literal { value: LiteralValue::Int(0), ty: HirType::Unknown, span }
            }
            Expression::Lambda { params, body, is_async: _is_async, is_generator: _is_generator, span } => {
                let mut hir_params = Vec::new();
                for (p_name, p_type) in params {
                    let ty = if let Some(t) = p_type { self.lower_type(t) } else { HirType::Unknown };
                    hir_params.push(HirParam { name: p_name, binding: None, ty });
                }
                let hir_body = Box::new(self.lower_statement(*body));
                HirExpression::Lambda { params: hir_params, body: hir_body, ty: HirType::Unknown, span }
            }
            Expression::List { elements, span } => {
                let mut hir_elements = Vec::new();
                for e in elements {
                    hir_elements.push(self.lower_expression(e));
                }
                HirExpression::List { elements: hir_elements, ty: HirType::Unknown, span }
            }
            Expression::Index { target, index, is_zero_based, span } => {
                let t = Box::new(self.lower_expression(*target));
                let i = Box::new(self.lower_expression(*index));
                HirExpression::Index { target: t, index: i, is_zero_based, ty: HirType::Unknown, span }
            }
            Expression::Block { body, span } => {
                let b = Box::new(self.lower_statement(*body));
                HirExpression::Block { body: b, ty: HirType::Unknown, span }
            }
        }
    }

    fn lower_type(&mut self, ty: TypeExpression) -> HirType {
        match ty {
            TypeExpression::Name { name, .. } => match name.as_str() {
                "int" | "i64" => HirType::Int,
                "float" | "f64" => HirType::Float,
                "bool" => HirType::Bool,
                "string" => HirType::String,
                "void" => HirType::Void,
                _ => HirType::Unknown,
            },
            TypeExpression::Generic { .. } => HirType::Unknown,
        }
    }
}
