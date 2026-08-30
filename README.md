# diagram-ast-parser

A Rust library and CLI that parse seven text-to-diagram formats into serializable, format-specific ASTs:

- DBML
- WaveDrom / WaveJSON
- D2
- Structurizr DSL
- LikeC4
- nomnoml
- Pikchr

The project deliberately produces a **syntax AST**, not a fully resolved semantic model. It preserves byte spans and hierarchy, while leaving operations such as imports, cross-file merging, name binding, inheritance, view evaluation, and layout to later compiler stages.

## Why separate ASTs

These languages do not share one adequate `Node + Edge` model:

- DBML models schemas, columns, indexes, and cardinalities.
- WaveDrom models signals over time and register fields.
- D2 is a declarative map/edge language.
- Structurizr and LikeC4 separate architecture models from views.
- nomnoml uses classifier compartments and relation notation.
- Pikchr is a procedural geometric drawing language.

The public root type is therefore:

```rust
pub enum Document {
    Dbml(DbmlDocument),
    WaveDrom(WaveDromDocument),
    D2(D2Document),
    Structurizr(StructurizrDocument),
    LikeC4(LikeC4Document),
    Nomnoml(NomnomlDocument),
    Pikchr(PikchrDocument),
}
```

Every statement-level node is wrapped in `Located<T>` with a UTF-8 byte `Span { start, end }`.

## Build

```bash
cargo build
cargo test
cargo clippy --all-targets --all-features
```

The crate declares Rust 1.85 as its minimum supported version.

## CLI

Parse a file and emit pretty JSON:

```bash
cargo run --bin diagram-parse -- \
  --format dbml \
  examples/schema.dbml
```

Read stdin:

```bash
cat examples/model.c4 |
  cargo run --bin diagram-parse -- --format likec4
```

Compact output and machine-readable diagnostics:

```bash
cargo run --bin diagram-parse -- \
  --format d2 \
  --compact \
  --diagnostic-json \
  examples/architecture.d2
```

`--format auto` uses conservative source heuristics. Explicit format selection is preferable in production because D2, Structurizr, LikeC4, and Pikchr can have overlapping lexical forms.

## Library API

```rust
use diagram_ast_parser::{parse, Format};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        Table users {
          id bigint [pk]
          email varchar [not null, unique]
        }
    "#;

    let document = parse(Format::Dbml, source)?;
    println!("{}", serde_json::to_string_pretty(&document)?);
    Ok(())
}
```

Resource limits can be changed with `parse_with_options`:

```rust
use diagram_ast_parser::{parse_with_options, Format, ParseOptions};

let options = ParseOptions {
    max_input_bytes: 2 * 1024 * 1024,
    max_nesting_depth: 64,
};
let ast = parse_with_options(Format::D2, "a -> b", &options)?;
# Ok::<(), diagram_ast_parser::ParseError>(())
```

## Parser architecture

```text
input
  |
  +-- WaveDrom ---------------- JSON5 parser -------- typed timing/register AST
  |
  +-- DBML/D2/Structurizr/
  |   LikeC4/Pikchr ----------- configurable lexer -- braced statement tree
  |                                                   |
  |                                                   +-- format classifier
  |                                                       and typed AST builder
  |
  +-- nomnoml ---------------- balanced classifier scanner
```

The lexer supports single, double, triple, and DBML backtick strings where appropriate; line/block comments are selected per language. It rejects unbalanced delimiters and enforces a configurable depth limit for the brace-based parsers. `max_nesting_depth` does not currently constrain JSON5 nesting in WaveDrom or nested classifier text in nomnoml.

## Compatibility matrix

| Format | Implemented | Deliberately deferred |
|---|---|---|
| DBML | Project, Table, TablePartial, columns, settings, Indexes, Check, Enum, Ref, TableGroup, Note | modules/import resolution, data-lineage extensions, SQL generation, semantic duplicate/type checks |
| WaveDrom | JSON5 input, nested signal groups, lanes, data, nodes, phase/period, edges, head/foot, register fields, unknown-field preservation | waveform-symbol validation, edge grammar validation, rendering semantics |
| D2 | entries/maps, scalar labels, edge chains, edge attribute maps, regular/spread imports | substitutions, variables, globs, classes, imports loading, shape/property validation, board/layer/scenario semantics |
| Structurizr DSL | workspace, model hierarchy, common elements, relationships, directives, generic blocks/properties, views as blocks | includes/scripts/plugins, expression evaluation, identifier resolution, implied relationships, archetype expansion, view computation |
| LikeC4 | specification kinds, model hierarchy, plain/kinded relationships, tags, views, extend, deployment nodes and `instanceOf` references | multi-file merge, lexical-scope resolution, predicates, inheritance, deployment expansion, view computation, style validation |
| nomnoml | directives/custom styles, classifier type/attributes, compartments, binary relationships and endpoint labels | relation chains containing more than two top-level classifiers, layout/config validation, nested graph semantic analysis |
| Pikchr | objects, labels, directions, assignments, define blocks, print, assert, named places; attributes retained as tokens | expression precedence AST, object/place reference resolution, macro expansion, geometry evaluation |

A successful parse means the source is structurally representable by this project's AST. It does **not** mean the upstream renderer would accept the source semantically.

## Failure behavior

The parser returns one `ParseError` containing:

- format
- message
- byte span when available
- one-based line and column

No parser silently drops an unknown top-level DBML or Pikchr statement. Extensible property-oriented languages such as Structurizr and LikeC4 intentionally retain unfamiliar properties as generic property nodes; semantic validation belongs in a separate pass.

## Suggested next compiler stages

```text
Syntax AST
   |
   +-- symbol collection
   +-- name/reference resolution
   +-- semantic validation
   +-- import/include expansion
   +-- normalized domain IR
   +-- layout IR
   +-- SVG/PNG renderer
```

Do not normalize all seven formats directly into a single graph IR. First normalize them into domain families such as schema, temporal waveform, architecture model/view, classifier graph, and geometric scene; only then lower compatible constructs into a shared layout IR.

## Upstream language references

- DBML syntax: https://dbml.dbdiagram.io/docs/
- WaveDrom tutorial: https://wavedrom.com/tutorial.html
- D2 documentation: https://d2lang.com/
- Structurizr DSL language reference: https://docs.structurizr.com/dsl/language
- LikeC4 DSL: https://likec4.dev/dsl/intro/
- nomnoml syntax: https://www.nomnoml.com/
- Pikchr grammar: https://pikchr.org/home/doc/trunk/doc/grammar.md
