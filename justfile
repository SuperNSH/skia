# nsh is the nightly kept in this repository, never the one on PATH.

set shell := ["bash", "-euo", "pipefail", "-c"]

nsh := justfile_directory() / ".nsh-nightly/nsh/bin/nsh"
export NSH_HOME := justfile_directory() / ".nsh-home"

# List the recipes.
default:
    @just --list

# Format and Clippy, warnings as errors.
check:
    cargo fmt -- --check
    cargo clippy --all-targets -- -D warnings

# nsh check, build (a warning fails) and test, in an example.
test-tsx app="headless":
    mkdir -p "$NSH_HOME"
    cd examples/{{ app }} && {{ nsh }} check
    cd examples/{{ app }} && {{ nsh }} build 2>&1 | tee "$NSH_HOME/build.log"
    ! grep -iE '^ *warn' "$NSH_HOME/build.log"
    cd examples/{{ app }} && {{ nsh }} test

# Every gate.
gates: check (test-tsx "headless") (test-tsx "parity")

# The gallery in a window.
gallery:
    cd examples/gallery && {{ nsh }} run --release --window-size 980x620 App.tsx

# Compare with react-native-skia: `just compare <checkout>` (npm install in tools/parity first).
compare reference out="target/parity":
    mkdir -p {{ out }}
    cd examples/parity && {{ nsh }} test > {{ justfile_directory() }}/{{ out }}/nsh.log
    cd tools/parity && node compare.mjs {{ reference }}/packages/skia/src {{ justfile_directory() }}/{{ out }}/nsh.log {{ justfile_directory() }}/{{ out }}
