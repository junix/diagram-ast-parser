use crate::{
    ast::nomnoml::{
        NomnomlClassifier, NomnomlCompartment, NomnomlDirective, NomnomlDocument, NomnomlRelation,
        NomnomlStatement,
    },
    Format, Located, ParseError, ParseResult, Span,
};
use std::collections::BTreeMap;

pub(crate) fn parse(source: &str) -> ParseResult<NomnomlDocument> {
    let raw_statements = split_source_statements(source)?;
    let mut directives = Vec::new();
    let mut statements = Vec::new();

    for raw in raw_statements {
        let text = source[raw.span.start..raw.span.end].trim();
        if text.starts_with('#') {
            directives.push(Located::new(
                raw.span,
                parse_directive(text, raw.span, source)?,
            ));
        } else {
            statements.push(Located::new(
                raw.span,
                parse_diagram_statement(text, raw.span, source)?,
            ));
        }
    }

    Ok(NomnomlDocument {
        span: Span::new(0, source.len()),
        directives,
        statements,
    })
}

#[derive(Debug, Clone, Copy)]
struct RawSourceStatement {
    span: Span,
}

fn split_source_statements(source: &str) -> ParseResult<Vec<RawSourceStatement>> {
    let mut statements = Vec::new();
    let mut current_start: Option<usize> = None;
    let mut bracket_depth = 0isize;
    let mut offset = 0usize;

    for segment in source.split_inclusive('\n') {
        let line_start = offset;
        let line_end = offset + segment.len();
        let line_without_newline = segment.strip_suffix('\n').unwrap_or(segment);
        let trimmed = line_without_newline.trim();

        if current_start.is_none() && (trimmed.is_empty() || trimmed.starts_with("//")) {
            offset = line_end;
            continue;
        }

        current_start.get_or_insert_with(|| {
            line_start
                + line_without_newline
                    .len()
                    .saturating_sub(line_without_newline.trim_start().len())
        });

        bracket_depth += bracket_delta(line_without_newline);
        if bracket_depth < 0 {
            return Err(ParseError::at(
                Format::Nomnoml,
                "unmatched closing classifier bracket",
                Span::new(line_start, line_end),
                source,
            ));
        }

        if bracket_depth == 0 {
            let start = current_start.take().expect("set above");
            let mut end = line_start + line_without_newline.len();
            while end > start && source.as_bytes()[end - 1].is_ascii_whitespace() {
                end -= 1;
            }
            if end > start {
                statements.push(RawSourceStatement {
                    span: Span::new(start, end),
                });
            }
        }
        offset = line_end;
    }

    if bracket_depth != 0 {
        return Err(ParseError::new(
            Format::Nomnoml,
            "unterminated classifier bracket",
            current_start.map(|start| Span::new(start, source.len())),
            source,
        ));
    }

    if let Some(start) = current_start {
        let end = source.trim_end().len();
        if end > start {
            statements.push(RawSourceStatement {
                span: Span::new(start, end),
            });
        }
    }

    Ok(statements)
}

fn bracket_delta(line: &str) -> isize {
    let mut delta = 0isize;
    let mut escaped = false;
    for ch in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
        } else if ch == '[' {
            delta += 1;
        } else if ch == ']' {
            delta -= 1;
        }
    }
    delta
}

fn parse_directive(text: &str, span: Span, source: &str) -> ParseResult<NomnomlDirective> {
    let content = text.trim_start_matches('#');
    let (name, value) = content.split_once(':').ok_or_else(|| {
        ParseError::at(
            Format::Nomnoml,
            "directive must use `#name: value`",
            span,
            source,
        )
    })?;
    let name = name.trim();
    if name.is_empty() {
        return Err(ParseError::at(
            Format::Nomnoml,
            "directive name cannot be empty",
            span,
            source,
        ));
    }
    Ok(NomnomlDirective {
        custom_style: name.starts_with('.'),
        name: name.to_owned(),
        value: value.trim().to_owned(),
    })
}

fn parse_diagram_statement(text: &str, span: Span, source: &str) -> ParseResult<NomnomlStatement> {
    let classifiers = extract_top_level_classifiers(text, span, source)?;
    match classifiers.as_slice() {
        [] => Err(ParseError::at(
            Format::Nomnoml,
            "expected a classifier enclosed in `[...]`",
            span,
            source,
        )),
        [only] => {
            let before = text[..only.0].trim();
            let after = text[only.1..].trim();
            if !before.is_empty() || !after.is_empty() {
                return Err(ParseError::at(
                    Format::Nomnoml,
                    "unexpected text around standalone classifier",
                    span,
                    source,
                ));
            }
            Ok(NomnomlStatement::Classifier(parse_classifier(
                &text[only.0 + 1..only.1 - 1],
            )))
        }
        [start, end] => {
            let before = text[..start.0].trim();
            let after = text[end.1..].trim();
            if !before.is_empty() || !after.is_empty() {
                return Err(ParseError::at(
                    Format::Nomnoml,
                    "unexpected text outside relation classifiers",
                    span,
                    source,
                ));
            }
            let raw_middle = text[start.1..end.0].trim().to_owned();
            let (association, start_label, end_label) =
                parse_relation_middle(&raw_middle, span, source)?;
            Ok(NomnomlStatement::Relation(NomnomlRelation {
                start: parse_classifier(&text[start.0 + 1..start.1 - 1]),
                end: parse_classifier(&text[end.0 + 1..end.1 - 1]),
                association,
                start_label,
                end_label,
                raw_middle,
            }))
        }
        _ => Err(ParseError::at(
            Format::Nomnoml,
            "relation chains with more than two top-level classifiers are not supported",
            span,
            source,
        )),
    }
}

fn extract_top_level_classifiers(
    text: &str,
    span: Span,
    source: &str,
) -> ParseResult<Vec<(usize, usize)>> {
    let mut result = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut escaped = false;

    for (index, ch) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '[' {
            if depth == 0 {
                start = index;
            }
            depth += 1;
        } else if ch == ']' {
            if depth == 0 {
                return Err(ParseError::at(
                    Format::Nomnoml,
                    "unmatched closing classifier bracket",
                    span,
                    source,
                ));
            }
            depth -= 1;
            if depth == 0 {
                result.push((start, index + ch.len_utf8()));
            }
        }
    }

    if depth != 0 {
        return Err(ParseError::at(
            Format::Nomnoml,
            "unterminated classifier",
            span,
            source,
        ));
    }
    Ok(result)
}

fn parse_classifier(content: &str) -> NomnomlClassifier {
    let compartments = split_top_level(content, '|');
    let first = compartments.first().copied().unwrap_or_default().trim();
    let (classifier_type, attributes, title) = parse_classifier_header(first);
    let mut parsed_compartments = Vec::new();
    parsed_compartments.push(NomnomlCompartment {
        lines: split_compartment_lines(title),
    });
    for compartment in compartments.iter().skip(1) {
        parsed_compartments.push(NomnomlCompartment {
            lines: split_compartment_lines(compartment),
        });
    }
    NomnomlClassifier {
        classifier_type,
        attributes,
        compartments: parsed_compartments,
    }
}

fn parse_classifier_header(header: &str) -> (Option<String>, BTreeMap<String, String>, &str) {
    let trimmed = header.trim();
    if !trimmed.starts_with('<') {
        return (None, BTreeMap::new(), trimmed);
    }
    let Some(end) = trimmed.find('>') else {
        return (None, BTreeMap::new(), trimmed);
    };
    let metadata = &trimmed[1..end];
    let mut words = metadata.split_whitespace();
    let classifier_type = words.next().map(str::to_owned);
    let mut attributes = BTreeMap::new();
    for word in words {
        if let Some((name, value)) = word.split_once('=') {
            attributes.insert(name.to_owned(), value.to_owned());
        } else {
            attributes.insert(word.to_owned(), "true".to_owned());
        }
    }
    (classifier_type, attributes, trimmed[end + 1..].trim())
}

fn split_compartment_lines(content: &str) -> Vec<String> {
    content
        .split(|ch| ch == ';' || ch == '\n')
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

fn split_top_level(text: &str, separator: char) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
        } else if ch == '[' {
            depth += 1;
        } else if ch == ']' {
            depth = depth.saturating_sub(1);
        } else if ch == separator && depth == 0 {
            result.push(&text[start..index]);
            start = index + ch.len_utf8();
        }
    }
    result.push(&text[start..]);
    result
}

fn parse_relation_middle(
    middle: &str,
    span: Span,
    source: &str,
) -> ParseResult<(String, Option<String>, Option<String>)> {
    let (start, end) = find_relation_run(middle).ok_or_else(|| {
        ParseError::at(
            Format::Nomnoml,
            "unable to identify relation operator between classifiers",
            span,
            source,
        )
    })?;
    let association = middle[start..end].trim().to_owned();
    let start_label = non_empty(middle[..start].trim());
    let end_label = non_empty(middle[end..].trim());
    Ok((association, start_label, end_label))
}

fn find_relation_run(text: &str) -> Option<(usize, usize)> {
    let mut run_start = None;
    let mut has_line = false;
    for (index, ch) in text
        .char_indices()
        .chain(std::iter::once((text.len(), ' ')))
    {
        if is_relation_char(ch) {
            run_start.get_or_insert(index);
            has_line |= ch == '-' || ch == '_';
        } else if let Some(start) = run_start.take() {
            if has_line {
                return Some((start, index));
            }
            has_line = false;
        }
    }
    None
}

fn is_relation_char(ch: char) -> bool {
    matches!(
        ch,
        '<' | '>' | '-' | '_' | ':' | '+' | 'o' | 'O' | '(' | ')'
    )
}

fn non_empty(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}
