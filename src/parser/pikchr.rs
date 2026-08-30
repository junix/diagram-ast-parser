use crate::{
    ast::pikchr::{
        PikchrAssignment, PikchrDefine, PikchrDirection, PikchrDocument, PikchrObject, PikchrPlace,
        PikchrStatement,
    },
    lexer::{render_tokens, LexerConfig, Token, TokenKind},
    Format, Located, ParseError, ParseOptions, ParseResult, Span,
};

use super::tree::{parse_braced_tree, tokens_to_scalars, RawStatement};

pub(crate) fn parse(source: &str, options: &ParseOptions) -> ParseResult<PikchrDocument> {
    let raw = parse_braced_tree(
        Format::Pikchr,
        source,
        LexerConfig::pikchr(),
        options.max_nesting_depth,
    )?;
    let statements = raw
        .iter()
        .map(|statement| {
            convert_statement(statement, source).map(|node| Located::new(statement.span, node))
        })
        .collect::<ParseResult<Vec<_>>>()?;
    Ok(PikchrDocument {
        span: Span::new(0, source.len()),
        statements,
    })
}

fn convert_statement(statement: &RawStatement, source: &str) -> ParseResult<PikchrStatement> {
    let first = statement.head.first().ok_or_else(|| {
        ParseError::at(
            Format::Pikchr,
            "empty Pikchr statement",
            statement.span,
            source,
        )
    })?;

    if let Some(direction) = parse_direction(first.text()) {
        if statement.head.len() != 1 || statement.body.is_some() {
            return Err(ParseError::at(
                Format::Pikchr,
                "direction statement cannot have arguments or a body",
                statement.span,
                source,
            ));
        }
        return Ok(PikchrStatement::Direction(direction));
    }

    if first.is_bare("define") {
        return parse_define(statement, source).map(PikchrStatement::Define);
    }
    if first.is_bare("print") {
        return Ok(PikchrStatement::Print(tokens_to_scalars(
            &statement.head[1..],
        )));
    }
    if first.is_bare("assert") {
        if statement.head.len() < 2 {
            return Err(ParseError::at(
                Format::Pikchr,
                "assert requires an expression",
                statement.span,
                source,
            ));
        }
        return Ok(PikchrStatement::Assert(render_tokens(&statement.head[1..])));
    }

    if let Some(operator_index) = find_assignment_operator(&statement.head) {
        if operator_index == 0 || operator_index + 1 >= statement.head.len() {
            return Err(ParseError::at(
                Format::Pikchr,
                "assignment requires a variable and an expression",
                statement.span,
                source,
            ));
        }
        return Ok(PikchrStatement::Assignment(PikchrAssignment {
            variable: render_tokens(&statement.head[..operator_index]),
            operator: statement.head[operator_index].text().to_owned(),
            expression: render_tokens(&statement.head[operator_index + 1..]),
        }));
    }

    let (label, content_start) = if statement
        .head
        .get(1)
        .is_some_and(|token| token.is_symbol(":"))
    {
        (Some(first.text().to_owned()), 2)
    } else {
        (None, 0)
    };

    let content = &statement.head[content_start..];
    let Some(object_or_place) = content.first() else {
        return Err(ParseError::at(
            Format::Pikchr,
            "label must be followed by an object definition or place",
            statement.span,
            source,
        ));
    };

    if is_object_start(object_or_place) {
        let object_type = match &object_or_place.kind {
            TokenKind::Quoted { .. } => "text".to_owned(),
            TokenKind::Symbol(value) if value == "[" => "block".to_owned(),
            _ => object_or_place.text().to_owned(),
        };
        let attribute_start = if object_or_place.is_quoted() || object_or_place.is_symbol("[") {
            0
        } else {
            1
        };
        return Ok(PikchrStatement::Object(PikchrObject {
            label,
            object_type,
            attributes: tokens_to_scalars(&content[attribute_start..]),
        }));
    }

    if let Some(label) = label {
        return Ok(PikchrStatement::Place(PikchrPlace {
            label,
            expression: render_tokens(content),
        }));
    }

    Err(ParseError::at(
        Format::Pikchr,
        format!(
            "unsupported Pikchr statement starting with `{}`",
            first.text()
        ),
        statement.span,
        source,
    ))
}

fn parse_define(statement: &RawStatement, source: &str) -> ParseResult<PikchrDefine> {
    let name = statement.head.get(1).map(Token::text).ok_or_else(|| {
        ParseError::at(
            Format::Pikchr,
            "define requires a macro name",
            statement.span,
            source,
        )
    })?;
    if statement.body.is_none() {
        return Err(ParseError::at(
            Format::Pikchr,
            "define requires a braced code block",
            statement.span,
            source,
        ));
    }
    let raw = &source[statement.span.start..statement.span.end];
    let open = raw.find('{').ok_or_else(|| {
        ParseError::at(
            Format::Pikchr,
            "define code block is missing `{`",
            statement.span,
            source,
        )
    })?;
    let close = raw.rfind('}').ok_or_else(|| {
        ParseError::at(
            Format::Pikchr,
            "define code block is missing `}`",
            statement.span,
            source,
        )
    })?;
    Ok(PikchrDefine {
        name: name.to_owned(),
        body: raw[open + 1..close].to_owned(),
    })
}

fn parse_direction(word: &str) -> Option<PikchrDirection> {
    if word.eq_ignore_ascii_case("right") {
        Some(PikchrDirection::Right)
    } else if word.eq_ignore_ascii_case("down") {
        Some(PikchrDirection::Down)
    } else if word.eq_ignore_ascii_case("left") {
        Some(PikchrDirection::Left)
    } else if word.eq_ignore_ascii_case("up") {
        Some(PikchrDirection::Up)
    } else {
        None
    }
}

fn find_assignment_operator(tokens: &[Token]) -> Option<usize> {
    tokens.iter().position(|token| {
        token.is_symbol("=")
            || token.is_symbol("+=")
            || token.is_symbol("-=")
            || token.is_symbol("*=")
            || token.is_symbol("/=")
    })
}

fn is_object_start(token: &Token) -> bool {
    if token.is_quoted() || token.is_symbol("[") {
        return true;
    }
    matches!(
        token.text().to_ascii_lowercase().as_str(),
        "arc"
            | "arrow"
            | "box"
            | "circle"
            | "cylinder"
            | "diamond"
            | "dot"
            | "ellipse"
            | "file"
            | "line"
            | "move"
            | "oval"
            | "spline"
            | "text"
    )
}
