use std::{env, path::PathBuf};
use valkyrie_rs::compile_and_run_program;

fn main() {
    let builder = std::thread::Builder::new().stack_size(32 * 1024 * 1024); // 32MB
    let handler = builder.spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async_main());
    }).unwrap();

    handler.join().unwrap();
}

async fn async_main() {
    const SOURCE: &str = include_str!("../../../valkyrie-vk/binary/vcc/main.vk");
    let args: Vec<String> = env::args().skip(1).collect();
    // Assume we run from project root, or we need to locate the source tree.
    // Since main.vk is embedded, we fake its location to be where it is in the source tree
    // so that relative @includes work.
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // We need to resolve the path relative to the crate root or workspace root.
    // If running from projects/valkyrie-rs:
    let base_path = if current_dir.ends_with("valkyrie-rs") {
        current_dir.parent().unwrap().join("valkyrie-vk/binary/vcc")
    } else if current_dir.ends_with("valkyrie-vk") {
        current_dir.join("binary/vcc")
    } else {
        // Fallback or assume we are at workspace root
        current_dir.join("projects/valkyrie-vk/binary/vcc")
    };

    match compile_and_run_program(SOURCE, Some(base_path), args).await {
        Ok(_) => {} // Program executed successfully (exit code usually handled inside or implicit 0)
        Err(err) => {
            eprintln!("Runtime Error: {}", err);
            std::process::exit(1);
        }
    }
}
