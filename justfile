set shell := ["bash", "-euo", "pipefail", "-c"]

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
