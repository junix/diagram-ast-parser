# Validation status

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
