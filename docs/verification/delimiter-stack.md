# Bounded structural delimiter matching

Prepared against `main` at `a8540652364157838d8c6398f08d9abfb4efcb93`, verified on 2026-10-03 UTC.

## Change

The shared statement parser previously counted brackets and parentheses independently, so crossed input such as `node: ([)]` or `node: [(])` could reach AST construction. It now matches each closing token against the latest open delimiter and reports a mismatch at that closing token's original byte span. Existing unmatched-closing and unterminated-expression messages and spans are preserved.

The stack is checked before each push. Its active length plus enclosing statement-block depth cannot exceed `max_nesting_depth`. The boundary is inclusive, completed expressions release their budget, and zero permits only unnested statements. The stack grows on demand rather than allocating from the configured limit, and budget subtraction cannot overflow at `usize::MAX`. This intentionally extends depth enforcement to bracket/parenthesis expressions, which previously had no depth check. Existing input-byte and braced-block limits remain in place.

This applies to DBML, D2, Structurizr, LikeC4, and Pikchr. Lexer treatment of quoted strings, language-specific comments, and original byte spans is unchanged. Braces inside expressions remain raw expression tokens. WaveDrom and nomnoml parsing and their documented depth-limit exclusions are unchanged.

## Regression coverage added

`tests/delimiters.rs` contains 16 Rust tests covering:

- Both crossing orders through all five shared-parser public entry points; offending-token spans and line/column diagnostics, including UTF-8 input
- Unmatched closings, unterminated expressions, valid mixed nesting, scalar text, block hierarchy, statement spans, and multiline/semicolon boundaries
- Single/double/triple/escaped strings, DBML backticks, and configured hash/slash/block comments
- DBML array suffixes and settings, and Pikchr bracket groups with nested parentheses
- Inclusive same-kind/mixed limits, combined block/expression depth, released budgets, existing brace limits, zero, the default 128 levels, and a shallow parse with `usize::MAX`

## Verification status

- The complete remote tree was retrieved without truncation. All 41 baseline file bytes were checked against their remote Git blob hashes. No `AGENTS.md` or repository skill files were present.
- The implementation and assertions were reviewed against the existing lexer, shared tree parser, AST converters, and diagnostic/span contracts. Patch application and whitespace checks were performed locally.
- **Rust compilation, Cargo tests, Clippy, and rustfmt were not run.** The environment has no `cargo`, `rustc`, or `rustfmt`; no toolchain or dependencies were installed. The added tests are unexecuted regression coverage, not passing-test evidence. No simulated parser or substitute implementation was used as runtime verification.
- No browser checks or CI runs were started, and no remote writes were performed during preparation.

Before treating this as runtime-verified, run in an authorized Rust checkout:

```sh
cargo test --test delimiters
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features
cargo test --all-targets --all-features
cargo build --release
cargo fmt --all -- --check
```

The check/lint/test commands match the repository's CI workflow. Formatting may also report pre-existing repository-wide style differences; any formatting follow-up should remain separately scoped.
