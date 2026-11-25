//! Paragraphs: text in lines.

use std::cell::RefCell;

use nsh_core::prelude::*;
use skia_safe::{
    FontMgr,
    textlayout::{
        FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, PlaceholderAlignment, PlaceholderStyle,
        RectHeightStyle, RectWidthStyle, StrutStyle, TextAlign, TextBaseline, TextDecoration, TextDecorationStyle,
        TextDirection, TextHeightBehavior, TextShadow, TextStyle, TypefaceFontProvider,
    },
};

use crate::{
    args::RectOut,
    canvas::SkCanvas,
    objects::{SkTypeface, font_mgr, font_style},
    paint::SkPaint,
    path::done,
    value::Value,
};

fn int(v: &Value, key: &str) -> Option<i32> {
    v.num_field(key).map(|n| n as i32)
}

fn texts(v: &Value, key: &str) -> Option<Vec<String>> {
    match v.field(key)? {
        Value::List(l) => Some(l.iter().filter_map(Value::text).map(str::to_owned).collect()),
        _ => None,
    }
}

fn flag(v: &Value, key: &str) -> Option<bool> {
    match v.field(key)? {
        Value::Bool(b) => Some(*b),
        _ => None,
    }
}

fn style_of(v: &Value) -> Option<skia_safe::FontStyle> {
    let s = v.field("fontStyle")?;
    Some(font_style(int(s, "weight"), int(s, "width"), int(s, "slant")))
}

/// `style` with what `v` (an `SkTextStyle`) says.
fn text_style(mut style: TextStyle, v: &Value) -> TextStyle {
    if let Some(c) = v.field("color").and_then(Value::color) {
        style.set_color(c.to_color());
    }
    if let Some(c) = v.field("foregroundColor").and_then(Value::color) {
        style.set_foreground_paint(crate::paint::new_paint().set_color4f(c, None));
    }
    if let Some(c) = v.field("backgroundColor").and_then(Value::color) {
        style.set_background_paint(crate::paint::new_paint().set_color4f(c, None));
    }
    if let Some(n) = int(v, "decoration") {
        style.set_decoration_type(TextDecoration::from_bits_truncate(n as u32));
    }
    if let Some(c) = v.field("decorationColor").and_then(Value::color) {
        style.set_decoration_color(c.to_color());
    }
    if let Some(n) = v.num_field("decorationThickness") {
        style.set_decoration_thickness_multiplier(n);
    }
    if let Some(n) = int(v, "decorationStyle") {
        style.set_decoration_style(match n {
            1 => TextDecorationStyle::Double,
            2 => TextDecorationStyle::Dotted,
            3 => TextDecorationStyle::Dashed,
            4 => TextDecorationStyle::Wavy,
            _ => TextDecorationStyle::Solid,
        });
    }
    if let Some(f) = texts(v, "fontFamilies") {
        style.set_font_families(&f);
    }
    if let Some(Value::List(features)) = v.field("fontFeatures") {
        for f in features {
            if let (Some(name), Some(value)) = (f.field("name").and_then(Value::text), int(f, "value")) {
                style.add_font_feature(name, value);
            }
        }
    }
    if let Some(n) = v.num_field("fontSize") {
        style.set_font_size(n);
    }
    if let Some(s) = style_of(v) {
        style.set_font_style(s);
    }
    if let Some(n) = v.num_field("heightMultiplier") {
        style.set_height(n).set_height_override(true);
    }
    if let Some(b) = flag(v, "halfLeading") {
        style.set_half_leading(b);
    }
    if let Some(n) = v.num_field("letterSpacing") {
        style.set_letter_spacing(n);
    }
    if let Some(n) = v.num_field("wordSpacing") {
        style.set_word_spacing(n);
    }
    if let Some(t) = v.field("locale").and_then(Value::text) {
        style.set_locale(t);
    }
    if let Some(n) = int(v, "textBaseline") {
        style.set_text_baseline(baseline(n));
    }
    if let Some(Value::List(shadows)) = v.field("shadows") {
        style.reset_shadows();
        for s in shadows {
            style.add_shadow(TextShadow {
                color: s.field("color").and_then(Value::color).map_or(skia_safe::Color::BLACK, |c| c.to_color()),
                offset: s.field("offset").and_then(Value::point).unwrap_or_default(),
                blur_sigma: f64::from(s.num_field("blurRadius").unwrap_or(0.0)),
            });
        }
    }
    style
}

fn baseline(n: i32) -> TextBaseline {
    if n == 1 { TextBaseline::Ideographic } else { TextBaseline::Alphabetic }
}

fn strut_style(v: &Value) -> StrutStyle {
    let mut s = StrutStyle::new();
    if let Some(b) = flag(v, "strutEnabled") {
        s.set_strut_enabled(b);
    }
    if let Some(f) = texts(v, "fontFamilies") {
        s.set_font_families(&f);
    }
    if let Some(f) = style_of(v) {
        s.set_font_style(f);
    }
    if let Some(n) = v.num_field("fontSize") {
        s.set_font_size(n);
    }
    if let Some(n) = v.num_field("heightMultiplier") {
        s.set_height(n).set_height_override(true);
    }
    if let Some(b) = flag(v, "halfLeading") {
        s.set_half_leading(b);
    }
    if let Some(n) = v.num_field("leading") {
        s.set_leading(n);
    }
    if let Some(b) = flag(v, "forceStrutHeight") {
        s.set_force_strut_height(b);
    }
    s
}

fn paragraph_style(v: &Value) -> ParagraphStyle {
    let mut s = ParagraphStyle::new();
    if flag(v, "disableHinting") == Some(true) {
        s.turn_hinting_off();
    }
    if let Some(t) = v.field("ellipsis").and_then(Value::text) {
        s.set_ellipsis(t);
    }
    if let Some(n) = v.num_field("heightMultiplier") {
        s.set_height(n);
    }
    if let Some(n) = int(v, "maxLines").filter(|n| *n > 0) {
        s.set_max_lines(n as usize);
    }
    if let Some(b) = flag(v, "replaceTabCharacters") {
        s.set_replace_tab_characters(b);
    }
    if let Some(strut) = v.field("strutStyle") {
        s.set_strut_style(strut_style(strut));
    }
    if let Some(n) = int(v, "textAlign") {
        s.set_text_align(match n {
            1 => TextAlign::Right,
            2 => TextAlign::Center,
            3 => TextAlign::Justify,
            4 => TextAlign::Start,
            5 => TextAlign::End,
            _ => TextAlign::Left,
        });
    }
    if let Some(n) = int(v, "textDirection") {
        s.set_text_direction(if n == 0 { TextDirection::RTL } else { TextDirection::LTR });
    }
    if let Some(n) = int(v, "textHeightBehavior") {
        s.set_text_height_behavior(match n {
            1 => TextHeightBehavior::DisableFirstAscent,
            2 => TextHeightBehavior::DisableLastDescent,
            3 => TextHeightBehavior::DisableAll,
            _ => TextHeightBehavior::All,
        });
    }
    if let Some(t) = v.field("textStyle") {
        let style = text_style(s.text_style().clone(), t);
        s.set_text_style(&style);
    }
    s
}

/// The fonts an app registers, by family.
#[class]
pub struct SkTypefaceFontProvider(TypefaceFontProvider);

#[class]
impl SkTypefaceFontProvider {
    /// With no fonts.
    pub fn make() -> SkTypefaceFontProvider {
        Self(TypefaceFontProvider::new())
    }
    /// Adds a font to a family.
    pub fn register_font(&mut self, typeface: Instance<SkTypeface>, family_name: String) {
        typeface.with(|t| self.0.register_typeface(t.0.clone(), family_name.as_str()));
    }
    /// How many families there are.
    pub fn count_families(&self) -> usize {
        FontMgr::from(self.0.clone()).count_families()
    }
}

/// Builds a paragraph.
#[class]
pub struct SkParagraphBuilder {
    builder: ParagraphBuilder,
}

#[class]
impl SkParagraphBuilder {
    /// With a paragraph's style and an app's fonts.
    pub fn new(style: Value, provider: Option<Instance<SkTypefaceFontProvider>>) -> Self {
        let mut fonts = FontCollection::new();
        fonts.set_default_font_manager(font_mgr(), None);
        if let Some(p) = provider {
            fonts.set_asset_font_manager(FontMgr::from(p.with(|p| p.0.clone())));
        }
        Self { builder: ParagraphBuilder::new(&paragraph_style(&style), fonts) }
    }
    /// Adds text.
    pub fn add_text(this: Instance<Self>, text: String) -> Instance<Self> {
        this.with_mut(|b| done(b.builder.add_text(text)));
        this
    }
    /// Text after this is so styled.
    pub fn push_style(
        this: Instance<Self>,
        style: Value,
        foreground: Option<Instance<SkPaint>>,
        background: Option<Instance<SkPaint>>,
    ) -> Instance<Self> {
        this.with_mut(|b| {
            let mut s = text_style(b.builder.peek_style(), &style);
            if let Some(p) = foreground {
                p.with(|p| done(s.set_foreground_paint(&p.0)));
            }
            if let Some(p) = background {
                p.with(|p| done(s.set_background_paint(&p.0)));
            }
            b.builder.push_style(&s);
        });
        this
    }
    /// Text after this is styled as before the last style.
    pub fn pop(this: Instance<Self>) -> Instance<Self> {
        this.with_mut(|b| done(b.builder.pop()));
        this
    }
    /// Adds a space for something else.
    pub fn add_placeholder(
        this: Instance<Self>,
        width: Option<f32>,
        height: Option<f32>,
        alignment: Option<i32>,
        base: Option<i32>,
        offset: Option<f32>,
    ) -> Instance<Self> {
        let alignment = match alignment.unwrap_or(0) {
            1 => PlaceholderAlignment::AboveBaseline,
            2 => PlaceholderAlignment::BelowBaseline,
            3 => PlaceholderAlignment::Top,
            4 => PlaceholderAlignment::Bottom,
            5 => PlaceholderAlignment::Middle,
            _ => PlaceholderAlignment::Baseline,
        };
        let style = PlaceholderStyle::new(
            width.unwrap_or(0.0),
            height.unwrap_or(0.0),
            alignment,
            baseline(base.unwrap_or(0)),
            offset.unwrap_or(0.0),
        );
        this.with_mut(|b| done(b.builder.add_placeholder(&style)));
        this
    }
    /// The paragraph.
    pub fn build(&mut self) -> SkParagraph {
        SkParagraph(RefCell::new(self.builder.build()))
    }
    /// Empties it.
    pub fn reset(&mut self) {
        self.builder.reset();
    }
}

/// Text in lines.
#[class]
pub struct SkParagraph(pub(crate) RefCell<Paragraph>);

impl std::fmt::Debug for SkParagraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SkParagraph")
    }
}

/// A line's measures.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct LineMetricsOut {
    /// Its first character.
    pub start_index: u32,
    /// Its end.
    pub end_index: u32,
    /// Its end without spaces.
    pub end_excluding_whitespaces: u32,
    /// Its end with the newline.
    pub end_including_newline: u32,
    /// Whether the text broke it.
    pub is_hard_break: bool,
    /// Above the baseline.
    pub ascent: f64,
    /// Below it.
    pub descent: f64,
    /// Its height.
    pub height: f64,
    /// Its width.
    pub width: f64,
    /// Its left.
    pub left: f64,
    /// Its baseline.
    pub baseline: f64,
    /// Its number, from 0.
    pub line_number: u32,
}

/// A placeholder's place.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct RectWithDirection {
    /// The rectangle.
    pub rect: RectOut,
    /// 0 right to left, 1 left to right.
    pub direction: i32,
}

#[class]
impl SkParagraph {
    /// Breaks the lines.
    pub fn layout(&self, width: f32) {
        self.0.borrow_mut().layout(width);
    }
    /// Draws it.
    pub fn paint(&self, canvas: Instance<SkCanvas>, x: f32, y: f32) -> Result<(), String> {
        canvas.with(|c| c.draw(|c| self.0.borrow().paint(c, (x, y))))
    }
    /// Its height.
    pub fn get_height(&self) -> f32 {
        self.0.borrow().height()
    }
    /// The width it was laid out in.
    pub fn get_max_width(&self) -> f32 {
        self.0.borrow().max_width()
    }
    /// The least width with no more breaks.
    pub fn get_min_intrinsic_width(&self) -> f32 {
        self.0.borrow().min_intrinsic_width()
    }
    /// The width of one line.
    pub fn get_max_intrinsic_width(&self) -> f32 {
        self.0.borrow().max_intrinsic_width()
    }
    /// The widest line.
    pub fn get_longest_line(&self) -> f32 {
        self.0.borrow().longest_line()
    }
    /// The character at a point.
    pub fn get_glyph_position_at_coordinate(&self, x: f32, y: f32) -> i32 {
        self.0.borrow().get_glyph_position_at_coordinate((x, y)).position
    }
    /// The rectangles of characters.
    pub fn get_rects_for_range(&self, start: usize, end: usize) -> Vec<RectOut> {
        let p = self.0.borrow();
        let boxes = p.get_rects_for_range(start..end, RectHeightStyle::Tight, RectWidthStyle::Tight);
        boxes.iter().map(|b| b.rect.into()).collect()
    }
    /// The rectangles of placeholders.
    pub fn get_rects_for_placeholders(&self) -> Vec<RectWithDirection> {
        let p = self.0.borrow();
        let boxes = p.get_rects_for_placeholders();
        boxes.iter().map(|b| RectWithDirection { rect: b.rect.into(), direction: b.direct as i32 }).collect()
    }
    /// Each line's measures.
    pub fn get_line_metrics(&self) -> Vec<LineMetricsOut> {
        let p = self.0.borrow();
        p.get_line_metrics()
            .iter()
            .map(|m| LineMetricsOut {
                start_index: m.start_index as u32,
                end_index: m.end_index as u32,
                end_excluding_whitespaces: m.end_excluding_whitespaces as u32,
                end_including_newline: m.end_including_newline as u32,
                is_hard_break: m.hard_break,
                ascent: m.ascent,
                descent: m.descent,
                height: m.height,
                width: m.width,
                left: m.left,
                baseline: m.baseline,
                line_number: m.line_number as u32,
            })
            .collect()
    }
}
