#![forbid(unsafe_code)]

pub mod ast;
mod error;
mod format;
mod lexer;
mod parser;
mod span;

pub use error::{ParseError, ParseResult};
pub use format::Format;
pub use span::{Located, Span};

use ast::Document;

#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub max_input_bytes: usize,
    pub max_nesting_depth: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_input_bytes: 8 * 1024 * 1024,
            max_nesting_depth: 128,
        }
    }
}

pub fn parse(format: Format, source: &str) -> ParseResult<Document> {
    parse_with_options(format, source, &ParseOptions::default())
}

pub fn parse_with_options(
    format: Format,
    source: &str,
    options: &ParseOptions,
) -> ParseResult<Document> {
    if source.len() > options.max_input_bytes {
        return Err(ParseError::new(
            format,
            format!(
                "input is {} bytes, exceeding the configured limit of {} bytes",
                source.len(),
                options.max_input_bytes
            ),
            None,
            source,
        ));
    }

    let resolved = if format == Format::Auto {
        Format::detect(source)
    } else {
        format
    };

    parser::parse_document(resolved, source, options)
}
