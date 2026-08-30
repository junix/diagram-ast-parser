use crate::{
    ast::{Scalar, ScalarKind},
    lexer::{lex, LexerConfig, Token, TokenKind},
    Format, ParseError, ParseResult, Span,
};

#[derive(Debug, Clone)]
pub(crate) struct RawStatement {
    pub span: Span,
    pub head: Vec<Token>,
    pub body: Option<Vec<RawStatement>>,
}

pub(crate) fn parse_braced_tree(
    format: Format,
    source: &str,
    config: LexerConfig,
    max_depth: usize,
) -> ParseResult<Vec<RawStatement>> {
    let tokens = lex(format, source, config)?;
    let mut parser = TreeParser {
        format,
        source,
        tokens,
        position: 0,
        max_depth,
    };
    let statements = parser.parse_list(0, false)?;
    parser.skip_separators();
    if let Some(token) = parser.peek() {
        return Err(ParseError::at(
            format,
            format!("unexpected token {:?}", token.text()),
            token.span,
            source,
        ));
    }
    Ok(statements)
}

struct TreeParser<'a> {
    format: Format,
    source: &'a str,
    tokens: Vec<Token>,
    position: usize,
    max_depth: usize,
}

impl TreeParser<'_> {
    fn parse_list(&mut self, depth: usize, expect_close: bool) -> ParseResult<Vec<RawStatement>> {
        if depth > self.max_depth {
            let span = self.peek().map_or(Span::new(0, 0), |token| token.span);
            return Err(ParseError::at(
                self.format,
                format!(
                    "nesting depth exceeds configured limit of {}",
                    self.max_depth
                ),
                span,
                self.source,
            ));
        }

        let mut statements = Vec::new();
        loop {
            self.skip_separators();

            match self.peek() {
                None if expect_close => {
                    return Err(ParseError::new(
                        self.format,
                        "unterminated block: expected `}`",
                        None,
                        self.source,
                    ));
                }
                None => break,
                Some(token) if token.is_symbol("}") => {
                    if expect_close {
                        break;
                    }
                    return Err(ParseError::at(
                        self.format,
                        "unexpected closing brace",
                        token.span,
                        self.source,
                    ));
                }
                Some(_) => {}
            }

            statements.push(self.parse_statement(depth)?);
        }
        Ok(statements)
    }

    fn parse_statement(&mut self, depth: usize) -> ParseResult<RawStatement> {
        let start = self
            .peek()
            .expect("parse_statement is called only with a token")
            .span
            .start;
        let mut head = Vec::new();
        let mut body = None;
        let mut square_depth = 0usize;
        let mut paren_depth = 0usize;
        let mut end = start;

        loop {
            let Some(token) = self.peek().cloned() else {
                break;
            };

            if square_depth == 0 && paren_depth == 0 {
                if matches!(&token.kind, TokenKind::Newline) || token.is_symbol(";") {
                    self.position += 1;
                    break;
                }

                if token.is_symbol("}") {
                    break;
                }

                if token.is_symbol("{") {
                    if head.is_empty() {
                        return Err(ParseError::at(
                            self.format,
                            "a block must have a statement head",
                            token.span,
                            self.source,
                        ));
                    }
                    self.position += 1;
                    let children = self.parse_list(depth + 1, true)?;
                    let close = self.peek().cloned().ok_or_else(|| {
                        ParseError::new(
                            self.format,
                            "unterminated block: expected `}`",
                            None,
                            self.source,
                        )
                    })?;
                    self.position += 1;
                    end = close.span.end;
                    body = Some(children);

                    if let Some(next) = self.peek() {
                        if !matches!(&next.kind, TokenKind::Newline)
                            && !next.is_symbol(";")
                            && !next.is_symbol("}")
                        {
                            return Err(ParseError::at(
                                self.format,
                                "tokens after a braced block are not supported",
                                next.span,
                                self.source,
                            ));
                        }
                    }
                    self.skip_one_separator();
                    break;
                }
            }

            if token.is_symbol("[") {
                square_depth += 1;
            } else if token.is_symbol("]") {
                if square_depth == 0 {
                    return Err(ParseError::at(
                        self.format,
                        "unmatched closing bracket",
                        token.span,
                        self.source,
                    ));
                }
                square_depth -= 1;
            } else if token.is_symbol("(") {
                paren_depth += 1;
            } else if token.is_symbol(")") {
                if paren_depth == 0 {
                    return Err(ParseError::at(
                        self.format,
                        "unmatched closing parenthesis",
                        token.span,
                        self.source,
                    ));
                }
                paren_depth -= 1;
            }

            end = token.span.end;
            head.push(token);
            self.position += 1;
        }

        if square_depth != 0 {
            return Err(ParseError::at(
                self.format,
                "unterminated bracket expression",
                Span::new(start, end),
                self.source,
            ));
        }
        if paren_depth != 0 {
            return Err(ParseError::at(
                self.format,
                "unterminated parenthesized expression",
                Span::new(start, end),
                self.source,
            ));
        }
        if head.is_empty() {
            return Err(ParseError::at(
                self.format,
                "empty statement",
                Span::new(start, end),
                self.source,
            ));
        }

        Ok(RawStatement {
            span: Span::new(start, end),
            head,
            body,
        })
    }

    fn skip_separators(&mut self) {
        while self
            .peek()
            .is_some_and(|token| matches!(&token.kind, TokenKind::Newline) || token.is_symbol(";"))
        {
            self.position += 1;
        }
    }

    fn skip_one_separator(&mut self) {
        if self
            .peek()
            .is_some_and(|token| matches!(&token.kind, TokenKind::Newline) || token.is_symbol(";"))
        {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
}

pub(crate) fn tokens_to_scalars(tokens: &[Token]) -> Vec<Scalar> {
    tokens.iter().map(token_to_scalar).collect()
}

pub(crate) fn token_to_scalar(token: &Token) -> Scalar {
    let kind = match &token.kind {
        TokenKind::Bare(_) => ScalarKind::Word,
        TokenKind::Quoted { .. } => ScalarKind::String,
        TokenKind::Symbol(_) | TokenKind::Newline => ScalarKind::Symbol,
    };
    Scalar {
        span: token.span,
        kind,
        value: token.text().to_owned(),
    }
}

pub(crate) fn quoted_values(tokens: &[Token]) -> Vec<String> {
    tokens
        .iter()
        .filter_map(|token| match &token.kind {
            TokenKind::Quoted { value, .. } => Some(value.clone()),
            _ => None,
        })
        .collect()
}

pub(crate) fn first_word(tokens: &[Token]) -> Option<&str> {
    tokens.first().and_then(|token| match &token.kind {
        TokenKind::Bare(value) => Some(value.as_str()),
        _ => None,
    })
}
