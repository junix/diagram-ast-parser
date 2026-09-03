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
    cp "{{ target_dir }}/release/diagram-parse" "{{ install_bin }}/diagram-parse"
    chmod +x "{{ install_bin }}/diagram-parse"
    echo "Installed {{ install_bin }}/diagram-parse"
