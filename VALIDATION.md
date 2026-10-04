# Validation status

## Native follow-up: 2026-10-04 UTC

The native verification gap described in the historical preparation report is
closed for commit `97a64e30c0647fd987d179bd19b52e93a1b72d5a` on Linux x86_64,
using `rustc 1.99.0 (b940084d7 2026-09-28)` and
`cargo 1.99.0 (5f94df478 2026-08-27)`, with two build jobs. All 43 tracked
files were materialized and checked against their remote Git blob hashes.
Dependencies were downloaded from the authorized official registry and the
checked-in lockfile was preserved.

These commands completed successfully:

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo clippy --locked --all-targets --all-features
cargo test --locked --all-targets --all-features
cargo build --locked --release
cargo test --locked --all-features --doc
```

All 28 integration tests passed: the 16 new delimiter regressions and 12 existing
parser tests. The doctest command registered zero tests; it does not provide
additional example coverage. The release CLI separately parsed all seven files
in `examples/` into valid JSON. For each of DBML, D2, Structurizr, LikeC4 and
Pikchr, `--diagnostic-json` rejected `node: ([)]` with exit 1, a mismatch
diagnostic at byte span `8..9`, and line 1, column 9.

Clippy exited 0 under the repository's existing warning policy. It was **not a
warning-free strict Clippy pass**: `large_enum_variant`, `collapsible_if`,
`needless_bool_assign`, `obfuscated_if_else`, `manual_pattern_char_comparison`
and `while_let_loop` warnings remain. No production or test repair was needed.

This was a local native run, not a GitHub Actions or browser result. The current
workflow selects stable Rust; the declared Rust 1.85 minimum and other platforms
were not verified. No claim is made that successful AST construction proves
acceptance by the upstream diagram renderers.

## Historical artifact-generation report

The following original report records the environment and checks at generation
time. Its unexecuted-native-check statements describe that earlier stage; the
native follow-up above supersedes those limits for the verified commit. The
original Rust 1.85 workflow description is historical; the current workflow
selects stable Rust.

Validation performed in the artifact-generation environment:

- Cargo, Rust toolchain, and workflow TOML/YAML files were structurally checked.
- The WaveDrom fixture was parsed with an independent JSON5 implementation.
- All Rust files passed a string/comment-aware delimiter-balance scan.
- The shared lexer/braced-statement assumptions were independently simulated against the DBML, D2, Structurizr DSL, LikeC4, and Pikchr fixtures.
- Regression checks cover DBML array type suffixes versus trailing settings.

The generation environment did not contain `rustc` or `cargo`, and outbound package installation was unavailable. Consequently, `cargo check`, `cargo clippy`, and `cargo test` were **not executed here**. The repository includes GitHub Actions configuration that runs all three commands with Rust 1.85.0.

Run locally before relying on the crate:

```bash
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features
cargo test --all-targets --all-features
```
