#!/usr/bin/env bash
set -e -o pipefail

MSRV=1.56.1
FUZZ_DURATION=60
FUZZ_PAUSE=2

if [[ ${1:0:1} == - || $1 == help ]] || (( $# > 1 )); then
    echo "Usage: $0 ([setup|[check|MSRV]|test|ignored|clippy|fuzz|test|help])" >&2
    echo "If no argument is provided, all parts except ignored and help are run," >&2
    echo "but setup is only done if auto-detection fails." >&2
    exit 1
fi

# Minimum supported Rust version
if [[ $1 == setup ]] || ! rustup show | grep --silent "$MSRV"; then
    rustup install "$MSRV" --no-self-update
fi
if [[ -z $1 || $1 == msrv || $1 == MSRV || $1 == check ]]; then
    cargo "+$MSRV" check --tests --examples --no-default-features
    cargo "+$MSRV" check --tests --examples
    cargo "+$MSRV" build --tests --examples --all-features
fi

# stable
if [[ -z $1 || $1 == test ]]; then
    cargo test --all-features -- --quiet
elif [[ $1 == ignored ]]; then
    cargo test --all-features -- --quiet --ignored
fi

if [[ $1 == setup ]] || ! rustup show | grep --silent nightly; then
    rustup install nightly --no-self-update
fi
# clippy
if [[ $1 == setup ]] || ! cargo +nightly help clippy >/dev/null 2>/dev/null; then
    rustup component add clippy --toolchain nightly
fi
if [[ -z $1 || $1 == clippy ]]; then
    cargo +nightly clippy --all-features --tests --benches --examples
fi

# fuzzing tests
if [[ $1 == setup ]] || ! command -V cargo-fuzz >/dev/null 2>/dev/null; then
    cargo +nightly install cargo-fuzz
fi
if [[ -z $1 || $1 == fuzz ]]; then
    cargo +nightly fuzz build
    for fuzztest in $(cargo +nightly fuzz list); do
        sleep "$FUZZ_PAUSE"
        echo "Fuzzing $fuzztest"
        timeout "$FUZZ_DURATION" \
            cargo +nightly fuzz run "$fuzztest" \
            || true
        echo
    done
fi

# benchmarks
if [[ -z $1 || $1 == bench ]]; then
    cargo +nightly check --benches --no-default-features
    cargo +nightly check --benches
    # need nocapture to not hide error if setup fails
    exec cargo +nightly bench --all-features -- --nocapture
fi