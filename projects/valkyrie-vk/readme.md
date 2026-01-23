# Valkyrie Project Structure

This directory contains the self-hosting implementation of the Valkyrie language.

## Directory Layout

- **library/**: Contains the core standard library and compiler components written in Valkyrie.
  - `compiler/lexer.vk`: Lexical analyzer implementation (Namespace: `compiler::lexer`).
  - `compiler/ast.vk`: AST node definitions (Namespace: `compiler::ast`).
  - `compiler/parser.vk`: Parser implementation (Namespace: `compiler::parser`).

- **binary/**: Contains executable entry points.
  - `vcc.vk`: The Valkyrie Self-Hosted Compiler entry point.
  - If a binary requires multiple files, it should be organized as `binary/<name>/main.vk`.

- **test/**: Contains test suites for the compiler and language features.
  - `test_lexer.vk`: Tests for the Lexer component.
  - Tests in this directory are standalone and ignore access control restrictions (if any).

## Building and Running

To run a Valkyrie program (e.g., a test):
```bash
# Run Lexer Test
cargo run --manifest-path ../valkyrie-rs/Cargo.toml -- run test/test_lexer.vk

# Run Parser Test
cargo run --manifest-path ../valkyrie-rs/Cargo.toml -- run test/test_parser.vk
```
(Assuming you are in the `projects/valkyrie-vk` directory, or adjust paths accordingly)
