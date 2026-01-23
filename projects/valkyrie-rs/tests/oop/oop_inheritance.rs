use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn eval_multiple_inheritance_mro() {
    // Diamond problem
    //    A
    //   / \
    //  B   C
    //   \ /
    //    D
    // D(B, C) -> MRO: [D, B, C, A]
    // Python 3 MRO for diamond problem
    
    let src = r#"
    class A { }
    imply A {
        micro foo(self) { return 1; }
    }
    
    class B (A) { }
    imply B {
        micro foo(self) { return 2; }
    }
    
    class C (A) { }
    imply C {
        micro foo(self) { return 3; }
    }
    
    class D (B, C) { }
    
    let d = new D();
    d.foo();
    "#;
    // B overrides A, C overrides A. D inherits B then C.
    // MRO(D) = [D, B, C, A]
    // So d.foo() should call B.foo() -> 2
    assert_eq!(compile_and_run(src).await.unwrap(), 2);
}

#[tokio::test]
async fn eval_multiple_inheritance_fields() {
    let src = r#"
    class Point2D { x: i32, y: i32 }
    class Color { r: i32, g: i32, b: i32 }
    
    class Pixel (Point2D, Color) { }
    
    let p = new Pixel();
    p.x = 10;
    p.r = 255;
    
    if p.x == 10 {
        if p.r == 255 { 1; } else { 0; }
    } else { 0; }
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}
