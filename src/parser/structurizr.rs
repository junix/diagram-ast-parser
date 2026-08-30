use crate::{
    ast::structurizr::{
        StructurizrBlock, StructurizrDirective, StructurizrDocument, StructurizrElement,
        StructurizrProperty, StructurizrRelationship, StructurizrStatement, StructurizrWorkspace,
    },
    lexer::{render_tokens, LexerConfig, Token, TokenKind},
    Format, Located, ParseError, ParseOptions, ParseResult, Span,
};

use super::tree::{parse_braced_tree, quoted_values, tokens_to_scalars, RawStatement};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Top,
    Workspace,
    Model,
    Views,
    Element,
    Other,
}

pub(crate) fn parse(source: &str, options: &ParseOptions) -> ParseResult<StructurizrDocument> {
    let raw = parse_braced_tree(
        Format::Structurizr,
        source,
        LexerConfig::structurizr(),
        options.max_nesting_depth,
    )?;
    let statements = convert_list(&raw, source, Scope::Top)?;
    Ok(StructurizrDocument {
        span: Span::new(0, source.len()),
        statements,
    })
}

fn convert_list(
    raw: &[RawStatement],
    source: &str,
    scope: Scope,
) -> ParseResult<Vec<Located<StructurizrStatement>>> {
    raw.iter()
        .map(|statement| {
            convert_statement(statement, source, scope)
                .map(|node| Located::new(statement.span, node))
        })
        .collect()
}

fn convert_statement(
    statement: &RawStatement,
    source: &str,
    scope: Scope,
) -> ParseResult<StructurizrStatement> {
    if is_directive(&statement.head) {
        return Ok(StructurizrStatement::Directive(parse_directive(statement)));
    }

    if has_symbol(&statement.head, "->") {
        return parse_relationship(statement, source).map(StructurizrStatement::Relationship);
    }

    let first = first_text(&statement.head).ok_or_else(|| {
        ParseError::at(
            Format::Structurizr,
            "empty Structurizr statement",
            statement.span,
            source,
        )
    })?;

    if first.eq_ignore_ascii_case("workspace") {
        return parse_workspace(statement, source).map(StructurizrStatement::Workspace);
    }

    if is_element_statement(statement, scope) {
        return parse_element(statement, source).map(StructurizrStatement::Element);
    }

    if statement.body.is_some() || is_block_keyword(first) {
        return parse_block(statement, source, scope).map(StructurizrStatement::Block);
    }

    Ok(StructurizrStatement::Property(parse_property(statement)))
}

fn parse_workspace(statement: &RawStatement, source: &str) -> ParseResult<StructurizrWorkspace> {
    let body = statement.body.as_deref().ok_or_else(|| {
        ParseError::at(
            Format::Structurizr,
            "workspace requires a braced body",
            statement.span,
            source,
        )
    })?;
    let strings = quoted_values(&statement.head[1..]);
    let extends_index = statement
        .head
        .iter()
        .position(|token| token.is_bare("extends"));
    let extends = extends_index
        .filter(|index| index + 1 < statement.head.len())
        .map(|index| render_tokens(&statement.head[index + 1..]));
    Ok(StructurizrWorkspace {
        name: strings.first().cloned(),
        description: strings.get(1).cloned(),
        extends,
        body: convert_list(body, source, Scope::Workspace)?,
    })
}

fn parse_element(statement: &RawStatement, source: &str) -> ParseResult<StructurizrElement> {
    let equal = statement.head.iter().position(|token| token.is_symbol("="));
    let (id, kind_index, argument_start) = if let Some(equal) = equal {
        if equal == 0 || equal + 1 >= statement.head.len() {
            return Err(ParseError::at(
                Format::Structurizr,
                "element assignment must be `identifier = elementType ...`",
                statement.span,
                source,
            ));
        }
        (
            Some(render_tokens(&statement.head[..equal])),
            equal + 1,
            equal + 2,
        )
    } else {
        let has_explicit_id = statement
            .head
            .get(1)
            .is_some_and(|token| matches!(&token.kind, TokenKind::Bare(_)))
            && statement.head.get(2).is_some_and(|token| token.is_quoted());
        let inferred_id = has_explicit_id.then(|| token_value(&statement.head[1]));
        (inferred_id, 0, if has_explicit_id { 2 } else { 1 })
    };

    let element_type = statement
        .head
        .get(kind_index)
        .map(token_value)
        .ok_or_else(|| {
            ParseError::at(
                Format::Structurizr,
                "missing element type",
                statement.span,
                source,
            )
        })?;
    let strings = quoted_values(&statement.head[argument_start..]);
    let body = statement
        .body
        .as_deref()
        .map(|body| convert_list(body, source, Scope::Element))
        .transpose()?
        .unwrap_or_default();
    Ok(StructurizrElement {
        id,
        element_type,
        name: strings.first().cloned(),
        description: strings.get(1).cloned(),
        technology: strings.get(2).cloned(),
        body,
    })
}

fn parse_relationship(
    statement: &RawStatement,
    source: &str,
) -> ParseResult<StructurizrRelationship> {
    let arrow = statement
        .head
        .iter()
        .position(|token| token.is_symbol("->"))
        .expect("caller checked arrow");
    let equal = statement.head[..arrow]
        .iter()
        .position(|token| token.is_symbol("="));
    let (id, source_start) = match equal {
        Some(index) => (Some(render_tokens(&statement.head[..index])), index + 1),
        None => (None, 0),
    };
    if source_start >= arrow {
        return Err(ParseError::at(
            Format::Structurizr,
            "relationship requires a source identifier",
            statement.span,
            source,
        ));
    }
    let target_end = statement.head[arrow + 1..]
        .iter()
        .position(Token::is_quoted)
        .map_or(statement.head.len(), |relative| arrow + 1 + relative);
    if arrow + 1 >= target_end {
        return Err(ParseError::at(
            Format::Structurizr,
            "relationship requires a target identifier",
            statement.span,
            source,
        ));
    }
    let strings = quoted_values(&statement.head[target_end..]);
    let body = statement
        .body
        .as_deref()
        .map(|body| convert_list(body, source, Scope::Other))
        .transpose()?
        .unwrap_or_default();
    Ok(StructurizrRelationship {
        id,
        source: render_tokens(&statement.head[source_start..arrow]),
        target: render_tokens(&statement.head[arrow + 1..target_end]),
        description: strings.first().cloned(),
        technology: strings.get(1).cloned(),
        tags: strings.get(2).cloned(),
        body,
    })
}

fn parse_directive(statement: &RawStatement) -> StructurizrDirective {
    let (name, start) = if statement
        .head
        .first()
        .is_some_and(|token| token.is_symbol("!"))
    {
        (
            statement.head.get(1).map(token_value).unwrap_or_default(),
            2,
        )
    } else {
        (
            first_text(&statement.head)
                .unwrap_or_default()
                .trim_start_matches('!')
                .to_owned(),
            1,
        )
    };
    StructurizrDirective {
        name,
        arguments: tokens_to_scalars(&statement.head[start..]),
    }
}

fn parse_block(
    statement: &RawStatement,
    source: &str,
    scope: Scope,
) -> ParseResult<StructurizrBlock> {
    let keyword = first_text(&statement.head).unwrap_or_default().to_owned();
    let child_scope = if keyword.eq_ignore_ascii_case("model") {
        Scope::Model
    } else if keyword.eq_ignore_ascii_case("views") {
        Scope::Views
    } else if scope == Scope::Model && keyword.eq_ignore_ascii_case("group") {
        Scope::Model
    } else {
        Scope::Other
    };
    let body = statement
        .body
        .as_deref()
        .map(|body| convert_list(body, source, child_scope))
        .transpose()?
        .unwrap_or_default();
    Ok(StructurizrBlock {
        keyword,
        arguments: tokens_to_scalars(&statement.head[1..]),
        body,
    })
}

fn parse_property(statement: &RawStatement) -> StructurizrProperty {
    StructurizrProperty {
        name: first_text(&statement.head).unwrap_or_default().to_owned(),
        values: tokens_to_scalars(&statement.head[1..]),
        body: Vec::new(),
    }
}

fn is_directive(tokens: &[Token]) -> bool {
    tokens.first().is_some_and(|token| token.is_symbol("!"))
        || first_text(tokens).is_some_and(|word| word.starts_with('!'))
}

fn is_element_statement(statement: &RawStatement, scope: Scope) -> bool {
    if scope == Scope::Views {
        return false;
    }
    if let Some(equal) = statement.head.iter().position(|token| token.is_symbol("=")) {
        return statement
            .head
            .get(equal + 1)
            .is_some_and(|token| is_element_keyword(token.text()));
    }
    first_text(&statement.head).is_some_and(is_element_keyword)
        && matches!(scope, Scope::Model | Scope::Element | Scope::Workspace)
}

fn is_element_keyword(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "person"
            | "softwaresystem"
            | "container"
            | "component"
            | "customelement"
            | "element"
            | "deploymentnode"
            | "infrastructurenode"
            | "softwaresysteminstance"
            | "containerinstance"
    )
}

fn is_block_keyword(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "model"
            | "views"
            | "configuration"
            | "styles"
            | "themes"
            | "properties"
            | "perspectives"
            | "archetypes"
            | "group"
            | "deploymentenvironment"
            | "deploymentgroup"
            | "systemlandscape"
            | "systemcontext"
            | "container"
            | "component"
            | "dynamic"
            | "deployment"
            | "filtered"
            | "custom"
            | "branding"
            | "terminology"
            | "enterprise"
            | "animation"
    )
}

fn first_text(tokens: &[Token]) -> Option<&str> {
    tokens.first().map(Token::text)
}

fn token_value(token: &Token) -> String {
    token.text().to_owned()
}

fn has_symbol(tokens: &[Token], symbol: &str) -> bool {
    tokens.iter().any(|token| token.is_symbol(symbol))
}
