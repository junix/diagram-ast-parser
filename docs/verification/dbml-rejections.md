# DBML rejection-only validation

The DBML parser rejects three forms that previously returned a successful AST
while discarding input:

- More than one relationship statement in a `Ref` block
- A braced body attached to a column or to a relationship within a `Ref` block
- Extra tokens after a table alias, before optional table settings

A missing alias before table settings is also rejected rather than treating the
settings opener as an alias. Diagnostics identify the extra statement/token, the
statement with the invalid body, or the `as` token when the alias is missing.

These checks do not expand the supported grammar. In particular, multiple
relationships in one `Ref` block are invalid according to the upstream
[DBML RefValidator](https://github.com/holistics/dbml/blob/c32c3b38b4034e577f521570dad095d190e69473/packages/dbml-parse/src/core/local_modules/ref/validate.ts).
Separate `Ref` declarations remain supported.

## Regression coverage

`tests/dbml_rejections.rs` covers newline and semicolon-separated relationships,
empty and nonempty nested bodies, columns in tables and partials, aliases with
and without settings, missing aliases, Unicode-aware diagnostic coordinates,
valid inline/block references, reference settings, quoted aliases, empty settings
after aliases, and normal
and array column types. CLI tests assert exit code 1, empty stdout (no partial
AST), and JSON diagnostics on stderr. The CLI has no output-file option; this
is not a guarantee about shell redirection preserving an existing file.

## Verification

Against parser baseline `c55350a58209ba4957abfaf73a5eecb1ba0c16f7`, using
Rust 1.99.0 and the original locked dependencies:

- Before the fix: six new rejection tests fail; two valid controls pass
- After the fix: all 37 tests pass (28 existing and nine new)
- `cargo fmt --check` and `cargo build --offline --locked` pass
- `cargo clippy --offline --locked --all-targets` succeeds with the six existing
  library warnings; it is not a warning-free or `-D warnings` result
- 15 valid fixtures in pretty and compact modes produce byte-identical stdout
  to the baseline (30 comparisons), including all seven repository examples
- The three original loss reproductions each exit 1 with empty stdout in
  default, compact, and JSON-diagnostic modes (nine CLI checks)

No dependencies, AST types, public APIs, parser options, or CLI code changed.

Scope remains limited to these rejection sites; unrelated declaration tails are
not comprehensively validated. Empty `[]` settings after aliases retain their
baseline no-op behavior, without changing column array suffix handling.
