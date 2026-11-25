# Architecture

## A frame

1. A component inside a `<Canvas>` runs once, and makes a `SceneNode` (`tsx/scene.ts`): a `SkNode` kept in Rust, with its props converted once (`src/value.rs`). It returns the node where NSH expects an element, so only a `<Canvas>` can show it.
2. A prop whose JSX is an expression is a getter on `props`, as NSH compiles it. `bind` follows it with an effect that sets that one prop. Children that are functions (conditions, `<If>`, `<Each>`) are kept as computeds, so each makes its nodes once.
3. A change climbs to the canvas's root, which asks for one frame per turn (`queueMicrotask`).
4. `SkNode.present` walks the tree in Rust (`src/scene.rs`) into an `SkPicture`, and hands it to the platform's view by the canvas's id.
5. The view draws the last picture: on macOS on the thread `nsh-skia` (`nextDrawable` waits for the display), on Linux on GTK's thread in the `GLArea`'s `render`.

A moving canvas costs one FFI call per changed prop and one `present`. Nothing is compared or walked in JS.

## Drawing

`scene.rs` is a port of react-native-skia's `Visitor`, `Player` and `DrawingContext`, as one recursive walk: each node's paint is its parent's copied and written to; `<Paint>` children are drawn with after it; opacity multiplies down; shaders, filters and effects are stacks composed from the last to the first. Two places are stricter than the reference: a shader with children, and `<Blend>`, take the shaders of their own children only (the reference takes them from the bottom of, or all of, a stack shared with their siblings).

What a node cannot draw (a missing prop, a program's missing uniform) is left out, and its siblings draw. `<Canvas>` warns once per message; `drawAsPicture` throws.

## Engine

`skia-safe` 0.157 (Skia m157), with feature sets rust-skia publishes binaries for (`Cargo.toml`): any other set builds Skia from source. Linux takes the full set because `GLArea` needs GL, and the smaller set with SVG, text and WebP has only Vulkan.

`vendor/skia-bindings` and `vendor/skia-safe` are 0.157.0 with one change. rust-skia's Linux archives hold Skia's own HarfBuzz and libwebp under their public names. NSH's Linux text code calls HarfBuzz itself (`hb_ot_metrics_get_position`), so with Skia's `textlayout` linked those calls went to Skia's copy with a font of the system's HarfBuzz, and the app crashed when it first measured text. `vendor/skia-bindings/build_support/private_symbols.rs` renames them in the archives on Linux. An app can hold one copy of `skia-bindings` only (`links = "skia"`), so it cannot also depend on rust-skia from crates.io.

nsh-webgpu is not used. Its Linux canvas reads every frame back through the CPU, and Skia's own GL draws straight into a `GLArea`. `skia-safe` can share a wgpu 29 device on Metal and Vulkan, so textures could be shared with it later.

## Platforms

- **macOS**: a `CAMetalLayer` tagged sRGB, not opaque. A canvas whose window is covered is not drawn: `nextDrawable` waits up to a second there. It is drawn again when the window shows.
- **Linux**: Skia's GL objects are released when the area is unrealized. `requestAnimationFrame` asked for before the window shows may never come (NSH nightly `2964346`), so `useClock` asks again until it does.
- **Others**: `SkiaImage` draws the last picture in memory at the screen's scale.
- **Web**: the crate builds empty on wasm. skia-bindings supports `wasm32-unknown-emscripten` only.

## Pins

Every `nsh-*` crate is the nightly's commit, and so must the CLI be: two copies of `nsh-core` do not type-check. Moving NSH moves them together (`Cargo.toml`, `examples/*/native/Cargo.toml`, AGENTS.md, CI).
