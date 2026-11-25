# AGENTS.md

Goal: react-native-skia's API for Native Shell (NSH) apps, on `skia-safe`. Reference: [wcandillon/react-native-skia at `6ef585b`](https://github.com/wcandillon/react-native-skia/tree/6ef585b/packages/skia). Read [docs/architecture.md](docs/architecture.md) first; [docs/parity.md](docs/parity.md) says what is left out.

## Toolchain

- NSH is the **nightly** built from `longbridge/nsh` at `2964346`, kept in the checkout at `./.nsh-nightly/nsh/bin/nsh` (ignored by git). Never `nsh update`; export `NSH_HOME="$PWD/.nsh-home"` (the `justfile` does both).
- Every `nsh-*` crate is `rev = "2964346c285f57004f2531fb768cc9840b8d7ff5"`.
- `vendor/` is rust-skia 0.157.0 with one addition (docs/architecture.md "Engine"). Keep it when moving to another version, and run the gallery in a Linux window: headless tests do not link GTK.
- `skia-safe`'s features in `Cargo.toml` are sets rust-skia publishes binaries for. Check its release's assets before changing one.

## Rules

- Gates: `just gates`. A change to `src/scene.rs` also runs `just compare` against a react-native-skia checkout.
- Names, props and defaults are the reference's. Leave one out only with a line in `docs/parity.md`.
- A Rust panic ends a frame: check lengths before calling a `skia-safe` function that asserts them (`Vertices::new_copy`, `TextBlob::from_rsxform`).
- A claim of "works" carries evidence under `docs/evidence/`. Look at every PNG you save.
- Code copied or adapted from elsewhere is listed in [NOTICE.md](NOTICE.md).
