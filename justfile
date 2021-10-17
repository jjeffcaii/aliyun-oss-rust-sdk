set dotenv-load := true

alias t := test

default:
    @just --list

test:
    @cargo test -- --nocapture
lint:
    @cargo clippy
fmt:
    @cargo +nightly fmt
