use diagram_ast_parser::{
    ast::{dbml::DbmlItem, Document},
    parse, Format, Span,
};
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn assert_rejected(source: &str, message: &str, rejected: &str) {
    let error = parse(Format::Dbml, source).expect_err("invalid DBML must not lose content");
    let start = source.find(rejected).expect("diagnostic substring");
    assert_eq!(error.format, Format::Dbml);
    assert_eq!(error.message, message);
    assert_eq!(error.span, Some(Span::new(start, start + rejected.len())));
    assert_eq!(
        error.line,
        source[..start].chars().filter(|c| *c == '\n').count() + 1
    );
    assert_eq!(
        error.column,
        source[..start].rsplit('\n').next().unwrap().chars().count() + 1
    );
}

#[test]
fn rejects_additional_ref_relationships() {
    for separator in ["\n  ", "; "] {
        let source = format!("Ref {{\n  a.id > b.id{separator}c.id > d.id\n}}\n");
        assert_rejected(
            &source,
            "Ref block must contain exactly one relationship",
            "c.id > d.id",
        );
    }
}

#[test]
fn rejects_nested_ref_relationship_bodies() {
    for nested in ["{}", "{ ignored.id > lost.id }"] {
        let child = format!("a.id > b.id {nested}");
        assert_rejected(
            &format!("Ref {{\n  {child}\n}}\n"),
            "Ref relationship must not have a braced body",
            &child,
        );
    }
}

#[test]
fn rejects_column_bodies_in_tables_and_partials() {
    for declaration in ["Table users", "TablePartial common"] {
        for nested in ["{}", "{ ignored text }"] {
            let column = format!("id int {nested}");
            assert_rejected(
                &format!("{declaration} {{\n  {column}\n}}\n"),
                "column must not have a braced body",
                &column,
            );
        }
    }
}

#[test]
fn rejects_extra_alias_tokens_before_settings() {
    for settings in ["", " []", " [headercolor: '#fff']"] {
        assert_rejected(
            &format!("Table users as u extra{settings} {{ id int }}\n"),
            "unexpected token after table alias",
            "extra",
        );
    }
    assert_rejected(
        "Table users as \"ü\" extra { id int }\n",
        "unexpected token after table alias",
        "extra",
    );
}

#[test]
fn rejects_missing_alias_before_settings() {
    for settings in ["", " []", " [headercolor: '#fff']"] {
        assert_rejected(
            &format!("Table users as{settings} {{ id int }}\n"),
            "`as` requires a table alias",
            "as",
        );
    }
}

#[test]
fn preserves_single_ref_forms_and_settings() {
    for source in [
        "Ref: a.id > b.id [delete: cascade]",
        "Ref link: a.id > b.id [delete: cascade]",
        "Ref { a.id > b.id [delete: cascade] }",
        "Ref link { a.id > b.id [delete: cascade] }",
    ] {
        let Document::Dbml(ast) = parse(Format::Dbml, source).unwrap() else {
            panic!("wrong format")
        };
        let DbmlItem::Ref(reference) = &ast.items[0].node else {
            panic!("expected reference")
        };
        assert_eq!(reference.from.table, "a");
        assert_eq!(reference.to.table, "b");
        assert_eq!(reference.settings[0].name, "delete");
        assert_eq!(
            reference.name.as_deref(),
            source.contains("link").then_some("link")
        );
    }
}

#[test]
fn preserves_alias_settings_columns_and_array_types() {
    use diagram_ast_parser::ast::dbml::DbmlTableItem;
    let source = "Table users as \"user alias\" [headercolor: '#fff'] {\n id int [pk]\n tags text[]\n values int[][] [not null]\n}\n";
    let Document::Dbml(ast) = parse(Format::Dbml, source).unwrap() else {
        panic!("wrong format")
    };
    let DbmlItem::Table(table) = &ast.items[0].node else {
        panic!("expected table")
    };
    assert_eq!(table.alias.as_deref(), Some("user alias"));
    assert_eq!(table.settings[0].name, "headercolor");
    for (item, expected) in table.items.iter().zip(["int", "text[]", "int[][]"]) {
        let DbmlTableItem::Column(column) = &item.node else {
            panic!("expected column")
        };
        assert_eq!(column.data_type, expected);
    }
    assert_eq!(table.items.len(), 3);
}

#[test]
fn cli_rejects_loss_cases_without_partial_stdout() {
    for source in [
        "Table valid { id int }\nRef { a.id > b.id; c.id > d.id }",
        "Table users { id int { ignored text } }",
        "Table users as u extra { id int }",
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_diagram-parse"))
            .args(["--format", "dbml", "--diagnostic-json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(source.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty(), "no partial AST may be emitted");
        let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(diagnostic["format"], "dbml");
        assert!(diagnostic["span"].is_object());
    }
}

#[test]
fn preserves_empty_settings_after_alias() {
    use diagram_ast_parser::ast::dbml::DbmlTableItem;
    for alias in ["u", "\"user alias\""] {
        let source = format!("Table users as {alias} [] {{ id int }}");
        let Document::Dbml(ast) = parse(Format::Dbml, &source).unwrap() else {
            panic!("wrong format")
        };
        let DbmlItem::Table(table) = &ast.items[0].node else {
            panic!("expected table")
        };
        assert_eq!(table.name, "users");
        assert_eq!(table.alias.as_deref(), Some(alias.trim_matches('"')));
        assert!(table.settings.is_empty());
        assert_eq!(table.items.len(), 1);
        let DbmlTableItem::Column(column) = &table.items[0].node else {
            panic!("expected column")
        };
        assert_eq!(column.name, "id");
        assert_eq!(column.data_type, "int");
        assert!(column.settings.is_empty());
    }
}
