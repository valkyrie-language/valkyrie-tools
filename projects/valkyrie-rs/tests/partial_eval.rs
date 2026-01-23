use std::collections::HashMap;
use valkyrie_rs::{
    ast::{Expression, LiteralValue, Position, ProgramNode, Span, Statement},
    hir::optimizer::partial_eval,
    mir::RuntimeValue,
};

fn dummy_span() -> Span {
    Span { start: Position { line: 0, column: 0, offset: 0 }, end: Position { line: 0, column: 0, offset: 0 } }
}

#[test]
fn test_partial_eval_identifier() {
    // Program: return x;
    // Known: x = 42
    // Result: return 42;

    let span = dummy_span();
    let stmt = Statement::Return { value: Some(Expression::Identifier { name: "x".to_string(), span }), span };
    let program = ProgramNode { statements: vec![stmt], span };

    let mut known = HashMap::new();
    known.insert("x".to_string(), RuntimeValue::Int(42));

    let optimized = partial_eval(program, known);

    if let Statement::Return { value: Some(Expression::Literal { value: LiteralValue::Int(v), .. }), .. } =
        &optimized.statements[0]
    {
        assert_eq!(*v, 42);
    }
    else {
        panic!("Expected literal 42, got {:?}", optimized.statements[0]);
    }
}

#[test]
fn test_partial_eval_if_constant() {
    // Program: if x { return 1; } else { return 2; }
    // Known: x = true
    // Result: return 1;

    let span = dummy_span();
    let stmt = Statement::If {
        condition: Expression::Identifier { name: "x".to_string(), span },
        then_branch: Box::new(Statement::Return {
            value: Some(Expression::Literal { value: LiteralValue::Int(1), span }),
            span,
        }),
        else_branch: Some(Box::new(Statement::Return {
            value: Some(Expression::Literal { value: LiteralValue::Int(2), span }),
            span,
        })),
        span,
    };
    let program = ProgramNode { statements: vec![stmt], span };

    let mut known = HashMap::new();
    known.insert("x".to_string(), RuntimeValue::Bool(true));

    let optimized = partial_eval(program, known);

    if let Statement::Return { value: Some(Expression::Literal { value: LiteralValue::Int(v), .. }), .. } =
        &optimized.statements[0]
    {
        assert_eq!(*v, 1);
    }
    else {
        panic!("Expected literal 1, got {:?}", optimized.statements[0]);
    }
}

#[test]
fn test_partial_eval_match_constant() {
    // Program: match x { case 1: return 10; case 2: return 20; }
    // Known: x = 2
    // Result: return 20;

    let span = dummy_span();
    use valkyrie_rs::ast::MatchPattern;

    let stmt = Statement::Expression {
        expression: Expression::Match {
            scrutinee: Box::new(Expression::Identifier { name: "x".to_string(), span }),
            arms: vec![
                (MatchPattern::Literal(LiteralValue::Int(1)), Expression::Literal { value: LiteralValue::Int(10), span }),
                (MatchPattern::Literal(LiteralValue::Int(2)), Expression::Literal { value: LiteralValue::Int(20), span }),
            ],
            else_arm: None,
            span,
        },
        span,
    };
    let program = ProgramNode { statements: vec![stmt], span };

    let mut known = HashMap::new();
    known.insert("x".to_string(), RuntimeValue::Int(2));

    let optimized = partial_eval(program, known);

    // Result should be Expression Statement containing Literal 20
    if let Statement::Expression { expression: Expression::Literal { value: LiteralValue::Int(v), .. }, .. } =
        &optimized.statements[0]
    {
        assert_eq!(*v, 20);
    }
    else {
        panic!("Expected literal 20, got {:?}", optimized.statements[0]);
    }
}
