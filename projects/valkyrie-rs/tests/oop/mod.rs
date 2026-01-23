use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn eval_class_definition() {
    let src = r#"
    class Point {
        x: float,
        y: float,
        micro distance(self, other: Point) -> float {
            0.0;
        }
    }
    let p = 1;
    p;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}

#[tokio::test]
async fn eval_trait_definition() {
    let src = r#"
    trait Drawable {
        micro draw(self);
        micro update(self) {
            0;
        }
    }
    let t = 1;
    t;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}

#[tokio::test]
async fn eval_class_instantiation_and_method() {
    let src = r#"
    class Point { x: i32, y: i32 }
    imply Point {
        micro get_x(self) { return self.x; }
        micro set_x(self, v) { self.x = v; }
    }
    let p = new Point();
    p.set_x(10);
    p.get_x();
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 10);
}

#[tokio::test]
async fn eval_polymorphism() {
    let src = r#"
    trait Drawable { micro draw(self); }
    
    class Circle { r: i32 }
    imply Circle {
        micro draw(self) { return 1; }
    }
    
    class Box { w: i32 }
    imply Box {
        micro draw(self) { return 2; }
    }
    
    micro render(d) {
        return d.draw();
    }
    
    let c = new Circle();
    let b = new Box();
    
    let r1 = render(c);
    let r2 = render(b);
    
    if r1 == 1 {
        if r2 == 2 {
            1;
        } else { 0; }
    } else { 0; }
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}

#[tokio::test]
async fn eval_constructor() {
    let src = r#"
    class Counter { val: i32 }
    imply Counter {
        micro new(self, v) {
            self.val = v;
        }
        micro get(self) { return self.val; }
    }
    let c = new Counter(42);
    c.get();
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 42);
}

#[tokio::test]
async fn eval_trait_inheritance() {
    let src = r#"
    trait Base {
        micro base_method(self);
    }
    
    trait Derived : Base {
        micro derived_method(self);
    }
    
    class Impl { }
    
    imply Impl : Derived {
        micro base_method(self) { return 1; }
        micro derived_method(self) { return 2; }
    }
    
    let i = new Impl();
    if i.base_method() == 1 {
        if i.derived_method() == 2 {
            1;
        } else { 0; }
    } else { 0; }
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}

#[tokio::test]
async fn eval_static_members() {
    let src = r#"
    class Utils { }
    imply Utils {
        micro add(a, b) {
            return a + b;
        }
        
        micro instance_method(self) {
            return 100;
        }
    }
    
    let sum = Utils.add(10, 20);
    
    let u = new Utils();
    let val = u.instance_method();
    
    if sum == 30 {
        if val == 100 {
            1;
        } else { 0; }
    } else { 0; }
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}

#[tokio::test]
async fn eval_constructor_overloading() {
    let src = r#"
    class Vector { x: i32, y: i32 }
    imply Vector {
        micro new(self) {
            self.x = 0;
            self.y = 0;
        }
        
        micro new(self, v) {
            self.x = v;
            self.y = v;
        }
        
        micro new(self, x, y) {
            self.x = x;
            self.y = y;
        }
    }
    
    let v1 = new Vector();
    let v2 = new Vector(5);
    let v3 = new Vector(1, 2);
    
    if v1.x == 0 {
        if v2.x == 5 {
            if v3.y == 2 {
                1;
            } else { 0; }
        } else { 0; }
    } else { 0; }
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}
