use clap::Parser;
use std::path::PathBuf;
use std::fmt::{Display, Formatter};
use std::error::Error;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtxBuilder};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};

#[derive(Debug)]
pub enum RunnerError {
    LoadModule(String),
    Instantiate(String),
    FunctionCall(String),
    Io(std::io::Error),
    Wasmtime(wasmtime::Error),
}

impl Display for RunnerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RunnerError::LoadModule(msg) => write!(f, "Failed to load module: {}", msg),
            RunnerError::Instantiate(msg) => write!(f, "Failed to instantiate module: {}", msg),
            RunnerError::FunctionCall(msg) => write!(f, "Failed to call function: {}", msg),
            RunnerError::Io(err) => write!(f, "IO error: {}", err),
            RunnerError::Wasmtime(err) => write!(f, "Wasmtime error: {}", err),
        }
    }
}

impl Error for RunnerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RunnerError::Io(err) => Some(err),
            RunnerError::Wasmtime(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RunnerError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<wasmtime::Error> for RunnerError {
    fn from(error: wasmtime::Error) -> Self {
        Self::Wasmtime(error)
    }
}

#[derive(Parser, Debug)]
#[command(name = "wasi-runner")]
#[command(about = "A tool to run WASM modules with wasmtime")]
struct Args {
    /// Path to the WASM module file
    #[arg(short, long)]
    module: PathBuf,

    /// Name of the function to call (optional)
    #[arg(short, long)]
    function: Option<String>,

    /// Arguments to pass to the function (optional)
    #[arg(short, long)]
    args: Vec<String>,

    /// Enable WASI preview1
    #[arg(long)]
    wasi_preview1: bool,

    /// Enable WASI preview2
    #[arg(long)]
    wasi_preview2: bool,
}

fn main() -> Result<(), RunnerError> {
    let args = Args::parse();

    // Initialize the wasmtime engine
    let engine = Engine::default();

    // Create a module store with WASI context
    let mut builder = WasiCtxBuilder::new();
    builder.inherit_stdio();
    let args_str: Vec<&str> = args.args.iter().map(|s| s.as_str()).collect();
    builder.args(&args_str);
    let wasi = builder.build_p1();
    let mut store = Store::new(&engine, wasi);

    // Create a linker for WASI
    let mut linker = Linker::new(&engine);
    preview1::add_to_linker_sync(&mut linker, |s: &mut WasiP1Ctx| s)?;

    // Load the module
    let module = Module::from_file(&engine, &args.module)
        .map_err(|e| RunnerError::LoadModule(format!("Failed to load module: {}", e)))?;

    // Instantiate the module
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| RunnerError::Instantiate(format!("Failed to instantiate module: {}", e)))?;

    // If a function name is provided, try to call it
    if let Some(func_name) = args.function {
        // Get the function from the instance
        let func = instance
            .get_typed_func::<(), ()>(&mut store, &func_name)
            .map_err(|e| RunnerError::FunctionCall(format!("Failed to get function '{}': {}", func_name, e)))?;

        // Call the function
        func.call(&mut store, ())
            .map_err(|e| RunnerError::FunctionCall(format!("Failed to call function '{}': {}", func_name, e)))?;

        println!("Successfully called function: {}", func_name);
    }
    else {
        // List all exported functions
        println!("Available exports in the module:");
        for export in module.exports() {
            println!("  {}: {:?}", export.name(), export.ty());
        }
    }

    Ok(())
}
