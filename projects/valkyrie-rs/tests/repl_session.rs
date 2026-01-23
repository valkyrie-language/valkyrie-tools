use valkyrie_rs::session::ValkyrieSession;
use valkyrie_rs::mir::value::RuntimeValue;

#[tokio::test]
async fn test_repl_session() {
    let mut session = ValkyrieSession::new(vec![], None);

    // 1. Define variable
    session.eval("let x = 10;").await.unwrap();

    // 2. Use variable
    let result = session.eval("x + 5").await.unwrap();
    match result {
        RuntimeValue::Int(v) => assert_eq!(v, 15),
        _ => panic!("Expected Int(15)"),
    }

    // 3. Define function
    session.eval("micro add(a, b) { return a + b; }").await.unwrap();

    // 4. Call function
    let result = session.eval("add(x, 20)").await.unwrap();
    match result {
        RuntimeValue::Int(v) => assert_eq!(v, 30),
        _ => panic!("Expected Int(30)"),
    }
    
    // 5. Define class
    session.eval("class Point { x: i32, y: i32 micro constructor(self, x, y) { self.x = x; self.y = y; } }").await.unwrap();
    
    // 6. Use class
    let result = session.eval("let p = new Point(1, 2); p.x").await.unwrap();
     match result {
        RuntimeValue::Int(v) => assert_eq!(v, 1),
        _ => panic!("Expected Int(1)"),
    }
}
