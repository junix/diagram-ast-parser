use diagram_ast_parser::{ast::Document, parse, Format};

#[test]
fn parses_dbml_core_schema() {
    let source = include_str!("../examples/schema.dbml");
    let document = parse(Format::Dbml, source).expect("DBML should parse");
    let Document::Dbml(ast) = document else {
        panic!("wrong AST variant");
    };
    assert_eq!(ast.items.len(), 5);
}

#[test]
fn parses_wavedrom_timing_json5() {
    let source = include_str!("../examples/timing.json5");
    let document = parse(Format::WaveDrom, source).expect("WaveDrom should parse");
    let Document::WaveDrom(ast) = document else {
        panic!("wrong AST variant");
    };
    let timing = ast.timing.expect("timing diagram");
    assert_eq!(timing.signal.len(), 2);
    assert_eq!(timing.edges, vec!["a~>b transfer"]);
}

#[test]
fn parses_d2_entries_and_edges() {
    let source = include_str!("../examples/architecture.d2");
    let document = parse(Format::D2, source).expect("D2 should parse");
    let Document::D2(ast) = document else {
        panic!("wrong AST variant");
    };
    assert!(ast.statements.len() >= 5);
}

#[test]
fn parses_structurizr_model_and_views() {
    let source = include_str!("../examples/workspace.dsl");
    let document = parse(Format::Structurizr, source).expect("Structurizr should parse");
    let Document::Structurizr(ast) = document else {
        panic!("wrong AST variant");
    };
    assert_eq!(ast.statements.len(), 1);
}

#[test]
fn parses_likec4_spec_model_and_view() {
    let source = include_str!("../examples/model.c4");
    let document = parse(Format::LikeC4, source).expect("LikeC4 should parse");
    let Document::LikeC4(ast) = document else {
        panic!("wrong AST variant");
    };
    assert_eq!(ast.statements.len(), 3);
}

#[test]
fn parses_nomnoml_directives_and_relations() {
    let source = include_str!("../examples/classes.nomnoml");
    let document = parse(Format::Nomnoml, source).expect("nomnoml should parse");
    let Document::Nomnoml(ast) = document else {
        panic!("wrong AST variant");
    };
    assert_eq!(ast.directives.len(), 3);
    assert_eq!(ast.statements.len(), 2);
}

#[test]
fn parses_pikchr_objects_and_assignment() {
    let source = include_str!("../examples/flow.pikchr");
    let document = parse(Format::Pikchr, source).expect("Pikchr should parse");
    let Document::Pikchr(ast) = document else {
        panic!("wrong AST variant");
    };
    assert_eq!(ast.statements.len(), 7);
}

#[test]
fn auto_detects_wavejson() {
    let source = "{ signal: [{ name: 'clk', wave: 'p...' }] }";
    let document = parse(Format::Auto, source).expect("auto detection should parse");
    assert!(matches!(document, Document::WaveDrom(_)));
}

#[test]
fn rejects_unclosed_blocks() {
    let error = parse(Format::D2, "a: {\n  b: c\n").expect_err("must reject malformed D2");
    assert!(error.message.contains("unterminated block"));
}

#[test]
fn dbml_keeps_array_suffix_in_column_type() {
    use diagram_ast_parser::ast::dbml::{DbmlItem, DbmlTableItem};

    let source = "Table events {\n  tags text[] [not null]\n}\n";
    let document = parse(Format::Dbml, source).expect("DBML array type should parse");
    let Document::Dbml(ast) = document else {
        panic!("wrong AST variant");
    };
    let DbmlItem::Table(table) = &ast.items[0].node else {
        panic!("expected table");
    };
    let DbmlTableItem::Column(column) = &table.items[0].node else {
        panic!("expected column");
    };
    assert_eq!(column.data_type, "text[]");
    assert_eq!(column.settings.len(), 1);
    assert_eq!(column.settings[0].name, "not null");
}

#[test]
fn enforces_input_size_limit() {
    use diagram_ast_parser::{parse_with_options, ParseOptions};

    let options = ParseOptions {
        max_input_bytes: 4,
        max_nesting_depth: 8,
    };
    let error = parse_with_options(Format::D2, "alpha: beta", &options)
        .expect_err("oversized input must be rejected");
    assert!(error.message.contains("exceeding the configured limit"));
}

#[test]
fn rejects_invalid_wavedrom_register_width() {
    let source = "{ reg: [{ bits: -1, name: 'reserved' }] }";
    let error =
        parse(Format::WaveDrom, source).expect_err("negative register width must be rejected");
    assert!(error.message.contains("unsigned integer"));
}

#[test]
fn wavedrom_enforces_configured_nesting_limit() {
    use diagram_ast_parser::{parse_with_options, ParseOptions};

    let options = ParseOptions {
        max_input_bytes: 1024,
        max_nesting_depth: 3,
    };

    // Root object (1) + signal array (2) + one group array (3) sits exactly at
    // the limit.
    let at_limit = "{ signal: [['a']] }";
    parse_with_options(Format::WaveDrom, at_limit, &options)
        .expect("nesting equal to the limit must parse");

    // One more group array reaches depth 4.
    let beyond_limit = "{ signal: [['a', ['b']]] }";
    let error = parse_with_options(Format::WaveDrom, beyond_limit, &options)
        .expect_err("nesting beyond the limit must be rejected");
    assert!(error
        .message
        .contains("nesting depth exceeds configured limit of 3"));
}

#[test]
fn wavedrom_nesting_limit_ignores_brackets_in_strings_and_comments() {
    use diagram_ast_parser::{parse_with_options, ParseOptions};

    let source = "{ signal: [\n  // brackets [[[[ inside a comment do not count\n  { name: 'clk', wave: 'p...', data: 'x[[0]], y[1]' }\n] }";
    let options = ParseOptions {
        max_input_bytes: 1024,
        max_nesting_depth: 3,
    };
    parse_with_options(Format::WaveDrom, source, &options)
        .expect("brackets inside strings and comments must not count");
}

#[test]
fn wavedrom_default_nesting_limit_bounds_group_recursion() {
    fn nested_groups(count: usize) -> String {
        let mut source = String::from("{ signal: [");
        for _ in 0..count {
            source.push_str("['g', ");
        }
        for _ in 0..count {
            source.push(']');
        }
        source.push_str("]}");
        source
    }

    // 126 nested groups reach depth 128, the default budget.
    parse(Format::WaveDrom, &nested_groups(126))
        .expect("nesting equal to the default limit must parse");

    let error = parse(Format::WaveDrom, &nested_groups(127))
        .expect_err("nesting beyond the default limit must be rejected");
    assert!(error
        .message
        .contains("nesting depth exceeds configured limit of 128"));
}

#[test]
fn wavedrom_nesting_limit_preserves_json5_errors() {
    let source = "{ signal: [{ name: 'clk', wave: 'p..' }, }";
    let error =
        parse(Format::WaveDrom, source).expect_err("malformed JSON5 must still be rejected");
    assert!(error.message.contains("invalid WaveJSON/JSON5"));
}
