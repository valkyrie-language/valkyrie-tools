use clap::Parser;
mod commands;
mod project;
pub use project::*;
use commands::LegionCommand;

#[derive(Parser)]
#[command(name = "legion")]
#[command(author, version, about = "Valkyrie compiler and runtime tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: LegionCommand,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    if let Err(err) = cli.command.execute().await {
        report_error(err);
        std::process::exit(1);
    }
}

fn report_error(err: valkyrie_error::ValkyrieError) {
    use valkyrie_error::ReportKind;
    let prefix = match err.level {
        ReportKind::Error => "error",
        _ => "info",
    };

    let code = format!("E{:04x}", err.code());
    let key = err.kind.key();
    let data = err.kind.data();

    eprintln!("{}[{}]: {}", prefix, code, key);
    if !data.is_empty() {
        for (k, v) in data {
            eprintln!("  | {} = {}", k, v);
        }
    }

    // Try to get source code if available
    let source = match &*err.kind {
        valkyrie_error::ValkyrieErrorKind::IoError { path: Some(path), .. } => {
            std::fs::read_to_string(path).ok().map(|s| (path.clone(), s))
        }
        _ => {
            // For other errors, we might not know the path in main.rs
            // unless we had a global source cache.
            None
        }
    };

    for label in err.labels {
        let primary = if label.primary { "primary" } else { "secondary" };
        let span = label.span;

        if let Some((path, source)) = &source {
            let (line, col) = offset_to_position(span.get_start() as usize, source);
            if let Some(lkey) = label.key {
                eprintln!("  --> {}:{}:{} [{}] {}", path, line + 1, col + 1, primary, lkey);
            }
            else {
                eprintln!("  --> {}:{}:{} [{}]", path, line + 1, col + 1, primary);
            }

            // Print source snippet
            print_source_snippet(source, line, col, span.get_end() as usize - span.get_start() as usize);
        }
        else {
            if let Some(lkey) = label.key {
                eprintln!("  --> [{} at {:?}] {}", primary, span, lkey);
            }
            else {
                eprintln!("  --> [{} at {:?}]", primary, span);
            }
        }

        for (k, v) in label.data {
            eprintln!("      | {} = {}", k, v);
        }
    }

    if let Some(help) = err.help {
        eprintln!("  help: {}", help.key);
        for (k, v) in help.data {
            eprintln!("    | {} = {}", k, v);
        }
    }
}

fn print_source_snippet(source: &str, line: usize, col: usize, len: usize) {
    let lines: Vec<&str> = source.lines().collect();
    if line < lines.len() {
        let line_text = lines[line];
        eprintln!("{:>5} | {}", line + 1, line_text);
        let padding = " ".repeat(col);
        let underline = "^".repeat(len.max(1));
        eprintln!("      | {}{}", padding, underline);
    }
}

fn offset_to_position(offset: usize, source: &str) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    for (i, c) in source.chars().enumerate() {
        if i == offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        }
        else {
            col += 1;
        }
    }
    (line, col)
}
