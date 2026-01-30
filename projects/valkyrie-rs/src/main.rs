use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tokio::fs;
use valkyrie_rs::{compile_and_run_program, build_hir, parse_program};
use valkyrie_rs::session::ValkyrieSession;
use valkyrie_rs::mir::value::RuntimeValue;
use valkyrie_rs::hir::lowering::HirLowering;
use valkyrie_rs::mir::wasm::WasmCompiler;
use valkyrie_rs::hir::optimizer::optimize;

#[derive(Parser)]
#[command(name = "valkyrie-rs")]
#[command(about = "Valkyrie Programming Language Compiler & Runtime", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate a string of source code
    Eval {
        /// Source code to evaluate
        source: String,
    },
    /// Run a source file
    Run {
        /// Path to the source file
        file: PathBuf,
        /// Arguments passed to the program
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Start a REPL session
    Repl,
    /// Check the code (Static Analysis)
    Check {
        /// Path to the source file
        file: PathBuf,
    },
    /// Compile to WASM
    Wasm {
        /// Path to the source file
        file: PathBuf,
        /// Output file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(8 * 1024 * 1024) // 8 MiB stack size
        .build()
        .unwrap()
        .block_on(async_main());
}

async fn async_main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Eval { source } => match compile_and_run_program(&source, None, vec![]).await {
            Ok(value) => println!("{:?}", value),
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        },
        Commands::Run { file, args } => {
            let file = match std::fs::canonicalize(&file) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to resolve file path {:?}: {}", file, e);
                    std::process::exit(1);
                }
            };
            match fs::read_to_string(&file).await {
                Ok(source) => {
                    let base_path = file.parent().map(|p| p.to_path_buf());
                    match compile_and_run_program(&source, base_path, args).await {
                        Ok(value) => {
                             if value != RuntimeValue::Void {
                                 println!("{:?}", value);
                             }
                        },
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            std::process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Failed to read file {:?}: {}", file, err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Repl => {
            println!("Valkyrie REPL v0.0.1");
            println!("Type 'exit' or Ctrl+C to quit.");
            let mut session = ValkyrieSession::new(vec![], None);
            let mut buffer = String::new();
            
            use std::io::{self, Write};
            
            loop {
                print!(">> ");
                io::stdout().flush().unwrap();
                
                buffer.clear();
                if io::stdin().read_line(&mut buffer).is_err() {
                    break;
                }
                
                let input = buffer.trim();
                if input == "exit" {
                    break;
                }
                if input.is_empty() {
                    continue;
                }
                
                match session.eval(input).await {
                    Ok(value) => {
                        if value != RuntimeValue::Void {
                            println!("{:?}", value);
                        }
                    },
                    Err(err) => println!("Error: {}", err),
                }
            }
        },
        Commands::Check { file } => match fs::read_to_string(&file).await {
            Ok(source) => {
                match build_hir(&source) {
                    Ok(_) => println!("Check passed: {:?}", file),
                    Err(err) => {
                        eprintln!("Check failed: {}", err);
                        std::process::exit(1);
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to read file {:?}: {}", file, err);
                std::process::exit(1);
            }
        },
        Commands::Wasm { file, output } => match fs::read_to_string(&file).await {
            Ok(source) => {
                let ast = parse_program(&source).expect("Parse error");
                let optimized = optimize(ast);
                let lower = HirLowering::new();
                let (mir, _, _) = lower.compile(optimized);
                let compiler = WasmCompiler::new();
                let wasm = compiler.compile(&mir);
                
                let out_path = output.unwrap_or_else(|| file.with_extension("wasm"));
                fs::write(&out_path, wasm).await.expect("Failed to write WASM");
                println!("Compiled to {:?}", out_path);
            }
            Err(err) => {
                eprintln!("Failed to read file: {}", err);
                std::process::exit(1);
            }
        },
    }
}
