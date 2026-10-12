set shell := ["bash", "-euo", "pipefail", "-c"]

# ADR-1168 build stamp: git short sha, ".dirty" when the worktree is dirty.
stamp := `git rev-parse --short HEAD` + `(git diff --quiet && git diff --cached --quiet) >/dev/null 2>&1 || printf .dirty`
export PM_BUILD_SHA := "g" + stamp

os_name := if os() == "macos" { "macos" } else { "linux" }
arch_name := if arch() == "aarch64" { "arm64" } else { "x86" }
default_install_bin := home_directory() / "sync" / (os_name + "-" + arch_name + "-bin")
install_bin := env("SYNC_BIN_DIR", default_install_bin)
target_dir := env("CARGO_TARGET_DIR", justfile_directory() / "target")

default: build

build:
    cargo build --release

test:
    cargo test --all-targets

install: build
    mkdir -p "{{ install_bin }}"
    @set -eu; dest="{{ install_bin }}/diagram-parse"; mkdir -p "$(dirname "$dest")"; tmp="$(mktemp "{{ install_bin }}/.diagram-parse.XXXXXX")"; trap 'rm -f "$tmp"' EXIT; cp "{{ target_dir }}/release/diagram-parse" "$tmp"; chmod 755 "$tmp"; if [ "$(uname -s)" = "Darwin" ]; then xattr -c "$tmp" 2>/dev/null || true; codesign --force --sign - "$tmp"; fi; mv -f "$tmp" "$dest"
    echo "Installed {{ install_bin }}/diagram-parse"

# Parse every example fixture to out/*.ast.json, render the optional D2
# preview when the upstream engine is installed, regenerate ROOT gallery.html.
rust-gallery: build
    #!/usr/bin/env bash
    set -euo pipefail
    bin="{{ target_dir }}/release/diagram-parse"
    mkdir -p out
    "$bin" --format dbml        examples/schema.dbml      > out/schema.ast.json
    "$bin" --format wavedrom    examples/timing.json5     > out/timing.ast.json
    "$bin" --format d2          examples/architecture.d2  > out/architecture.ast.json
    "$bin" --format structurizr examples/workspace.dsl    > out/workspace.ast.json
    "$bin" --format likec4      examples/model.c4         > out/model.ast.json
    "$bin" --format nomnoml     examples/classes.nomnoml  > out/classes.ast.json
    "$bin" --format pikchr      examples/flow.pikchr      > out/flow.ast.json
    if command -v d2 >/dev/null; then d2 examples/architecture.d2 out/architecture.svg; fi
    python3 tools/gallery.py

rust-gallery-check:
    python3 tools/gallery.py --check
