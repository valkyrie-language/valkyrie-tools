use crate::{
    ast::{BinaryOperator, Expression, LiteralValue, ProgramNode, Statement},
    mir::value::RuntimeValue,
};
use std::collections::HashMap;

pub fn optimize(program: ProgramNode) -> ProgramNode {
    // Default optimization without known values
    partial_eval(program, HashMap::new())
}

pub fn partial_eval(program: ProgramNode, known_values: HashMap<String, RuntimeValue>) -> ProgramNode {
    let mut evaluator = PartialEvaluator::new(known_values);
    evaluator.eval_program(program)
}

struct PartialEvaluator {
    known_values: HashMap<String, RuntimeValue>,
}

impl PartialEvaluator {
    fn new(known_values: HashMap<String, RuntimeValue>) -> Self {
        Self { known_values }
    }

    fn eval_program(&mut self, program: ProgramNode) -> ProgramNode {
        let mut statements = Vec::new();
        for stmt in program.statements {
            statements.push(self.eval_statement(stmt));
        }
        ProgramNode { statements, span: program.span }
    }

    fn eval_statement(&mut self, stmt: Statement) -> Statement {
        match stmt {
            Statement::Block { statements, span } => {
                let mut optimized_stmts = Vec::new();
                // TODO: Handle scoping if we want to support local variable folding
                for s in statements {
                    optimized_stmts.push(self.eval_statement(s));
                }
                Statement::Block { statements: optimized_stmts, span }
            }
            Statement::Let { name, type_hint, value, span } => {
                let optimized_value = self.eval_expression(value);
                // If the value is a literal, we could add it to known_values for subsequent statements in this block
                // But we need to handle scope properly. For now, we only use the initial known_values.
                Statement::Let { name, type_hint, value: optimized_value, span }
            }
            Statement::If { condition, then_branch, else_branch, span } => {
                let cond = self.eval_expression(condition);
                // Constant folding for IF
                if let Expression::Literal { value: LiteralValue::Bool(b), .. } = &cond {
                    if *b {
                        return self.eval_statement(*then_branch);
                    }
                    else {
                        if let Some(else_b) = else_branch {
                            return self.eval_statement(*else_b);
                        }
                        else {
                            // Empty block to preserve structure if needed, or Nop
                            return Statement::Block { statements: vec![], span };
                        }
                    }
                }

                let then_b = Box::new(self.eval_statement(*then_branch));
                let else_b = else_branch.map(|b| Box::new(self.eval_statement(*b)));
                Statement::If { condition: cond, then_branch: then_b, else_branch: else_b, span }
            }
            Statement::Return { value, span } => Statement::Return { value: value.map(|v| self.eval_expression(v)), span },
            Statement::Expression { expression, span } => {
                Statement::Expression { expression: self.eval_expression(expression), span }
            }
            Statement::Function { name, generics, params, return_type, body, is_async, is_generator, span } => {
                let optimized_body = body.map(|b| Box::new(self.eval_statement(*b)));
                Statement::Function { name, generics, params, return_type, body: optimized_body, is_async, is_generator, span }
            }
            Statement::Annotation { name, args, target, span } => {
                let optimized_target = Box::new(self.eval_statement(*target));
                Statement::Annotation { name, args, target: optimized_target, span }
            }
            // Handle other statements recursively if needed, otherwise return as is
            _ => stmt,
        }
    }

    fn eval_expression(&mut self, expr: Expression) -> Expression {
        match expr {
            Expression::Identifier { name, span } => {
                if let Some(val) = self.known_values.get(&name) {
                    // Replace identifier with known value literal
                    match val {
                        RuntimeValue::Int(v) => Expression::Literal { value: LiteralValue::Int(*v), span },
                        RuntimeValue::Float(v) => Expression::Literal { value: LiteralValue::Float(v.to_bits()), span },
                        RuntimeValue::Bool(v) => Expression::Literal { value: LiteralValue::Bool(*v), span },
                        RuntimeValue::String(v) => Expression::Literal { value: LiteralValue::String(v.clone()), span },
                        _ => Expression::Identifier { name, span },
                    }
                }
                else {
                    Expression::Identifier { name, span }
                }
            }
            Expression::BinaryOp { left, op, right, span } => {
                let left_opt = self.eval_expression(*left);
                let right_opt = self.eval_expression(*right);

                // Constant Folding
                if let (Expression::Literal { value: l_val, .. }, Expression::Literal { value: r_val, .. }) =
                    (&left_opt, &right_opt)
                {
                    match (l_val, r_val) {
                        (LiteralValue::Int(l), LiteralValue::Int(r)) => match op {
                            BinaryOperator::Add => return Expression::Literal { value: LiteralValue::Int(l + r), span },
                            BinaryOperator::Sub => return Expression::Literal { value: LiteralValue::Int(l - r), span },
                            BinaryOperator::Mul => return Expression::Literal { value: LiteralValue::Int(l * r), span },
                            BinaryOperator::Div => return Expression::Literal { value: LiteralValue::Int(l / r), span },
                            BinaryOperator::Equal => return Expression::Literal { value: LiteralValue::Bool(l == r), span },
                            BinaryOperator::NotEqual => return Expression::Literal { value: LiteralValue::Bool(l != r), span },
                            BinaryOperator::Less => return Expression::Literal { value: LiteralValue::Bool(l < r), span },
                            BinaryOperator::LessEqual => {
                                return Expression::Literal { value: LiteralValue::Bool(l <= r), span }
                            }
                            BinaryOperator::Greater => return Expression::Literal { value: LiteralValue::Bool(l > r), span },
                            BinaryOperator::GreaterEqual => {
                                return Expression::Literal { value: LiteralValue::Bool(l >= r), span }
                            }
                            _ => {}
                        },
                        // Add more types if needed
                        _ => {}
                    }
                }

                Expression::BinaryOp { left: Box::new(left_opt), op, right: Box::new(right_opt), span }
            }
            Expression::Call { callee, args, span } => {
                let callee_opt = self.eval_expression(*callee);
                let args_opt = args.into_iter().map(|a| self.eval_expression(a)).collect();
                Expression::Call { callee: Box::new(callee_opt), args: args_opt, span }
            }
            Expression::Match { scrutinee, arms, else_arm, span } => {
                let s = self.eval_expression(*scrutinee);

                // If scrutinee is constant, we can try to match at compile time
                if let Expression::Literal { value: s_val, .. } = &s {
                    for (pat, arm_expr) in &arms {
                        if self.pattern_matches(pat, s_val) {
                            return self.eval_expression(arm_expr.clone());
                        }
                    }
                    if let Some(e) = else_arm {
                        return self.eval_expression(*e.clone());
                    }
                }

                let mut optimized_arms = Vec::new();
                for (pat, expr) in arms {
                    optimized_arms.push((pat, self.eval_expression(expr)));
                }
                let else_opt = else_arm.map(|e| Box::new(self.eval_expression(*e)));
                Expression::Match { scrutinee: Box::new(s), arms: optimized_arms, else_arm: else_opt, span }
            }
            Expression::MacroCall { name, args, span } => {
                if name == "stringify" {
                    if let Some(arg) = args.first() {
                        // Return string representation of the UN-optimized argument
                        let s = format!("{:?}", arg);
                        return Expression::Literal { value: LiteralValue::String(s), span };
                    }
                }
                else if name == "location.line_number" {
                    // Lexer usually 0-indexed or 1-indexed. Test expects 2.
                    // If span.start.line is 0-indexed, and test expects 2 (line 3 in file), maybe it's +1?
                    // Let's assume span.start.line is correct from parser.
                    return Expression::Literal { value: LiteralValue::Int(span.start.line as i64), span };
                }

                let optimized_args = args.into_iter().map(|a| self.eval_expression(a)).collect();
                Expression::MacroCall { name, args: optimized_args, span }
            }
            Expression::New { class, args, closure, span } => {
                let optimized_args = args.into_iter().map(|a| self.eval_expression(a)).collect();
                let optimized_closure = closure.map(|c| Box::new(self.eval_statement(*c)));
                Expression::New { class, args: optimized_args, closure: optimized_closure, span }
            }
            Expression::Await { future, span } => Expression::Await { future: Box::new(self.eval_expression(*future)), span },
            Expression::Get { object, name, span } => {
                Expression::Get { object: Box::new(self.eval_expression(*object)), name, span }
            }
            // Recursively optimize other expressions
            _ => expr,
        }
    }

    fn pattern_matches(&self, pat: &crate::ast::MatchPattern, val: &LiteralValue) -> bool {
        match pat {
            crate::ast::MatchPattern::Wildcard => true,
            crate::ast::MatchPattern::Literal(p_val) => p_val == val,
        }
    }
}
