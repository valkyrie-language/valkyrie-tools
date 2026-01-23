use valkyrie_rs::{
    build_hir,
    hir::{HirExpression, HirStatement},
};

#[test]
fn hir_resolves_shadowing_in_nested_blocks() {
    let src = r#"
        let x = 1;
        {
            let x = 2;
            x;
        }
        x;
    "#;

    let hir = build_hir(src).unwrap();

    let HirStatement::Let { binding: Some(outer_id), .. } = &hir.statements[0]
    else {
        panic!("expected outer let");
    };

    let HirStatement::Block { statements, .. } = &hir.statements[1]
    else {
        panic!("expected block");
    };

    let HirStatement::Let { binding: Some(inner_id), .. } = &statements[0]
    else {
        panic!("expected inner let");
    };

    let HirStatement::Expression { expression, .. } = &statements[1]
    else {
        panic!("expected inner expr");
    };
    let HirExpression::Identifier { binding: Some(inner_ref), .. } = expression
    else {
        panic!("expected inner identifier");
    };

    let HirStatement::Expression { expression, .. } = &hir.statements[2]
    else {
        panic!("expected outer expr");
    };
    let HirExpression::Identifier { binding: Some(outer_ref), .. } = expression
    else {
        panic!("expected outer identifier");
    };

    assert_eq!(inner_ref, inner_id);
    assert_eq!(outer_ref, outer_id);
    assert_ne!(inner_id, outer_id);
}

#[test]
fn hir_reports_simple_type_mismatch() {
    let err = build_hir("let x: bool = 1;").unwrap_err();
    assert!(err.contains("type mismatch") || err.contains("Bool"));
}
