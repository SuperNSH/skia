//! Paints.

use nsh_core::prelude::*;
use skia_safe::Paint;

use crate::{
    args::{ColorArg, enums},
    objects::{SkColorFilter, SkImageFilter, SkMaskFilter, SkPathEffect, SkShader, inner},
};

/// How drawing is painted.
#[class]
pub struct SkPaint(pub(crate) Paint);

pub(crate) fn new_paint() -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint
}

#[class]
impl SkPaint {
    /// An opaque black fill, antialiased.
    pub fn new() -> Self {
        Self(new_paint())
    }
    /// Another, the same.
    pub fn copy(&self) -> SkPaint {
        Self(self.0.clone())
    }
    /// Makes it as Skia's default (not antialiased).
    pub fn reset(&mut self) {
        self.0 = Paint::default();
    }
    /// Makes it as `paint`.
    pub fn assign(this: Instance<Self>, paint: Instance<SkPaint>) {
        if !this.ptr_eq(&paint) {
            let other = paint.with(|p| p.0.clone());
            this.with_mut(|p| p.0 = other);
        }
    }
    /// Its color: red, green, blue and alpha.
    pub fn get_color(&self) -> Vec<f32> {
        let c = self.0.color4f();
        vec![c.r, c.g, c.b, c.a]
    }
    /// Its alpha.
    pub fn get_alphaf(&self) -> f32 {
        self.0.alpha_f()
    }
    /// How lines end.
    pub fn get_stroke_cap(&self) -> i32 {
        self.0.stroke_cap() as i32
    }
    /// How lines join.
    pub fn get_stroke_join(&self) -> i32 {
        self.0.stroke_join() as i32
    }
    /// The miter's limit.
    pub fn get_stroke_miter(&self) -> f32 {
        self.0.stroke_miter()
    }
    /// The stroke's width.
    pub fn get_stroke_width(&self) -> f32 {
        self.0.stroke_width()
    }
    /// Sets the color.
    pub fn set_color(&mut self, color: ColorArg) {
        self.0.set_color4f(color.0, None);
    }
    /// Sets the alpha.
    pub fn set_alphaf(&mut self, alpha: f32) {
        self.0.set_alpha_f(alpha);
    }
    /// Smooths edges.
    pub fn set_anti_alias(&mut self, on: bool) {
        self.0.set_anti_alias(on);
    }
    /// Dithers colors.
    pub fn set_dither(&mut self, on: bool) {
        self.0.set_dither(on);
    }
    /// How it blends.
    pub fn set_blend_mode(&mut self, mode: i32) -> Result<(), String> {
        self.0.set_blend_mode(enums::blend(mode)?);
        Ok(())
    }
    /// Fills (0) or strokes (1).
    pub fn set_style(&mut self, style: i32) -> Result<(), String> {
        self.0.set_style(enums::paint_style(style)?);
        Ok(())
    }
    /// How lines end.
    pub fn set_stroke_cap(&mut self, cap: i32) -> Result<(), String> {
        self.0.set_stroke_cap(enums::cap(cap)?);
        Ok(())
    }
    /// How lines join.
    pub fn set_stroke_join(&mut self, join: i32) -> Result<(), String> {
        self.0.set_stroke_join(enums::join(join)?);
        Ok(())
    }
    /// The miter's limit.
    pub fn set_stroke_miter(&mut self, limit: f32) {
        self.0.set_stroke_miter(limit);
    }
    /// The stroke's width.
    pub fn set_stroke_width(&mut self, width: f32) {
        self.0.set_stroke_width(width);
    }
    /// Its shader.
    pub fn set_shader(&mut self, shader: Option<Instance<SkShader>>) {
        self.0.set_shader(inner(&shader, |s| &s.0));
    }
    /// Its color filter.
    pub fn set_color_filter(&mut self, filter: Option<Instance<SkColorFilter>>) {
        self.0.set_color_filter(inner(&filter, |s| &s.0));
    }
    /// Its image filter.
    pub fn set_image_filter(&mut self, filter: Option<Instance<SkImageFilter>>) {
        self.0.set_image_filter(inner(&filter, |s| &s.0));
    }
    /// Its mask filter.
    pub fn set_mask_filter(&mut self, filter: Option<Instance<SkMaskFilter>>) {
        self.0.set_mask_filter(inner(&filter, |s| &s.0));
    }
    /// Its path effect.
    pub fn set_path_effect(&mut self, effect: Option<Instance<SkPathEffect>>) {
        self.0.set_path_effect(inner(&effect, |s| &s.0));
    }
}
