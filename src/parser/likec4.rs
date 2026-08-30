use crate::{
    ast::likec4::{
        LikeC4Document, LikeC4Element, LikeC4Extend, LikeC4KindDefinition, LikeC4Property,
        LikeC4Relationship, LikeC4Section, LikeC4SectionKind, LikeC4Statement, LikeC4Tag,
        LikeC4View,
    },
    lexer::{render_tokens, LexerConfig, Token},
    Format, Located, ParseError, ParseOptions, ParseResult, Span,
};
use std::collections::BTreeSet;

use super::tree::{parse_braced_tree, quoted_values, tokens_to_scalars, RawStatement};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Top,
    Specification,
    Model,
    Views,
    Global,
    Deployment,
    Element,
    View,
    Other,
}

#[derive(Debug, Default)]
struct Kinds {
    elements: BTreeSet<String>,
    relationships: BTreeSet<String>,
    deployment_nodes: BTreeSet<String>,
}

pub(crate) fn parse(source: &str, options: &ParseOptions) -> ParseResult<LikeC4Document> {
    let raw = parse_braced_tree(
        Format::LikeC4,
        source,
        LexerConfig::likec4(),
        options.max_nesting_depth,
    )?;
    let kinds = collect_kinds(&raw);
    let statements = convert_list(&raw, source, Scope::Top, &kinds)?;
    Ok(LikeC4Document {
        span: Span::new(0, source.len()),
        statements,
    })
}

fn collect_kinds(raw: &[RawStatement]) -> Kinds {
    let mut kinds = Kinds::default();
    for statement in raw {
        if first_text(&statement.head)
            .is_some_and(|word| word.eq_ignore_ascii_case("specification"))
        {
            if let Some(body) = &statement.body {
                for child in body {
                    let category = first_text(&child.head).unwrap_or_default();
                    let name = child.head.get(1).map(Token::text).unwrap_or_default();
                    if category.eq_ignore_ascii_case("element") && !name.is_empty() {
                        kinds.elements.insert(name.to_ascii_lowercase());
                    } else if category.eq_ignore_ascii_case("relationship") && !name.is_empty() {
                        kinds.relationships.insert(name.to_ascii_lowercase());
                    } else if category.eq_ignore_ascii_case("deploymentNode") && !name.is_empty() {
                        kinds.deployment_nodes.insert(name.to_ascii_lowercase());
                    }
                }
            }
        }
    }
    kinds
}

fn convert_list(
    raw: &[RawStatement],
    source: &str,
    scope: Scope,
    kinds: &Kinds,
) -> ParseResult<Vec<Located<LikeC4Statement>>> {
    raw.iter()
        .map(|statement| {
            convert_statement(statement, source, scope, kinds)
                .map(|node| Located::new(statement.span, node))
        })
        .collect()
}

fn convert_statement(
    statement: &RawStatement,
    source: &str,
    scope: Scope,
    kinds: &Kinds,
) -> ParseResult<LikeC4Statement> {
    let first = first_text(&statement.head).ok_or_else(|| {
        ParseError::at(
            Format::LikeC4,
            "empty LikeC4 statement",
            statement.span,
            source,
        )
    })?;

    if scope == Scope::Top {
        if let Some(section_kind) = section_kind(first) {
            let body = statement.body.as_deref().ok_or_else(|| {
                ParseError::at(
                    Format::LikeC4,
                    format!("{first} requires a braced body"),
                    statement.span,
                    source,
                )
            })?;
            let child_scope = match section_kind {
                LikeC4SectionKind::Specification => Scope::Specification,
                LikeC4SectionKind::Model => Scope::Model,
                LikeC4SectionKind::Views => Scope::Views,
                LikeC4SectionKind::Global => Scope::Global,
                LikeC4SectionKind::Deployment => Scope::Deployment,
            };
            return Ok(LikeC4Statement::Section(LikeC4Section {
                section: section_kind,
                body: convert_list(body, source, child_scope, kinds)?,
            }));
        }
    }

    if scope == Scope::Specification && is_kind_definition(first) {
        return parse_kind_definition(statement, source, kinds)
            .map(LikeC4Statement::KindDefinition);
    }

    if first.eq_ignore_ascii_case("extend") {
        return parse_extend(statement, source, scope, kinds).map(LikeC4Statement::Extend);
    }

    if statement
        .head
        .first()
        .is_some_and(|token| token.is_symbol("#"))
    {
        return Ok(LikeC4Statement::Tag(parse_tags(statement)));
    }

    if matches!(scope, Scope::Views | Scope::View) && is_view_statement(&statement.head) {
        return parse_view(statement, source, kinds).map(LikeC4Statement::View);
    }

    if has_arrow(&statement.head) || is_dot_kind_relationship(&statement.head, kinds) {
        return parse_relationship(statement, source, kinds).map(LikeC4Statement::Relationship);
    }

    if matches!(scope, Scope::Model | Scope::Element | Scope::Deployment)
        && is_element_statement(statement, scope, kinds)
    {
        return parse_element(statement, source, scope, kinds).map(LikeC4Statement::Element);
    }

    parse_property(statement, source, scope, kinds).map(LikeC4Statement::Property)
}

fn parse_kind_definition(
    statement: &RawStatement,
    source: &str,
    kinds: &Kinds,
) -> ParseResult<LikeC4KindDefinition> {
    let category = first_text(&statement.head).unwrap_or_default().to_owned();
    let name = statement.head.get(1).map(Token::text).ok_or_else(|| {
        ParseError::at(
            Format::LikeC4,
            format!("{category} kind definition requires a name"),
            statement.span,
            source,
        )
    })?;
    let body = statement
        .body
        .as_deref()
        .map(|body| convert_list(body, source, Scope::Other, kinds))
        .transpose()?
        .unwrap_or_default();
    Ok(LikeC4KindDefinition {
        category,
        name: name.to_owned(),
        body,
    })
}

fn parse_element(
    statement: &RawStatement,
    source: &str,
    scope: Scope,
    kinds: &Kinds,
) -> ParseResult<LikeC4Element> {
    let equal = statement.head.iter().position(|token| token.is_symbol("="));

    if scope == Scope::Deployment
        && statement
            .head
            .first()
            .is_some_and(|token| token.is_bare("instanceOf"))
    {
        let quoted_start = statement.head[1..]
            .iter()
            .position(Token::is_quoted)
            .map_or(statement.head.len(), |relative| relative + 1);
        if quoted_start <= 1 {
            return Err(ParseError::at(
                Format::LikeC4,
                "instanceOf requires a logical element reference",
                statement.span,
                source,
            ));
        }
        let reference = render_tokens(&statement.head[1..quoted_start]);
        let strings = quoted_values(&statement.head[quoted_start..]);
        let body = statement
            .body
            .as_deref()
            .map(|body| convert_list(body, source, Scope::Element, kinds))
            .transpose()?
            .unwrap_or_default();
        return Ok(LikeC4Element {
            name: reference
                .rsplit('.')
                .next()
                .unwrap_or(reference.as_str())
                .to_owned(),
            element_type: "instanceOf".to_owned(),
            title: strings.first().cloned(),
            description: strings.get(1).cloned(),
            reference: Some(reference),
            body,
        });
    }

    let (name_index, kind_index, argument_start, reference_start) = if let Some(equal) = equal {
        if equal != 1 || equal + 1 >= statement.head.len() {
            return Err(ParseError::at(
                Format::LikeC4,
                "element assignment must be `name = kind ...`",
                statement.span,
                source,
            ));
        }
        let reference_start = if statement.head[equal + 1].is_bare("instanceOf") {
            Some(equal + 2)
        } else {
            None
        };
        (0, equal + 1, equal + 2, reference_start)
    } else {
        if statement.head.len() < 2 {
            return Err(ParseError::at(
                Format::LikeC4,
                "element declaration must contain both a kind and a name",
                statement.span,
                source,
            ));
        }
        (1, 0, 2, None)
    };
    let name = statement.head[name_index].text().to_owned();
    let element_type = statement.head[kind_index].text().to_owned();
    let quoted_start = statement.head[argument_start..]
        .iter()
        .position(Token::is_quoted)
        .map_or(statement.head.len(), |relative| argument_start + relative);
    let strings = quoted_values(&statement.head[quoted_start..]);
    let reference =
        reference_start.map(|start| render_tokens(&statement.head[start..quoted_start]));
    let body = statement
        .body
        .as_deref()
        .map(|body| convert_list(body, source, Scope::Element, kinds))
        .transpose()?
        .unwrap_or_default();

    Ok(LikeC4Element {
        name,
        element_type,
        title: strings.first().cloned(),
        description: strings.get(1).cloned(),
        reference,
        body,
    })
}

fn parse_relationship(
    statement: &RawStatement,
    source: &str,
    kinds: &Kinds,
) -> ParseResult<LikeC4Relationship> {
    if let Some(arrow) = statement
        .head
        .iter()
        .position(|token| token.is_symbol("->"))
    {
        let mut relationship_type = None;
        let mut source_end = arrow;
        if arrow >= 4
            && statement.head[arrow - 1].is_symbol("]")
            && statement.head[arrow - 3].is_symbol("[")
            && statement.head[arrow - 4].is_symbol("-")
        {
            relationship_type = Some(statement.head[arrow - 2].text().to_owned());
            source_end = arrow - 4;
        }
        let source_name = if source_end == 0 {
            None
        } else {
            Some(render_tokens(&statement.head[..source_end]))
        };
        let target_end = statement.head[arrow + 1..]
            .iter()
            .position(Token::is_quoted)
            .map_or(statement.head.len(), |relative| arrow + 1 + relative);
        if arrow + 1 >= target_end {
            return Err(ParseError::at(
                Format::LikeC4,
                "relationship requires a target",
                statement.span,
                source,
            ));
        }
        let strings = quoted_values(&statement.head[target_end..]);
        let body = statement
            .body
            .as_deref()
            .map(|body| convert_list(body, source, Scope::Other, kinds))
            .transpose()?
            .unwrap_or_default();
        return Ok(LikeC4Relationship {
            source: source_name,
            target: render_tokens(&statement.head[arrow + 1..target_end]),
            relationship_type,
            title: strings.first().cloned(),
            description: strings.get(1).cloned(),
            body,
        });
    }

    let dot = statement
        .head
        .windows(2)
        .position(|window| {
            window[0].is_symbol(".")
                && kinds
                    .relationships
                    .contains(&window[1].text().to_ascii_lowercase())
        })
        .ok_or_else(|| {
            ParseError::at(
                Format::LikeC4,
                "relationship requires `->` or `.kind` syntax",
                statement.span,
                source,
            )
        })?;
    if dot == 0 || dot + 2 >= statement.head.len() {
        return Err(ParseError::at(
            Format::LikeC4,
            "invalid `.kind` relationship",
            statement.span,
            source,
        ));
    }
    let target_end = statement.head[dot + 2..]
        .iter()
        .position(Token::is_quoted)
        .map_or(statement.head.len(), |relative| dot + 2 + relative);
    if target_end == dot + 2 {
        return Err(ParseError::at(
            Format::LikeC4,
            "`.kind` relationship requires a target",
            statement.span,
            source,
        ));
    }
    let strings = quoted_values(&statement.head[target_end..]);
    let body = statement
        .body
        .as_deref()
        .map(|body| convert_list(body, source, Scope::Other, kinds))
        .transpose()?
        .unwrap_or_default();
    Ok(LikeC4Relationship {
        source: Some(render_tokens(&statement.head[..dot])),
        relationship_type: Some(statement.head[dot + 1].text().to_owned()),
        target: render_tokens(&statement.head[dot + 2..target_end]),
        title: strings.first().cloned(),
        description: strings.get(1).cloned(),
        body,
    })
}

fn parse_view(statement: &RawStatement, source: &str, kinds: &Kinds) -> ParseResult<LikeC4View> {
    let view_index = statement
        .head
        .iter()
        .position(|token| token.is_bare("view"))
        .ok_or_else(|| {
            ParseError::at(
                Format::LikeC4,
                "view declaration requires the `view` keyword",
                statement.span,
                source,
            )
        })?;
    let view_type = if view_index == 0 {
        "view".to_owned()
    } else {
        format!("{} view", render_tokens(&statement.head[..view_index]))
    };
    let of_index = statement.head.iter().position(|token| token.is_bare("of"));
    let name = match of_index {
        Some(index) if index == view_index + 1 => None,
        _ => statement
            .head
            .get(view_index + 1)
            .filter(|token| !token.is_bare("of") && !token.is_quoted())
            .map(Token::text)
            .map(str::to_owned),
    };
    let scope = of_index
        .filter(|index| index + 1 < statement.head.len())
        .map(|index| render_tokens(&statement.head[index + 1..]));
    let body = statement
        .body
        .as_deref()
        .map(|body| convert_list(body, source, Scope::View, kinds))
        .transpose()?
        .unwrap_or_default();
    Ok(LikeC4View {
        view_type,
        name,
        scope,
        body,
    })
}

fn parse_extend(
    statement: &RawStatement,
    source: &str,
    scope: Scope,
    kinds: &Kinds,
) -> ParseResult<LikeC4Extend> {
    if statement.head.len() < 2 {
        return Err(ParseError::at(
            Format::LikeC4,
            "extend requires a target",
            statement.span,
            source,
        ));
    }
    let body = statement
        .body
        .as_deref()
        .map(|body| convert_list(body, source, scope, kinds))
        .transpose()?
        .unwrap_or_default();
    Ok(LikeC4Extend {
        target: render_tokens(&statement.head[1..]),
        body,
    })
}

fn parse_tags(statement: &RawStatement) -> LikeC4Tag {
    let mut names = Vec::new();
    let mut index = 0usize;
    while index < statement.head.len() {
        if statement.head[index].is_symbol("#") {
            if let Some(name) = statement.head.get(index + 1) {
                names.push(name.text().to_owned());
                index += 2;
                continue;
            }
        }
        index += 1;
    }
    LikeC4Tag { names }
}

fn parse_property(
    statement: &RawStatement,
    source: &str,
    _scope: Scope,
    kinds: &Kinds,
) -> ParseResult<LikeC4Property> {
    let name = first_text(&statement.head).unwrap_or_default().to_owned();
    let value_start = statement
        .head
        .get(1)
        .is_some_and(|token| token.is_symbol(":"))
        .then_some(2)
        .unwrap_or(1);
    let body = statement
        .body
        .as_deref()
        .map(|body| convert_list(body, source, Scope::Other, kinds))
        .transpose()?
        .unwrap_or_default();
    Ok(LikeC4Property {
        name,
        values: tokens_to_scalars(&statement.head[value_start..]),
        body,
    })
}

fn is_element_statement(statement: &RawStatement, scope: Scope, kinds: &Kinds) -> bool {
    if let Some(equal) = statement.head.iter().position(|token| token.is_symbol("=")) {
        return statement.head.get(equal + 1).is_some_and(|token| {
            is_known_element_kind(token.text(), scope, kinds)
                || (scope == Scope::Deployment && token.is_bare("instanceOf"))
        });
    }
    statement.head.first().is_some_and(|token| {
        is_known_element_kind(token.text(), scope, kinds)
            || (scope == Scope::Deployment && token.is_bare("instanceOf"))
    })
}

fn is_known_element_kind(word: &str, scope: Scope, kinds: &Kinds) -> bool {
    let lower = word.to_ascii_lowercase();
    if scope == Scope::Deployment {
        kinds.deployment_nodes.contains(&lower)
    } else if kinds.elements.is_empty() {
        !is_reserved_property(word)
    } else {
        kinds.elements.contains(&lower)
    }
}

fn is_reserved_property(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "title"
            | "description"
            | "summary"
            | "technology"
            | "notation"
            | "link"
            | "links"
            | "metadata"
            | "style"
            | "tags"
            | "navigateto"
            | "include"
            | "exclude"
            | "autolayout"
            | "rank"
            | "where"
            | "with"
    )
}

fn is_dot_kind_relationship(tokens: &[Token], kinds: &Kinds) -> bool {
    tokens.windows(3).any(|window| {
        window[1].is_symbol(".")
            && kinds
                .relationships
                .contains(&window[2].text().to_ascii_lowercase())
    })
}

fn has_arrow(tokens: &[Token]) -> bool {
    tokens.iter().any(|token| token.is_symbol("->"))
}

fn is_view_statement(tokens: &[Token]) -> bool {
    tokens.iter().any(|token| token.is_bare("view"))
}

fn is_kind_definition(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "element" | "relationship" | "deploymentnode"
    )
}

fn section_kind(word: &str) -> Option<LikeC4SectionKind> {
    if word.eq_ignore_ascii_case("specification") {
        Some(LikeC4SectionKind::Specification)
    } else if word.eq_ignore_ascii_case("model") {
        Some(LikeC4SectionKind::Model)
    } else if word.eq_ignore_ascii_case("views") {
        Some(LikeC4SectionKind::Views)
    } else if word.eq_ignore_ascii_case("global") {
        Some(LikeC4SectionKind::Global)
    } else if word.eq_ignore_ascii_case("deployment") {
        Some(LikeC4SectionKind::Deployment)
    } else {
        None
    }
}

fn first_text(tokens: &[Token]) -> Option<&str> {
    tokens.first().map(Token::text)
}
