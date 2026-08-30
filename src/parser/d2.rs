use crate::{
    ast::d2::{D2Document, D2EdgeChain, D2EdgeOperator, D2Entry, D2Import, D2Statement, D2Value},
    lexer::{render_tokens, LexerConfig, Token},
    Format, Located, ParseError, ParseOptions, ParseResult, Span,
};

use super::tree::{parse_braced_tree, RawStatement};

pub(crate) fn parse(source: &str, options: &ParseOptions) -> ParseResult<D2Document> {
    let raw = parse_braced_tree(
        Format::D2,
        source,
        LexerConfig::d2(),
        options.max_nesting_depth,
    )?;
    let statements = convert_statements(&raw, source)?;
    Ok(D2Document {
        span: Span::new(0, source.len()),
        statements,
    })
}

fn convert_statements(
    raw: &[RawStatement],
    source: &str,
) -> ParseResult<Vec<Located<D2Statement>>> {
    raw.iter()
        .map(|statement| {
            convert_statement(statement, source).map(|node| Located::new(statement.span, node))
        })
        .collect()
}

fn convert_statement(statement: &RawStatement, source: &str) -> ParseResult<D2Statement> {
    if is_import(&statement.head) {
        return parse_import(statement, source).map(D2Statement::Import);
    }

    let operators = edge_operators(&statement.head);
    if !operators.is_empty() {
        return parse_edge(statement, source, &operators).map(D2Statement::EdgeChain);
    }

    parse_entry(statement, source).map(D2Statement::Entry)
}

fn parse_entry(statement: &RawStatement, source: &str) -> ParseResult<D2Entry> {
    let colon = find_symbol(&statement.head, ":");
    let key_end = colon.unwrap_or(statement.head.len());
    if key_end == 0 {
        return Err(ParseError::at(
            Format::D2,
            "D2 entry requires a key before `:`",
            statement.span,
            source,
        ));
    }
    let key = render_tokens(&statement.head[..key_end]);
    let scalar = colon
        .filter(|index| index + 1 < statement.head.len())
        .map(|index| render_tokens(&statement.head[index + 1..]))
        .filter(|value| !value.trim().is_empty());

    let value = match &statement.body {
        Some(body) => Some(D2Value::Map {
            label: scalar,
            statements: convert_statements(body, source)?,
        }),
        None => scalar.map(D2Value::Scalar),
    };

    Ok(D2Entry { key, value })
}

fn parse_edge(
    statement: &RawStatement,
    source: &str,
    operators: &[(usize, D2EdgeOperator)],
) -> ParseResult<D2EdgeChain> {
    let last_operator = operators.last().expect("checked non-empty").0;
    let label_colon = statement.head[last_operator + 1..]
        .iter()
        .position(|token| token.is_symbol(":"))
        .map(|relative| last_operator + 1 + relative);
    let endpoint_end = label_colon.unwrap_or(statement.head.len());
    let label = label_colon
        .filter(|index| index + 1 < statement.head.len())
        .map(|index| render_tokens(&statement.head[index + 1..]))
        .filter(|value| !value.trim().is_empty());

    let mut endpoints = Vec::with_capacity(operators.len() + 1);
    let mut start = 0usize;
    for (operator_index, _) in operators {
        if *operator_index <= start {
            return Err(ParseError::at(
                Format::D2,
                "edge operator requires an endpoint on its left",
                statement.head[*operator_index].span,
                source,
            ));
        }
        endpoints.push(render_tokens(&statement.head[start..*operator_index]));
        start = *operator_index + 1;
    }
    if start >= endpoint_end {
        return Err(ParseError::at(
            Format::D2,
            "edge operator requires an endpoint on its right",
            statement.span,
            source,
        ));
    }
    endpoints.push(render_tokens(&statement.head[start..endpoint_end]));

    let attributes = match &statement.body {
        Some(body) => convert_statements(body, source)?,
        None => Vec::new(),
    };

    Ok(D2EdgeChain {
        endpoints,
        operators: operators.iter().map(|(_, operator)| *operator).collect(),
        label,
        attributes,
    })
}

fn parse_import(statement: &RawStatement, source: &str) -> ParseResult<D2Import> {
    if statement.body.is_some() {
        return Err(ParseError::at(
            Format::D2,
            "D2 imports cannot have a braced body",
            statement.span,
            source,
        ));
    }
    let at = find_symbol(&statement.head, "@").ok_or_else(|| {
        ParseError::at(
            Format::D2,
            "invalid import: missing `@`",
            statement.span,
            source,
        )
    })?;
    if at + 1 >= statement.head.len() {
        return Err(ParseError::at(
            Format::D2,
            "D2 import requires a path after `@`",
            statement.span,
            source,
        ));
    }
    Ok(D2Import {
        path: render_tokens(&statement.head[at + 1..]),
        spread: statement
            .head
            .first()
            .is_some_and(|token| token.is_symbol("...")),
    })
}

fn is_import(tokens: &[Token]) -> bool {
    tokens.first().is_some_and(|token| token.is_symbol("@"))
        || (tokens.first().is_some_and(|token| token.is_symbol("..."))
            && tokens.get(1).is_some_and(|token| token.is_symbol("@")))
}

fn edge_operators(tokens: &[Token]) -> Vec<(usize, D2EdgeOperator)> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            let operator = if token.is_symbol("->") {
                D2EdgeOperator::Directed
            } else if token.is_symbol("<-") {
                D2EdgeOperator::ReverseDirected
            } else if token.is_symbol("--") {
                D2EdgeOperator::Undirected
            } else if token.is_symbol("<->") {
                D2EdgeOperator::Bidirectional
            } else {
                return None;
            };
            Some((index, operator))
        })
        .collect()
}

fn find_symbol(tokens: &[Token], symbol: &str) -> Option<usize> {
    tokens.iter().position(|token| token.is_symbol(symbol))
}
