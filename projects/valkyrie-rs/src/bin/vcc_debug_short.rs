use std::{env, path::PathBuf};
use valkyrie_rs::compile_and_run_program;

#[tokio::main]
async fn main() {
    // Embed the Valkyrie source code
    const SOURCE: &str = include_str!("../../../valkyrie-vk/binary/vcc_debug_short.vk");

    // Collect arguments, skipping the first one
    let args: Vec<String> = env::args().skip(1).collect();

    // Use current directory as base path
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // We need to resolve the path relative to the crate root or workspace root.
    // If running from projects/valkyrie-rs:
    let base_path = if current_dir.ends_with("valkyrie-rs") {
        current_dir.parent().unwrap().join("valkyrie-vk/binary")
    }
    else {
        // Fallback or assume we are at workspace root
        current_dir.join("projects/valkyrie-vk/binary")
    };

    match compile_and_run_program(SOURCE, Some(base_path), args).await {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Runtime Error: {}", err);
            std::process::exit(1);
        }
    }
}
