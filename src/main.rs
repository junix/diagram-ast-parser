use clap::Parser;
use diagram_ast_parser::{parse_with_options, Format, ParseError, ParseOptions};
use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
    process::ExitCode,
    str::FromStr,
};

#[derive(Debug, Parser)]
#[command(name = "diagram-parse")]
#[command(about = "Parse diagram DSL input and emit a JSON AST")]
struct Cli {
    /// Input format: auto, dbml, wavedrom, d2, structurizr, likec4, nomnoml, or pikchr.
    #[arg(short, long, default_value = "auto")]
    format: String,

    /// Input file. Use `-` or omit it to read stdin.
    input: Option<PathBuf>,

    /// Emit compact JSON instead of pretty-printed JSON.
    #[arg(long)]
    compact: bool,

    /// Emit parse failures as JSON.
    #[arg(long)]
    diagnostic_json: bool,

    /// Reject inputs larger than this many bytes.
    #[arg(long, default_value_t = 8_388_608)]
    max_input_bytes: usize,

    /// Maximum nested braced-block depth for brace-based DSL parsers.
    #[arg(long, default_value_t = 128)]
    max_depth: usize,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let format = match Format::from_str(&cli.format) {
        Ok(format) => format,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::from(2);
        }
    };

    let source = match read_input(cli.input.as_ref()) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    };

    let options = ParseOptions {
        max_input_bytes: cli.max_input_bytes,
        max_nesting_depth: cli.max_depth,
    };

    match parse_with_options(format, &source, &options) {
        Ok(document) => {
            let serialized = if cli.compact {
                serde_json::to_string(&document)
            } else {
                serde_json::to_string_pretty(&document)
            };
            match serialized {
                Ok(json) => {
                    println!("{json}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("error: failed to serialize AST: {error}");
                    ExitCode::from(3)
                }
            }
        }
        Err(error) => {
            print_parse_error(&error, &source, cli.diagnostic_json);
            ExitCode::from(1)
        }
    }
}

fn read_input(path: Option<&PathBuf>) -> io::Result<String> {
    match path {
        Some(path) if path.as_os_str() != std::ffi::OsStr::new("-") => fs::read_to_string(path),
        _ => {
            let mut source = String::new();
            io::stdin().read_to_string(&mut source)?;
            Ok(source)
        }
    }
}

fn print_parse_error(error: &ParseError, source: &str, as_json: bool) {
    if as_json {
        match serde_json::to_string_pretty(error) {
            Ok(json) => eprintln!("{json}"),
            Err(_) => eprintln!("{error}"),
        }
        return;
    }

    eprintln!("{error}");
    if let Some(span) = error.span {
        if let Some(line) = source.lines().nth(error.line.saturating_sub(1)) {
            eprintln!("  {line}");
            let caret_offset = error.column.saturating_sub(1);
            let width = span
                .len()
                .max(1)
                .min(line.len().saturating_sub(caret_offset).max(1));
            eprintln!("  {}{}", " ".repeat(caret_offset), "^".repeat(width));
        }
    }
}
