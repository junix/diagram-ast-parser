use diagram_ast_parser::{
    ast::{
        d2::{D2Document, D2Statement, D2Value},
        dbml::{DbmlItem, DbmlTableItem},
        Document,
    },
    parse, parse_with_options, Format, ParseOptions, Span,
};

fn options(max_nesting_depth: usize) -> ParseOptions {
    ParseOptions {
        max_nesting_depth,
        ..ParseOptions::default()
    }
}

fn parse_d2(source: &str, max_nesting_depth: usize) -> D2Document {
    let document = parse_with_options(Format::D2, source, &options(max_nesting_depth))
        .expect("structurally valid D2 should parse");
    let Document::D2(ast) = document else {
        panic!("wrong AST variant");
    };
    ast
}

#[test]
fn rejects_crossed_delimiters_in_every_braced_parser() {
    for format in [
        Format::Dbml,
        Format::D2,
        Format::Structurizr,
        Format::LikeC4,
        Format::Pikchr,
    ] {
        for (expression, found, expected) in [("([)]", ')', ']'), ("[(])", ']', ')')] {
            let source = format!("node: {expression}");
            let offset = source.find(found).unwrap();
            let error = parse(format, &source).expect_err("crossed delimiters must fail");
            assert_eq!(error.format, format);
            assert_eq!(
                error.message,
                format!("mismatched closing delimiter `{found}`: expected `{expected}`")
            );
            assert_eq!(error.span, Some(Span::new(offset, offset + 1)));
            assert_eq!((error.line, error.column), (1, offset + 1));
        }
    }
}

#[test]
fn mismatched_delimiter_diagnostic_uses_utf8_byte_span_and_character_column() {
    let source = "node: [\n  🦀(])";
    let offset = source.find(']').unwrap();
    let error = parse(Format::D2, source).expect_err("crossed delimiters must fail");
    assert_eq!(error.span, Some(Span::new(offset, offset + 1)));
    assert_eq!((error.line, error.column), (2, 5));
    assert_eq!(
        error.message,
        "mismatched closing delimiter `]`: expected `)`"
    );
}

#[test]
fn preserves_unmatched_closing_diagnostics() {
    for (source, message) in [
        ("node: ]", "unmatched closing bracket"),
        ("node: []]", "unmatched closing bracket"),
        ("node: )", "unmatched closing parenthesis"),
        ("node: ())", "unmatched closing parenthesis"),
    ] {
        let error = parse(Format::D2, source).expect_err("unmatched closing delimiter");
        assert_eq!(error.message, message);
        assert_eq!(error.span, Some(Span::new(source.len() - 1, source.len())));
    }
}

#[test]
fn preserves_unterminated_expression_diagnostics_and_spans() {
    for (source, message) in [
        ("node: [value", "unterminated bracket expression"),
        ("node: (value", "unterminated parenthesized expression"),
        ("node: ([value", "unterminated bracket expression"),
        ("node: [(value", "unterminated bracket expression"),
    ] {
        let error = parse(Format::D2, source).expect_err("unterminated expression");
        assert_eq!(error.message, message);
        assert_eq!(error.span, Some(Span::new(0, source.len())));
    }
}

#[test]
fn preserves_well_nested_expression_text_and_spans() {
    for expression in [
        "([])",
        "[()]",
        "([(value)])",
        "[(value)]",
        "()[]",
        "({value})",
    ] {
        let source = format!("node: {expression}");
        let ast = parse_d2(&source, 3);
        assert_eq!(ast.statements.len(), 1);
        assert_eq!(ast.statements[0].span, Span::new(0, source.len()));
        let D2Statement::Entry(entry) = &ast.statements[0].node else {
            panic!("expected entry");
        };
        assert_eq!(entry.value, Some(D2Value::Scalar(expression.to_owned())));
    }
}

#[test]
fn preserves_nested_blocks_and_expression_statement_boundaries() {
    let source = "outer: {\n  node: ([\n    value;\n  ])\n  sibling: ok\n}\nafter: ok\n";
    let ast = parse_d2(source, 3);
    assert_eq!(ast.statements.len(), 2);
    assert_eq!(
        ast.statements[0].span,
        Span::new(0, source.find('}').unwrap() + 1)
    );
    let D2Statement::Entry(entry) = &ast.statements[0].node else {
        panic!("expected entry");
    };
    let Some(D2Value::Map { statements, .. }) = &entry.value else {
        panic!("expected map");
    };
    assert_eq!(statements.len(), 2);
    assert_eq!(
        statements[0].span,
        Span::new(
            source.find("node:").unwrap(),
            source.find("])").unwrap() + 2
        )
    );
    assert_eq!(
        &source[statements[1].span.start..statements[1].span.end],
        "sibling: ok"
    );
    assert_eq!(
        &source[ast.statements[1].span.start..ast.statements[1].span.end],
        "after: ok"
    );
}

#[test]
fn quoted_delimiters_do_not_consume_depth_or_cross_match() {
    for source in [
        "node: '([)]'",
        "node: \"[(])\"",
        "node: '''([)]'''",
        "node: \"\"\"[(])\"\"\"",
        "node: \"escaped \\\" ([)]\"",
    ] {
        let ast = parse_d2(source, 0);
        assert_eq!(ast.statements[0].span, Span::new(0, source.len()));
    }
    parse_d2("node: [\")\" ']' \"\"\"(\"\"\"]", 1);
}

#[test]
fn comments_do_not_consume_depth_or_cross_match() {
    for (format, source, limit) in [
        (Format::D2, "node: value # ([)]\n", 0),
        (Format::D2, "node: [\n# [(])\nvalue]", 1),
        (Format::Structurizr, "title \"value\" // ([)]\n", 0),
        (Format::LikeC4, "title /* [(]) */ \"value\"", 0),
        (Format::Pikchr, "box \"value\" /* ([)] */", 0),
        (Format::Dbml, "Table t { // ([)]\n id int /* [(]) */\n}", 1),
    ] {
        parse_with_options(format, source, &options(limit))
            .expect("comment delimiters must be ignored");
    }
}

#[test]
fn preserves_dbml_array_types_settings_and_backtick_expressions() {
    let source = "Table t {\n values decimal(10, 2)[] [default: `([)]`]\n}";
    let document = parse_with_options(Format::Dbml, source, &options(2))
        .expect("DBML types, settings, and backticks should parse");
    let Document::Dbml(ast) = document else {
        panic!("wrong AST variant");
    };
    let DbmlItem::Table(table) = &ast.items[0].node else {
        panic!("expected table");
    };
    let DbmlTableItem::Column(column) = &table.items[0].node else {
        panic!("expected column");
    };
    assert_eq!(column.data_type, "decimal(10,2)[]");
    assert_eq!(column.settings.len(), 1);
    assert_eq!(column.settings[0].name, "default");
}

#[test]
fn preserves_pikchr_bracket_groups_with_parentheses_and_separators() {
    let source = "Group: [\n box \"([)]\"; move (1, 2)\n]\n";
    let document = parse_with_options(Format::Pikchr, source, &options(2))
        .expect("Pikchr bracket group should parse");
    let Document::Pikchr(ast) = document else {
        panic!("wrong AST variant");
    };
    assert_eq!(ast.statements.len(), 1);
    assert_eq!(ast.statements[0].span, Span::new(0, source.len() - 1));
}

#[test]
fn expression_depth_limit_is_inclusive_for_each_kind_and_mixed_nesting() {
    for (expression, limit, first_excess) in [
        ("((value))", 2, 1),
        ("[[value]]", 2, 1),
        ("([value])", 2, 1),
        ("[(value)]", 2, 1),
    ] {
        let source = format!("node: {expression}");
        parse_d2(&source, limit);
        let error = parse_with_options(Format::D2, &source, &options(limit - 1))
            .expect_err("expression nesting must be bounded");
        assert_eq!(error.message, "nesting depth exceeds configured limit of 1");
        let offset = "node: ".len() + first_excess;
        assert_eq!(error.span, Some(Span::new(offset, offset + 1)));
    }
}

#[test]
fn expression_depth_includes_enclosing_blocks() {
    let source = "outer: {\n node: ([value])\n}";
    parse_d2(source, 3);
    let error = parse_with_options(Format::D2, source, &options(2))
        .expect_err("brace and expression nesting share the limit");
    let offset = source.find('[').unwrap();
    assert_eq!(error.message, "nesting depth exceeds configured limit of 2");
    assert_eq!(error.span, Some(Span::new(offset, offset + 1)));
}

#[test]
fn completed_expressions_release_the_depth_budget() {
    parse_d2("node: []()[]; next: ()[]()", 1);
    parse_d2("outer ([value]): { child: ok }", 2);
    parse_d2("outer: { child: [] }; sibling: [()]", 2);
}

#[test]
fn preserves_brace_depth_limit_and_zero_depth_behavior() {
    let nested_blocks = "outer: { inner: { child: ok } }";
    parse_d2(nested_blocks, 2);
    let error = parse_with_options(Format::D2, nested_blocks, &options(1))
        .expect_err("braced blocks remain bounded");
    assert_eq!(error.message, "nesting depth exceeds configured limit of 1");
    parse_d2("node: value", 0);
    for source in ["node: []", "node: ()", "node: {}"] {
        let error = parse_with_options(Format::D2, source, &options(0))
            .expect_err("zero depth forbids structural nesting");
        assert_eq!(error.message, "nesting depth exceeds configured limit of 0");
    }
}

#[test]
fn default_depth_limit_bounds_the_delimiter_stack() {
    let limit = ParseOptions::default().max_nesting_depth;
    let source = format!("node: {}value{}", "(".repeat(limit), ")".repeat(limit));
    parse(Format::D2, &source).expect("the default nesting limit is inclusive");
    let source = format!("node: ({}value{})", "(".repeat(limit), ")".repeat(limit));
    let error = parse(Format::D2, &source).expect_err("one level over the default limit");
    let offset = "node: ".len() + limit;
    assert_eq!(error.span, Some(Span::new(offset, offset + 1)));
    assert_eq!(
        error.message,
        format!("nesting depth exceeds configured limit of {limit}")
    );
}

#[test]
fn large_configured_limit_does_not_allocate_a_full_sized_stack() {
    parse_d2("outer: { node: ([value]) }", usize::MAX);
}
