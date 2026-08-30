use crate::{Format, ParseError, ParseResult, Span};

#[derive(Debug, Clone, Copy)]
pub(crate) struct LexerConfig {
    pub hash_comments: bool,
    pub slash_line_comments: bool,
    pub block_comments: bool,
    pub triple_strings: bool,
    pub backtick_strings: bool,
}

impl LexerConfig {
    pub const fn dbml() -> Self {
        Self {
            hash_comments: false,
            slash_line_comments: true,
            block_comments: true,
            triple_strings: true,
            backtick_strings: true,
        }
    }

    pub const fn d2() -> Self {
        Self {
            hash_comments: true,
            slash_line_comments: false,
            block_comments: false,
            triple_strings: true,
            backtick_strings: false,
        }
    }

    pub const fn structurizr() -> Self {
        Self {
            hash_comments: false,
            slash_line_comments: true,
            block_comments: true,
            triple_strings: true,
            backtick_strings: false,
        }
    }

    pub const fn likec4() -> Self {
        Self {
            hash_comments: false,
            slash_line_comments: true,
            block_comments: true,
            triple_strings: true,
            backtick_strings: false,
        }
    }

    pub const fn pikchr() -> Self {
        Self {
            hash_comments: true,
            slash_line_comments: true,
            block_comments: true,
            triple_strings: false,
            backtick_strings: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Bare(String),
    Quoted {
        value: String,
        delimiter: char,
        triple: bool,
    },
    Symbol(String),
    Newline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn text(&self) -> &str {
        match &self.kind {
            TokenKind::Bare(value) => value,
            TokenKind::Quoted { value, .. } => value,
            TokenKind::Symbol(value) => value,
            TokenKind::Newline => "\n",
        }
    }

    pub fn is_symbol(&self, symbol: &str) -> bool {
        matches!(&self.kind, TokenKind::Symbol(value) if value == symbol)
    }

    pub fn is_bare(&self, word: &str) -> bool {
        matches!(&self.kind, TokenKind::Bare(value) if value.eq_ignore_ascii_case(word))
    }

    pub fn is_quoted(&self) -> bool {
        matches!(&self.kind, TokenKind::Quoted { .. })
    }
}

pub(crate) fn lex(format: Format, source: &str, config: LexerConfig) -> ParseResult<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut index = 0usize;

    while index < source.len() {
        let ch = next_char(source, index);

        if ch == '\n' {
            tokens.push(Token {
                kind: TokenKind::Newline,
                span: Span::new(index, index + 1),
            });
            index += 1;
            continue;
        }

        if ch.is_whitespace() {
            index += ch.len_utf8();
            continue;
        }

        if config.hash_comments && ch == '#' && hash_starts_comment(source, index) {
            index = skip_line_comment(source, index);
            continue;
        }

        if ch == '/' && source[index..].starts_with("//") && config.slash_line_comments {
            if slash_starts_comment(source, index) {
                index = skip_line_comment(source, index);
                continue;
            }
        }

        if ch == '/' && source[index..].starts_with("/*") && config.block_comments {
            let start = index;
            if let Some(end) = source[index + 2..].find("*/") {
                index += 2 + end + 2;
                continue;
            }
            return Err(ParseError::at(
                format,
                "unterminated block comment",
                Span::new(start, source.len()),
                source,
            ));
        }

        if ch == '\'' || ch == '"' || (ch == '`' && config.backtick_strings) {
            let (token, next) = lex_string(format, source, index, config.triple_strings)?;
            tokens.push(token);
            index = next;
            continue;
        }

        if let Some(operator) = match_operator(&source[index..]) {
            tokens.push(Token {
                kind: TokenKind::Symbol(operator.to_owned()),
                span: Span::new(index, index + operator.len()),
            });
            index += operator.len();
            continue;
        }

        if is_single_symbol(ch) {
            let end = index + ch.len_utf8();
            tokens.push(Token {
                kind: TokenKind::Symbol(ch.to_string()),
                span: Span::new(index, end),
            });
            index = end;
            continue;
        }

        let start = index;
        while index < source.len() {
            let current = next_char(source, index);
            if current == '\n'
                || current.is_whitespace()
                || current == '\''
                || current == '"'
                || (current == '`' && config.backtick_strings)
                || is_single_symbol(current)
                || match_operator(&source[index..]).is_some()
            {
                break;
            }
            index += current.len_utf8();
        }

        if start == index {
            return Err(ParseError::at(
                format,
                format!("unexpected character {ch:?}"),
                Span::new(index, index + ch.len_utf8()),
                source,
            ));
        }

        tokens.push(Token {
            kind: TokenKind::Bare(source[start..index].to_owned()),
            span: Span::new(start, index),
        });
    }

    Ok(tokens)
}

fn lex_string(
    format: Format,
    source: &str,
    start: usize,
    allow_triple: bool,
) -> ParseResult<(Token, usize)> {
    let delimiter = next_char(source, start);
    let delimiter_len = delimiter.len_utf8();
    let triple = allow_triple
        && delimiter != '`'
        && source[start..].starts_with(&delimiter.to_string().repeat(3));
    let opening_len = if triple {
        delimiter_len * 3
    } else {
        delimiter_len
    };
    let mut index = start + opening_len;
    let content_start = index;
    let mut escaped = false;

    while index < source.len() {
        if triple && source[index..].starts_with(&delimiter.to_string().repeat(3)) && !escaped {
            let raw = &source[content_start..index];
            let end = index + delimiter_len * 3;
            return Ok((
                Token {
                    kind: TokenKind::Quoted {
                        value: decode_string(raw, delimiter),
                        delimiter,
                        triple,
                    },
                    span: Span::new(start, end),
                },
                end,
            ));
        }

        let ch = next_char(source, index);
        if !triple && ch == delimiter && !escaped {
            let raw = &source[content_start..index];
            let end = index + delimiter_len;
            return Ok((
                Token {
                    kind: TokenKind::Quoted {
                        value: decode_string(raw, delimiter),
                        delimiter,
                        triple,
                    },
                    span: Span::new(start, end),
                },
                end,
            ));
        }

        if ch == '\\' && !escaped {
            escaped = true;
        } else {
            escaped = false;
        }
        index += ch.len_utf8();
    }

    Err(ParseError::at(
        format,
        "unterminated string literal",
        Span::new(start, source.len()),
        source,
    ))
}

fn decode_string(raw: &str, delimiter: char) -> String {
    let mut decoded = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            decoded.push(ch);
            continue;
        }

        let Some(next) = chars.next() else {
            decoded.push('\\');
            break;
        };

        match next {
            'n' => decoded.push('\n'),
            'r' => decoded.push('\r'),
            't' => decoded.push('\t'),
            '\\' => decoded.push('\\'),
            value if value == delimiter => decoded.push(value),
            other => {
                decoded.push('\\');
                decoded.push(other);
            }
        }
    }
    decoded
}

fn hash_starts_comment(source: &str, index: usize) -> bool {
    let line_start = source[..index]
        .rfind('\n')
        .map_or(0, |position| position + 1);
    let prefix = &source[line_start..index];
    if prefix.trim().is_empty() {
        return true;
    }

    let previous_non_whitespace = prefix.chars().rev().find(|ch| !ch.is_whitespace());
    let next = source[index + 1..].chars().next();
    if previous_non_whitespace == Some(':') && next.is_some_and(|ch| ch.is_ascii_hexdigit()) {
        return false;
    }
    true
}

fn slash_starts_comment(source: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }
    match source[..index].chars().next_back() {
        Some(previous) => previous != ':',
        None => true,
    }
}

fn skip_line_comment(source: &str, index: usize) -> usize {
    source[index..]
        .find('\n')
        .map_or(source.len(), |relative| index + relative)
}

fn next_char(source: &str, index: usize) -> char {
    source[index..]
        .chars()
        .next()
        .expect("index must point inside source")
}

fn match_operator(source: &str) -> Option<&'static str> {
    const OPERATORS: &[&str] = &[
        "<->", "-->", "<--", "...", "::", "->", "<-", "--", "=>", "<=", ">=", "==", "!=", "+=",
        "-=", "*=", "/=", "||", "&&", "<>",
    ];
    OPERATORS
        .iter()
        .copied()
        .find(|operator| source.starts_with(operator))
}

fn is_single_symbol(ch: char) -> bool {
    matches!(
        ch,
        '{' | '}'
            | '['
            | ']'
            | '('
            | ')'
            | ','
            | ';'
            | ':'
            | '='
            | '.'
            | '!'
            | '@'
            | '#'
            | '~'
            | '*'
            | '+'
            | '-'
            | '/'
            | '<'
            | '>'
            | '|'
            | '&'
    )
}

pub(crate) fn render_tokens(tokens: &[Token]) -> String {
    let mut output = String::new();
    let mut previous: Option<&Token> = None;

    for token in tokens {
        if matches!(&token.kind, TokenKind::Newline) {
            if !output.ends_with('\n') {
                output.push('\n');
            }
            previous = Some(token);
            continue;
        }

        let needs_space = previous.is_some_and(|previous| token_needs_space(previous, token));
        if needs_space && !output.ends_with(' ') && !output.ends_with('\n') {
            output.push(' ');
        }

        match &token.kind {
            TokenKind::Quoted {
                value,
                delimiter,
                triple,
            } => {
                let count = if *triple { 3 } else { 1 };
                for _ in 0..count {
                    output.push(*delimiter);
                }
                output.push_str(value);
                for _ in 0..count {
                    output.push(*delimiter);
                }
            }
            TokenKind::Bare(value) | TokenKind::Symbol(value) => output.push_str(value),
            TokenKind::Newline => {}
        }

        previous = Some(token);
    }

    output
}

fn token_needs_space(previous: &Token, current: &Token) -> bool {
    let previous_closes = matches!(
        &previous.kind,
        TokenKind::Bare(_) | TokenKind::Quoted { .. }
    ) || previous.is_symbol(")")
        || previous.is_symbol("]");
    let current_opens = matches!(&current.kind, TokenKind::Bare(_) | TokenKind::Quoted { .. })
        || current.is_symbol("(")
        || current.is_symbol("[");

    if previous.is_symbol(".")
        || current.is_symbol(".")
        || previous.is_symbol("(")
        || current.is_symbol("(")
        || previous.is_symbol("[")
        || current.is_symbol("[")
        || current.is_symbol(")")
        || current.is_symbol("]")
        || current.is_symbol(",")
        || current.is_symbol(":")
        || previous.is_symbol(":")
    {
        return false;
    }

    previous_closes && current_opens
}
