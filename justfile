#!/usr/bin/env just --justfile

set dotenv-load := true


default:
    @just --list

build:
    cargo build

release:
    cargo build --release

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
