use crate::{
    ast::dbml::{
        DbmlCardinality, DbmlCheck, DbmlColumn, DbmlDocument, DbmlEndpoint, DbmlEnum,
        DbmlEnumValue, DbmlIndex, DbmlItem, DbmlProject, DbmlProperty, DbmlRef, DbmlSetting,
        DbmlTable, DbmlTableGroup, DbmlTableItem, DbmlTablePartial, DbmlValue,
    },
    lexer::{render_tokens, LexerConfig, Token, TokenKind},
    Format, Located, ParseError, ParseOptions, ParseResult, Span,
};

use super::tree::{first_word, parse_braced_tree, RawStatement};

pub(crate) fn parse(source: &str, options: &ParseOptions) -> ParseResult<DbmlDocument> {
    let raw = parse_braced_tree(
        Format::Dbml,
        source,
        LexerConfig::dbml(),
        options.max_nesting_depth,
    )?;
    let items = raw
        .iter()
        .map(|statement| {
            parse_item(statement, source).map(|node| Located::new(statement.span, node))
        })
        .collect::<ParseResult<Vec<_>>>()?;
    Ok(DbmlDocument {
        span: Span::new(0, source.len()),
        items,
    })
}

fn parse_item(statement: &RawStatement, source: &str) -> ParseResult<DbmlItem> {
    let keyword = first_word(&statement.head).ok_or_else(|| {
        ParseError::at(
            Format::Dbml,
            "expected a DBML declaration keyword",
            statement.span,
            source,
        )
    })?;

    if keyword.eq_ignore_ascii_case("project") {
        return parse_project(statement, source).map(DbmlItem::Project);
    }
    if keyword.eq_ignore_ascii_case("table") {
        return parse_table(statement, source).map(DbmlItem::Table);
    }
    if keyword.eq_ignore_ascii_case("tablepartial") {
        return parse_table_partial(statement, source).map(DbmlItem::TablePartial);
    }
    if keyword.eq_ignore_ascii_case("enum") {
        return parse_enum(statement, source).map(DbmlItem::Enum);
    }
    if keyword.eq_ignore_ascii_case("ref") {
        return parse_ref(statement, source).map(DbmlItem::Ref);
    }
    if keyword.eq_ignore_ascii_case("tablegroup") {
        return parse_table_group(statement, source).map(DbmlItem::TableGroup);
    }
    if keyword.eq_ignore_ascii_case("note") {
        return Ok(DbmlItem::Note(parse_colon_value(
            &statement.head,
            1,
            statement.span,
            source,
        )?));
    }

    Err(ParseError::at(
        Format::Dbml,
        format!("unsupported top-level DBML declaration `{keyword}`"),
        statement.span,
        source,
    ))
}

fn parse_project(statement: &RawStatement, source: &str) -> ParseResult<DbmlProject> {
    let name = statement
        .head
        .get(1)
        .map(token_value)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ParseError::at(
                Format::Dbml,
                "Project requires a name",
                statement.span,
                source,
            )
        })?;
    let body = required_body(statement, "Project", source)?;
    let properties = body
        .iter()
        .map(|child| parse_property(child, source).map(|node| Located::new(child.span, node)))
        .collect::<ParseResult<Vec<_>>>()?;
    Ok(DbmlProject { name, properties })
}

fn parse_property(statement: &RawStatement, source: &str) -> ParseResult<DbmlProperty> {
    let colon = find_top_level_symbol(&statement.head, ":").ok_or_else(|| {
        ParseError::at(
            Format::Dbml,
            "project/group property must use `name: value`",
            statement.span,
            source,
        )
    })?;
    if colon == 0 || colon + 1 >= statement.head.len() {
        return Err(ParseError::at(
            Format::Dbml,
            "property requires both a name and a value",
            statement.span,
            source,
        ));
    }
    Ok(DbmlProperty {
        name: render_tokens(&statement.head[..colon]),
        value: parse_value(&statement.head[colon + 1..]),
    })
}

fn parse_table(statement: &RawStatement, source: &str) -> ParseResult<DbmlTable> {
    let (name_tokens, alias, settings) = parse_named_header(&statement.head[1..], source)?;
    let (schema, name) = split_qualified_name(&render_tokens(name_tokens));
    let body = required_body(statement, "Table", source)?;
    let items = body
        .iter()
        .map(|child| parse_table_item(child, source).map(|node| Located::new(child.span, node)))
        .collect::<ParseResult<Vec<_>>>()?;
    Ok(DbmlTable {
        schema,
        name,
        alias,
        settings,
        items,
    })
}

fn parse_table_partial(statement: &RawStatement, source: &str) -> ParseResult<DbmlTablePartial> {
    let name = statement
        .head
        .get(1)
        .map(token_value)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ParseError::at(
                Format::Dbml,
                "TablePartial requires a name",
                statement.span,
                source,
            )
        })?;
    let body = required_body(statement, "TablePartial", source)?;
    let items = body
        .iter()
        .map(|child| parse_table_item(child, source).map(|node| Located::new(child.span, node)))
        .collect::<ParseResult<Vec<_>>>()?;
    Ok(DbmlTablePartial { name, items })
}

fn parse_enum(statement: &RawStatement, source: &str) -> ParseResult<DbmlEnum> {
    if statement.head.len() < 2 {
        return Err(ParseError::at(
            Format::Dbml,
            "Enum requires a name",
            statement.span,
            source,
        ));
    }
    let (schema, name) = split_qualified_name(&render_tokens(&statement.head[1..]));
    let body = required_body(statement, "Enum", source)?;
    let values = body
        .iter()
        .map(|child| parse_enum_value(child, source).map(|node| Located::new(child.span, node)))
        .collect::<ParseResult<Vec<_>>>()?;
    Ok(DbmlEnum {
        schema,
        name,
        values,
    })
}

fn parse_enum_value(statement: &RawStatement, source: &str) -> ParseResult<DbmlEnumValue> {
    let Some(first) = statement.head.first() else {
        return Err(ParseError::at(
            Format::Dbml,
            "empty enum value",
            statement.span,
            source,
        ));
    };
    let settings_start = find_trailing_settings_start(&statement.head);
    let settings = settings_start
        .map(|index| parse_settings(&statement.head[index..], source))
        .transpose()?
        .unwrap_or_default();
    Ok(DbmlEnumValue {
        name: token_value(first),
        settings,
    })
}

fn parse_table_item(statement: &RawStatement, source: &str) -> ParseResult<DbmlTableItem> {
    if statement
        .head
        .first()
        .is_some_and(|token| token.is_symbol("~"))
    {
        let name = statement.head.get(1).map(token_value).ok_or_else(|| {
            ParseError::at(
                Format::Dbml,
                "table partial inclusion requires a partial name",
                statement.span,
                source,
            )
        })?;
        return Ok(DbmlTableItem::Partial(name));
    }

    let keyword = first_word(&statement.head).unwrap_or_default();
    if keyword.eq_ignore_ascii_case("indexes") {
        let body = required_body(statement, "Indexes", source)?;
        let indexes = body
            .iter()
            .map(|child| parse_index(child, source).map(|node| Located::new(child.span, node)))
            .collect::<ParseResult<Vec<_>>>()?;
        return Ok(DbmlTableItem::Indexes(indexes));
    }
    if keyword.eq_ignore_ascii_case("note") {
        return Ok(DbmlTableItem::Note(parse_colon_value(
            &statement.head,
            1,
            statement.span,
            source,
        )?));
    }
    if keyword.eq_ignore_ascii_case("checks") {
        let body = required_body(statement, "Checks", source)?;
        let checks = body
            .iter()
            .map(|child| {
                parse_check(child, source, false).map(|node| Located::new(child.span, node))
            })
            .collect::<ParseResult<Vec<_>>>()?;
        return Ok(DbmlTableItem::Checks(checks));
    }
    if keyword.eq_ignore_ascii_case("check") {
        return parse_check(statement, source, true).map(DbmlTableItem::Check);
    }

    parse_column(statement, source).map(DbmlTableItem::Column)
}

fn parse_column(statement: &RawStatement, source: &str) -> ParseResult<DbmlColumn> {
    let name = statement.head.first().map(token_value).ok_or_else(|| {
        ParseError::at(
            Format::Dbml,
            "column requires a name",
            statement.span,
            source,
        )
    })?;
    let settings_start = find_trailing_settings_start(&statement.head);
    let type_end = settings_start.unwrap_or(statement.head.len());
    if type_end <= 1 {
        return Err(ParseError::at(
            Format::Dbml,
            format!("column `{name}` requires a data type"),
            statement.span,
            source,
        ));
    }
    let settings = settings_start
        .map(|index| parse_settings(&statement.head[index..], source))
        .transpose()?
        .unwrap_or_default();
    Ok(DbmlColumn {
        name,
        data_type: render_tokens(&statement.head[1..type_end]),
        settings,
    })
}

fn parse_index(statement: &RawStatement, source: &str) -> ParseResult<DbmlIndex> {
    let settings_start = find_trailing_settings_start(&statement.head);
    let expression_end = settings_start.unwrap_or(statement.head.len());
    let expression_tokens = &statement.head[..expression_end];
    if expression_tokens.is_empty() {
        return Err(ParseError::at(
            Format::Dbml,
            "index requires at least one expression",
            statement.span,
            source,
        ));
    }
    let expressions = if expression_tokens.first().is_some_and(|t| t.is_symbol("("))
        && expression_tokens.last().is_some_and(|t| t.is_symbol(")"))
    {
        split_top_level(&expression_tokens[1..expression_tokens.len() - 1], ",")
            .into_iter()
            .map(render_tokens)
            .filter(|value| !value.trim().is_empty())
            .collect()
    } else {
        vec![render_tokens(expression_tokens)]
    };
    let settings = settings_start
        .map(|index| parse_settings(&statement.head[index..], source))
        .transpose()?
        .unwrap_or_default();
    Ok(DbmlIndex {
        expressions,
        settings,
    })
}

fn parse_check(
    statement: &RawStatement,
    source: &str,
    has_keyword: bool,
) -> ParseResult<DbmlCheck> {
    let expression_start = if has_keyword { 1 } else { 0 };
    let settings_start = find_trailing_settings_start(&statement.head);
    let expression_end = settings_start.unwrap_or(statement.head.len());
    if expression_end <= expression_start {
        return Err(ParseError::at(
            Format::Dbml,
            "Check requires an expression",
            statement.span,
            source,
        ));
    }
    let settings = settings_start
        .map(|index| parse_settings(&statement.head[index..], source))
        .transpose()?
        .unwrap_or_default();
    let expression_tokens = &statement.head[expression_start..expression_end];
    let expression = if expression_tokens.len() == 1 {
        match &expression_tokens[0].kind {
            TokenKind::Quoted {
                value, delimiter, ..
            } if *delimiter == '`' => value.clone(),
            _ => render_tokens(expression_tokens),
        }
    } else {
        render_tokens(expression_tokens)
    };
    Ok(DbmlCheck {
        expression,
        settings,
    })
}

fn parse_ref(statement: &RawStatement, source: &str) -> ParseResult<DbmlRef> {
    let relationship_tokens = if let Some(body) = &statement.body {
        let child = body.first().ok_or_else(|| {
            ParseError::at(
                Format::Dbml,
                "Ref block must contain a relationship",
                statement.span,
                source,
            )
        })?;
        child.head.clone()
    } else {
        statement.head[1..].to_vec()
    };
    let tokens = relationship_tokens.as_slice();

    let block_name = statement
        .body
        .as_ref()
        .filter(|_| statement.head.len() > 1)
        .map(|_| render_tokens(&statement.head[1..]));
    let colon = find_top_level_symbol(tokens, ":");
    let (name, relation_start) = match colon {
        Some(index) if index > 0 => (Some(render_tokens(&tokens[..index])), index + 1),
        Some(index) => (block_name, index + 1),
        None => (block_name, 0),
    };
    let relation = &tokens[relation_start..];
    let settings_start = find_trailing_settings_start(relation);
    let relation_end = settings_start.unwrap_or(relation.len());
    let core = &relation[..relation_end];
    let (operator_index, cardinality) = find_ref_operator(core).ok_or_else(|| {
        ParseError::at(
            Format::Dbml,
            "Ref requires one of `>`, `<`, `-`, or `<>`",
            statement.span,
            source,
        )
    })?;
    if operator_index == 0 || operator_index + 1 >= core.len() {
        return Err(ParseError::at(
            Format::Dbml,
            "Ref requires endpoints on both sides of the cardinality operator",
            statement.span,
            source,
        ));
    }
    let settings = settings_start
        .map(|index| parse_settings(&relation[index..], source))
        .transpose()?
        .unwrap_or_default();
    Ok(DbmlRef {
        name,
        from: parse_endpoint(&core[..operator_index], statement.span, source)?,
        cardinality,
        to: parse_endpoint(&core[operator_index + 1..], statement.span, source)?,
        settings,
    })
}

fn parse_endpoint(tokens: &[Token], span: Span, source: &str) -> ParseResult<DbmlEndpoint> {
    let raw = render_tokens(tokens).replace(' ', "");
    if raw.is_empty() {
        return Err(ParseError::at(
            Format::Dbml,
            "empty reference endpoint",
            span,
            source,
        ));
    }

    if let Some(open) = raw.find(".(") {
        if !raw.ends_with(')') {
            return Err(ParseError::at(
                Format::Dbml,
                "composite reference endpoint has an unterminated column list",
                span,
                source,
            ));
        }
        let table_path = &raw[..open];
        let columns = raw[open + 2..raw.len() - 1]
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let (schema, table) = split_qualified_name(table_path);
        return Ok(DbmlEndpoint {
            schema,
            table,
            columns,
        });
    }

    let mut parts = raw.split('.').map(str::to_owned).collect::<Vec<_>>();
    if parts.len() < 2 {
        return Err(ParseError::at(
            Format::Dbml,
            "reference endpoint must be `table.column` or `schema.table.column`",
            span,
            source,
        ));
    }
    let column = parts.pop().expect("length checked");
    let table = parts.pop().expect("length checked");
    let schema = if parts.is_empty() {
        None
    } else {
        Some(parts.join("."))
    };
    Ok(DbmlEndpoint {
        schema,
        table,
        columns: vec![column],
    })
}

fn parse_table_group(statement: &RawStatement, source: &str) -> ParseResult<DbmlTableGroup> {
    let name = statement.head.get(1).map(token_value).ok_or_else(|| {
        ParseError::at(
            Format::Dbml,
            "TableGroup requires a name",
            statement.span,
            source,
        )
    })?;
    let body = required_body(statement, "TableGroup", source)?;
    let mut tables = Vec::new();
    let mut properties = Vec::new();
    for child in body {
        if find_top_level_symbol(&child.head, ":").is_some() {
            properties.push(Located::new(child.span, parse_property(child, source)?));
        } else {
            tables.push(render_tokens(&child.head));
        }
    }
    Ok(DbmlTableGroup {
        name,
        tables,
        properties,
    })
}

fn parse_named_header<'a>(
    tokens: &'a [Token],
    source: &str,
) -> ParseResult<(&'a [Token], Option<String>, Vec<DbmlSetting>)> {
    if tokens.is_empty() {
        return Err(ParseError::new(
            Format::Dbml,
            "Table requires a name",
            None,
            source,
        ));
    }
    let settings_start = find_trailing_settings_start(tokens);
    let main_end = settings_start.unwrap_or(tokens.len());
    let alias_index = tokens[..main_end]
        .iter()
        .position(|token| token.is_bare("as"));
    let name_end = alias_index.unwrap_or(main_end);
    let name_tokens = &tokens[..name_end];
    if name_tokens.is_empty() {
        return Err(ParseError::new(
            Format::Dbml,
            "Table requires a name",
            None,
            source,
        ));
    }
    let alias = alias_index
        .and_then(|index| tokens.get(index + 1))
        .map(token_value);
    if alias_index.is_some() && alias.is_none() {
        return Err(ParseError::new(
            Format::Dbml,
            "`as` requires a table alias",
            None,
            source,
        ));
    }
    let settings = settings_start
        .map(|index| parse_settings(&tokens[index..], source))
        .transpose()?
        .unwrap_or_default();
    Ok((name_tokens, alias, settings))
}

fn parse_settings(tokens: &[Token], source: &str) -> ParseResult<Vec<DbmlSetting>> {
    if !tokens.first().is_some_and(|token| token.is_symbol("["))
        || !tokens.last().is_some_and(|token| token.is_symbol("]"))
    {
        let span = tokens
            .first()
            .map_or(Span::new(0, 0), |first| first.span)
            .join(tokens.last().map_or(Span::new(0, 0), |last| last.span));
        return Err(ParseError::at(
            Format::Dbml,
            "settings must be enclosed in `[...]`",
            span,
            source,
        ));
    }
    split_top_level(&tokens[1..tokens.len() - 1], ",")
        .into_iter()
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            if let Some(colon) = find_top_level_symbol(segment, ":") {
                let name = render_tokens(&segment[..colon]);
                let value = if colon + 1 < segment.len() {
                    Some(parse_value(&segment[colon + 1..]))
                } else {
                    None
                };
                Ok(DbmlSetting { name, value })
            } else {
                Ok(DbmlSetting {
                    name: render_tokens(segment),
                    value: None,
                })
            }
        })
        .collect::<ParseResult<Vec<_>>>()
}

fn parse_value(tokens: &[Token]) -> DbmlValue {
    if tokens.len() == 1 {
        match &tokens[0].kind {
            TokenKind::Quoted {
                value, delimiter, ..
            } if *delimiter == '`' => return DbmlValue::Expression(value.clone()),
            TokenKind::Quoted { value, .. } => return DbmlValue::String(value.clone()),
            TokenKind::Bare(value) if value.eq_ignore_ascii_case("true") => {
                return DbmlValue::Boolean(true)
            }
            TokenKind::Bare(value) if value.eq_ignore_ascii_case("false") => {
                return DbmlValue::Boolean(false)
            }
            TokenKind::Bare(value) if value.parse::<f64>().is_ok() => {
                return DbmlValue::Number(value.clone())
            }
            TokenKind::Bare(value) => return DbmlValue::Identifier(value.clone()),
            TokenKind::Symbol(value) => return DbmlValue::Raw(value.clone()),
            TokenKind::Newline => return DbmlValue::Raw(String::new()),
        }
    }
    DbmlValue::Raw(render_tokens(tokens))
}

fn parse_colon_value(
    tokens: &[Token],
    search_from: usize,
    span: Span,
    source: &str,
) -> ParseResult<String> {
    let colon = tokens[search_from..]
        .iter()
        .position(|token| token.is_symbol(":"))
        .map(|relative| search_from + relative)
        .ok_or_else(|| {
            ParseError::at(
                Format::Dbml,
                "expected `:` followed by a value",
                span,
                source,
            )
        })?;
    if colon + 1 >= tokens.len() {
        return Err(ParseError::at(
            Format::Dbml,
            "expected a value after `:`",
            tokens.get(colon).map_or(span, |token| token.span),
            source,
        ));
    }
    Ok(match parse_value(&tokens[colon + 1..]) {
        DbmlValue::String(value)
        | DbmlValue::Expression(value)
        | DbmlValue::Identifier(value)
        | DbmlValue::Number(value)
        | DbmlValue::Raw(value) => value,
        DbmlValue::Boolean(value) => value.to_string(),
    })
}

fn required_body<'a>(
    statement: &'a RawStatement,
    declaration: &str,
    source: &str,
) -> ParseResult<&'a [RawStatement]> {
    statement.body.as_deref().ok_or_else(|| {
        ParseError::at(
            Format::Dbml,
            format!("{declaration} requires a braced body"),
            statement.span,
            source,
        )
    })
}

fn token_value(token: &Token) -> String {
    token.text().to_owned()
}

fn split_qualified_name(raw: &str) -> (Option<String>, String) {
    let clean = raw.trim().trim_matches('`').trim_matches('"');
    match clean.rsplit_once('.') {
        Some((schema, name)) => (Some(schema.to_owned()), name.to_owned()),
        None => (None, clean.to_owned()),
    }
}

fn find_ref_operator(tokens: &[Token]) -> Option<(usize, DbmlCardinality)> {
    tokens.iter().enumerate().find_map(|(index, token)| {
        if token.is_symbol(">") {
            Some((index, DbmlCardinality::ManyToOne))
        } else if token.is_symbol("<") {
            Some((index, DbmlCardinality::OneToMany))
        } else if token.is_symbol("-") {
            Some((index, DbmlCardinality::OneToOne))
        } else if token.is_symbol("<>") {
            Some((index, DbmlCardinality::ManyToMany))
        } else {
            None
        }
    })
}

fn find_trailing_settings_start(tokens: &[Token]) -> Option<usize> {
    if !tokens.last().is_some_and(|token| token.is_symbol("]")) {
        return None;
    }

    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().rev() {
        if token.is_symbol("]") {
            depth += 1;
        } else if token.is_symbol("[") {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                // An empty trailing `[]` is treated as an array type suffix, not settings.
                return (index + 1 < tokens.len() - 1).then_some(index);
            }
        }
    }
    None
}

fn find_top_level_symbol(tokens: &[Token], symbol: &str) -> Option<usize> {
    let mut paren = 0usize;
    let mut square = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if token.is_symbol("(") {
            paren += 1;
        } else if token.is_symbol(")") {
            paren = paren.saturating_sub(1);
        } else if token.is_symbol("[") {
            if symbol == "[" && paren == 0 && square == 0 {
                return Some(index);
            }
            square += 1;
        } else if token.is_symbol("]") {
            square = square.saturating_sub(1);
        } else if paren == 0 && square == 0 && token.is_symbol(symbol) {
            return Some(index);
        }
    }
    None
}

fn split_top_level<'a>(tokens: &'a [Token], separator: &str) -> Vec<&'a [Token]> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut paren = 0usize;
    let mut square = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if token.is_symbol("(") {
            paren += 1;
        } else if token.is_symbol(")") {
            paren = paren.saturating_sub(1);
        } else if token.is_symbol("[") {
            square += 1;
        } else if token.is_symbol("]") {
            square = square.saturating_sub(1);
        } else if paren == 0 && square == 0 && token.is_symbol(separator) {
            parts.push(&tokens[start..index]);
            start = index + 1;
        }
    }
    parts.push(&tokens[start..]);
    parts
}
