use valkyrie_rs::parse_program;
use valkyrie_rs::hir::lowering::HirLowering;
use valkyrie_rs::mir::wasm::WasmCompiler;
use wasmtime::{Engine, Linker, Module, Store};
use valkyrie_rs::hir::optimizer::optimize;

#[test]
fn test_wasm_simple_arithmetic() {
    let src = "return 1 + 2 * 3;";
    
    // 1. Compile to MIR
    let ast = parse_program(src).unwrap();
    // We need optimization to remove complex structures if any, though simple arithmetic works
    let optimized = optimize(ast);
    
    let lower = HirLowering::new();
    let (mir, _, _) = lower.compile(optimized);

    // 2. Compile to WASM
    let compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&mir);

    // 3. Verify with Wasmtime
    let engine = Engine::default();
    match Module::new(&engine, &wasm_bytes) {
        Ok(module) => {
            let mut store = Store::new(&engine, ());
            let mut linker = Linker::new(&engine);
            
            // Define imports if needed (though simple arithmetic might not use them, 
            // but my updated compiler adds import section, so instantiation requires it)
            linker.func_wrap("env", "print", |v: i64| {
                println!("WASM Print: {}", v);
            }).unwrap();

            let instance = linker.instantiate(&mut store, &module).expect("Failed to instantiate");
            let main = instance.get_typed_func::<(), i64>(&mut store, "main").expect("Failed to get main");
            let result = main.call(&mut store, ()).expect("Failed to call main");
            assert_eq!(result, 7);
        },
        Err(e) => {
            panic!("Failed to create WASM module: {}\nBytes: {:?}", e, wasm_bytes);
        }
    }
}

#[test]
fn test_wasm_locals() {
    let src = "
    let x = 10;
    let y = 20;
    return x + y;
    ";
    
    // 1. Compile to MIR
    let ast = parse_program(src).unwrap();
    let optimized = optimize(ast);
    let lower = HirLowering::new();
    let (mir, _, _) = lower.compile(optimized);

    // 2. Compile to WASM
    let compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&mir);

    // 3. Verify with Wasmtime
    let engine = Engine::default();
    match Module::new(&engine, &wasm_bytes) {
        Ok(module) => {
            let mut store = Store::new(&engine, ());
            let mut linker = Linker::new(&engine);
            
            linker.func_wrap("env", "print", |v: i64| {
                println!("WASM Print: {}", v);
            }).unwrap();

            let instance = linker.instantiate(&mut store, &module).expect("Failed to instantiate");
            let main = instance.get_typed_func::<(), i64>(&mut store, "main").expect("Failed to get main");
            let result = main.call(&mut store, ()).expect("Failed to call main");
            assert_eq!(result, 30);
        },
        Err(e) => {
            panic!("Failed to create WASM module: {}\nBytes: {:?}", e, wasm_bytes);
        }
    }
}
