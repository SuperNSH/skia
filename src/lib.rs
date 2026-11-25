//! Skia for Native Shell apps: the react-native-skia API on skia-safe.
//!
//! `tsx/` is the API apps import (`nsh-skia`); this crate is the engine
//! under it: Skia's objects as classes, the declarative scene, and the
//! canvas view of each platform.

#[cfg(not(target_arch = "wasm32"))]
mod args;
#[cfg(not(target_arch = "wasm32"))]
mod canvas;
#[cfg(not(target_arch = "wasm32"))]
mod objects;
#[cfg(not(target_arch = "wasm32"))]
mod paint;
#[cfg(not(target_arch = "wasm32"))]
mod paragraph;
#[cfg(not(target_arch = "wasm32"))]
mod path;
#[cfg(not(target_arch = "wasm32"))]
mod scene;
#[cfg(not(target_arch = "wasm32"))]
mod value;
#[cfg(not(target_arch = "wasm32"))]
mod view;

use nsh_core::prelude::*;

#[cfg(target_os = "linux")]
pub use view::linux::*;
#[cfg(target_os = "macos")]
pub use view::macos::*;

#[cfg(not(target_arch = "wasm32"))]
#[native_module("nsh-skia/native")]
fn native(registry: &mut Registry) {
    registry
        .class::<canvas::SkCanvas>()
        .class::<canvas::SkPictureRecorder>()
        .class::<canvas::SkSurface>()
        .class::<objects::SkColorFilter>()
        .class::<objects::SkData>()
        .class::<objects::SkFont>()
        .class::<objects::SkFontMgr>()
        .class::<objects::SkImage>()
        .class::<objects::SkImageFilter>()
        .class::<objects::SkMaskFilter>()
        .class::<objects::SkMatrix>()
        .class::<objects::SkPathEffect>()
        .class::<objects::SkPicture>()
        .class::<objects::SkRuntimeEffect>()
        .class::<objects::SkRuntimeShaderBuilder>()
        .class::<objects::SkShader>()
        .class::<objects::SkSvg>()
        .class::<objects::SkTextBlob>()
        .class::<objects::SkTypeface>()
        .class::<objects::SkVertices>()
        .class::<paint::SkPaint>()
        .class::<paragraph::SkParagraph>()
        .class::<paragraph::SkParagraphBuilder>()
        .class::<paragraph::SkTypefaceFontProvider>()
        .class::<path::SkContourMeasure>()
        .class::<path::SkContourMeasureIter>()
        .class::<path::SkPath>()
        .class::<path::SkPathBuilder>()
        .class::<scene::SkNode>()
        .module::<view::SkiaHost>()
        .register::<view::SkiaImage>();
}
