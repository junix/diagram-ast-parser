use crate::{Format, Span};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseError {
    pub format: Format,
    pub message: String,
    pub span: Option<Span>,
    pub line: usize,
    pub column: usize,
}

impl ParseError {
    pub fn new(
        format: Format,
        message: impl Into<String>,
        span: Option<Span>,
        source: &str,
    ) -> Self {
        let (line, column) = span
            .map(|span| line_column(source, span.start))
            .unwrap_or((1, 1));
        Self {
            format,
            message: message.into(),
            span,
            line,
            column,
        }
    }

    pub fn at(format: Format, message: impl Into<String>, span: Span, source: &str) -> Self {
        Self::new(format, message, Some(span), source)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} parse error at {}:{}: {}",
            self.format, self.line, self.column, self.message
        )
    }
}

impl Error for ParseError {}

fn line_column(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut offset = byte_offset.min(source.len());
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }
    let mut line = 1usize;
    let mut column = 1usize;

    for ch in source[..offset].chars() {
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}
