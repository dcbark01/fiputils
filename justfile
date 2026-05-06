#!/usr/bin/env just --justfile

set dotenv-load := true


default:
    @just --list

build:
    cargo build

release:
    cargo build --release

# Bump Cargo.toml version, commit, and tag (does not push).
# Usage: just release-tag 0.2.0
release-tag VERSION:
    @[ -z "$(git status --porcelain)" ] || { echo "error: working tree is not clean"; exit 1; }
    sed -i.bak '/^\[package\]/,/^\[/ s/^version = ".*"/version = "{{VERSION}}"/' Cargo.toml
    rm Cargo.toml.bak
    cargo check --quiet
    git add Cargo.toml Cargo.lock
    git commit -m "chore: release v{{VERSION}}"
    git tag v{{VERSION}}
    @echo "Tagged v{{VERSION}}. Push with: git push && git push --tags"

run *ARGS:
    cargo run -- {{ARGS}}

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy -- -D warnings

lint-fix:
    cargo clippy --fix --allow-dirty --allow-staged -- -D warnings

test:
    cargo test

check: fmt-check lint test

clean:
    cargo clean
