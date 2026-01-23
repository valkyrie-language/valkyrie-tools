# Valkyrie Language

Valkyrie is a statically typed, object-oriented language designed for robustness and expressiveness. It supports modern features like pattern matching, algebraic data types (enums), and generics.

## Bootstrapping Status

Valkyrie is currently in the bootstrapping phase. The compiler (`vcc.vk`) is being written in Valkyrie itself, running on a host compiler written in Rust (`valkyrie-rs`).

### Current Features (Implemented in `vcc.vk`)

- **Lexer & Parser**: Complete support for Valkyrie syntax, including comments (`#`), literals, control flow, classes, and traits. Fixed "Unexpected EOF" and return statement parsing.
- **AST**: Full Abstract Syntax Tree definition, including `TraitStatement`.
- **HIR Lowering**: Conversion from AST to High-Level IR, with scope resolution.
- **Type Checking**: Basic type inference and checking. Supports primitive types (`i32`, `f32`, `bool`, `()`, `String`) and Generics (`Box<T>`).
- **MIR Generation**: Compilation to Middle-Level IR (bytecode).
- **Interpretation**: Iterative Stack-based MIR VM (supports async/await, generators, and deep recursion). Improved stack management for robust function/constructor returns.
- **Control Flow**: `if`, `while`, `loop`, and Block Expressions. Implicit returns supported (Rust-style).
- **Generics**: Full parsing and representation support. `List<T>` is strongly typed, while `List` (or `List<Any>`) remains dynamic.
- **Standard Library**: `std::result::Result` (Fine/Fail), `std::collections` (`List<T>`, `HashMap<K,V>`), and `std::os` (FFI support) implementation in `valkyrie-std`.
- **FFI Support**: Multi-backend native function binding using `@.native` annotation.
- **Macro Expansion**: Basic macro expansion infrastructure (macros called via `@name`).
- **Package Manager**: Prototype implemented in `library/pkg`, supporting `legion.toml` parsing and dependency resolution.
- **Test Framework**: Native testing support via `@.test` annotation, `@assert`/`@debug` macros, and `vcc test` command.
- **CLI Capabilities**: Native support for reading command-line arguments, environment variables, and file system operations.
- **Module System**: Enhanced `using` resolver with support for nested imports (`using a::{b, c}`), wildcards (`*`), and `self` aliasing. Enforced package prefix for namespaces. Integrated package name reading from `legion.toml`. Replaced legacy `@include` with standard `using package::` module system.
- **Error Handling**: Improved host error reporting and `EvalError` conversions. Implemented unified `Diagnostic` class in bootstrapping compiler.
- **Data Structures**: 
    - `HashMap<K,V>`: Unordered hash map (Available in `std::collections` and `valkyrie-std`).
    - `TreeMap<K,V>`: Prefix-ordered map (Skeleton in `valkyrie-std`).
    - `IndexMap<K,V>`: Insertion-ordered map (Skeleton in `valkyrie-std`).
    - `LinkedList<T>`: Singly linked list (Available in `valkyrie-std`).
    - `Array<T>`: Fixed-size array (Available in `valkyrie-std`).
    - `ArrayList<T>`: Dynamic array/Vector (Available in `valkyrie-std`).
    - `List<T>`, `Map<K,V>`: Traits defining collection interfaces.
- **Cross-Project Parsing**: Support for including files from external projects (e.g., `valkyrie-std`).
- **WASM Backend**: Basic compilation to WebAssembly (Arithmetic operations, Locals, and `print` supported). Capable of generating `output.wasm` directly from `vcc`.
- **JVM Backend**: Compiles to JVM bytecode (via Jasmin assembly), supporting basic OOP and control flow.
- **C Backend**: Compiles to portable C99 code, serving as a universal native backend.
- **Self-Hosting**: `vcc` has successfully compiled itself (`main.vk`) to WASM, verifying the full bootstrapping cycle (Lexing -> Parsing -> Lowering -> Analysis -> MIR -> CodeGenWasm).

### Project Structure

- `projects/valkyrie-rs`: Host compiler (Rust).
- `projects/valkyrie-vk`: Bootstrapping compiler (Valkyrie).
- `projects/valkyrie-std`: Standard Library (Valkyrie).
  - `library/collections`: Generic collections (`HashMap`, `TreeMap`, `IndexMap`).
  - `library/primitive`: Primitive types (`i32`, `bool`, etc.).
  - `library/result.vk`: `Result` type (`Fine`, `Fail`).
  - `library/ast`: AST nodes and parser.
  - `library/hir`: HIR nodes, lowering, and semantic analysis.
  - `library/mir`: MIR nodes, compiler, and interpreter.
  - `library/std`: Standard library (`Result`, etc.).
  - `binary/vcc.vk`: Compiler entry point.

## Usage

The compiler CLI `valkyrie-rs` supports multiple modes:

- **Eval**: Evaluate a code string.
  ```bash
  valkyrie-rs eval "print(1 + 2)"
  ```
- **Run**: Compile and run a source file (Project Mode).
  ```bash
  valkyrie-rs run main.vk
  ```
- **Repl**: Start an interactive REPL session (Cell-by-cell execution with state persistence).
  ```bash
  valkyrie-rs repl
  ```
- **Check**: Perform static analysis on a source file without execution.
  ```bash
  valkyrie-rs check main.vk
  ```
- **Wasm**: Compile a source file to WASM.
  ```bash
  valkyrie-rs wasm main.vk --output main.wasm
  ```

To run the bootstrapping compiler using the generated native executable:

```bash
./projects/valkyrie-rs/bin/vcc.exe
```

Or using the debug version for more verbose output:

```bash
./projects/valkyrie-rs/bin/vcc_debug.exe
```

To build the native executables from source:

```bash
cd projects/valkyrie-rs
cargo build --release --bin vcc --bin vcc_debug
```

## Roadmap

See `Roadmap.md` for detailed progress and future plans.
