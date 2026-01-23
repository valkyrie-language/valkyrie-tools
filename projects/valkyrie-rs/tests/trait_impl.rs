use valkyrie_rs::compile_and_run;
use valkyrie_rs::mir::value::RuntimeValue;

async fn run(src: &str) -> RuntimeValue {
    compile_and_run(src, None, vec![]).await.unwrap()
}

#[tokio::test]
async fn test_trait_impl_syntax() {
    let src = r#"
    trait MyTrait {
        micro foo(self);
    }
    
    class MyClass {
        value: i32
    }
    
    imply MyClass : MyTrait {
        micro foo(self) {
            return self.value;
        }
    }
    
    let c = new MyClass();
    c.value = 42;
    c.foo()
    "#;
    match run(src).await {
        RuntimeValue::Int(v) => assert_eq!(v, 42),
        _ => panic!("Expected Int(42)"),
    }
}

#[tokio::test]
async fn test_iterator_trait() {
    let src = r#"
    trait Iterator {
        micro next(self);
    }
    
    class Counter {
        current: i32,
        limit: i32
    }
    
    imply Counter : Iterator {
        micro next(self) {
            if self.current < self.limit {
                let v = self.current;
                self.current = self.current + 1;
                v // Implicit return
            } else {
                -1 // Implicit return
            }
        }
    }
    
    let c = new Counter();
    c.current = 0;
    c.limit = 3;
    
    let sum = 0;
    let loop_cond = true;
    while loop_cond {
        let n = c.next();
        if n == -1 {
            loop_cond = false;
        } else {
            sum = sum + n;
        }
    }
    sum
    "#;
    match run(src).await {
        RuntimeValue::Int(v) => assert_eq!(v, 3), // 0 + 1 + 2 = 3
        _ => panic!("Expected Int(3)"),
    }
}
