//! Canvases: what a surface and a picture recorder draw with.

use std::{cell::RefCell, rc::Rc};

use nsh_core::prelude::*;
use skia_safe::{
    AlphaType, Canvas, ColorType, CubicResampler, ImageInfo, Paint, Picture, PictureRecorder, Rect, Surface,
    canvas::{SaveLayerFlags, SaveLayerRec, SrcRectConstraint},
    surfaces,
};

use crate::{
    args::{ColorArg, Colors, Floats, MatrixArg, Points, RRectArg, RectArg, Rects, Transforms, enums},
    objects::{SkFont, SkImage, SkImageFilter, SkMatrix, SkPicture, SkSvg, SkTextBlob, SkVertices, inner, read_pixels},
    paint::SkPaint,
    path::{SkPath, bad},
    value::{Value, sampling_of},
};

enum Target {
    Surface(Surface),
    Recorder(PictureRecorder),
    /// A recorder that gave its picture.
    Done,
}

type Shared = Rc<RefCell<Target>>;

/// What draws.
#[class]
pub struct SkCanvas(Shared);

impl SkCanvas {
    pub(crate) fn draw<R>(&self, f: impl FnOnce(&Canvas) -> R) -> Result<R, String> {
        match &mut *self.0.borrow_mut() {
            Target::Surface(s) => Ok(f(s.canvas())),
            Target::Recorder(r) => r.recording_canvas().map(f).ok_or_else(|| "nothing is being recorded".into()),
            Target::Done => Err("the recording this canvas belongs to is finished".into()),
        }
    }
}

type P = Option<Instance<SkPaint>>;

fn paint_of(p: &P) -> Option<Paint> {
    inner(p, |p| &p.0)
}

#[class]
impl SkCanvas {
    /// Clears to a color.
    pub fn clear(&self, color: ColorArg) -> Result<(), String> {
        self.draw(|c| {
            c.clear(color.0);
        })
    }
    /// Fills with a color.
    pub fn draw_color(&self, color: ColorArg, mode: Option<i32>) -> Result<(), String> {
        let mode = mode.map(enums::blend).transpose()?;
        self.draw(|c| {
            c.draw_color(color.0, mode);
        })
    }
    /// Fills with a paint.
    pub fn draw_paint(&self, paint: &SkPaint) -> Result<(), String> {
        self.draw(|c| {
            c.draw_paint(&paint.0);
        })
    }
    /// A line.
    pub fn draw_line(&self, x0: f32, y0: f32, x1: f32, y1: f32, paint: &SkPaint) -> Result<(), String> {
        self.draw(|c| {
            c.draw_line((x0, y0), (x1, y1), &paint.0);
        })
    }
    /// A rectangle.
    pub fn draw_rect(&self, rect: RectArg, paint: &SkPaint) -> Result<(), String> {
        self.draw(|c| {
            c.draw_rect(rect.0, &paint.0);
        })
    }
    /// A rounded rectangle.
    pub fn draw_r_rect(&self, rrect: RRectArg, paint: &SkPaint) -> Result<(), String> {
        self.draw(|c| {
            c.draw_rrect(rrect.0, &paint.0);
        })
    }
    /// What is between two rounded rectangles.
    pub fn draw_d_r_rect(&self, outer: RRectArg, inner: RRectArg, paint: &SkPaint) -> Result<(), String> {
        self.draw(|c| {
            c.draw_drrect(outer.0, inner.0, &paint.0);
        })
    }
    /// A circle.
    pub fn draw_circle(&self, cx: f32, cy: f32, radius: f32, paint: &SkPaint) -> Result<(), String> {
        self.draw(|c| {
            c.draw_circle((cx, cy), radius, &paint.0);
        })
    }
    /// An oval.
    pub fn draw_oval(&self, oval: RectArg, paint: &SkPaint) -> Result<(), String> {
        self.draw(|c| {
            c.draw_oval(oval.0, &paint.0);
        })
    }
    /// An arc, by degrees.
    pub fn draw_arc(
        &self,
        oval: RectArg,
        start: f32,
        sweep: f32,
        use_center: bool,
        paint: &SkPaint,
    ) -> Result<(), String> {
        self.draw(|c| {
            c.draw_arc(oval.0, start, sweep, use_center, &paint.0);
        })
    }
    /// A path.
    pub fn draw_path(&self, path: &SkPath, paint: &SkPaint) -> Result<(), String> {
        self.draw(|c| {
            c.draw_path(&path.0, &paint.0);
        })
    }
    /// Points, lines or a polygon.
    pub fn draw_points(&self, mode: i32, points: Points, paint: &SkPaint) -> Result<(), String> {
        let mode = enums::point_mode(mode)?;
        self.draw(|c| {
            c.draw_points(mode, &points.0, &paint.0);
        })
    }
    /// A mesh.
    pub fn draw_vertices(&self, vertices: &SkVertices, mode: i32, paint: &SkPaint) -> Result<(), String> {
        let mode = enums::blend(mode)?;
        self.draw(|c| {
            c.draw_vertices(&vertices.0, mode, &paint.0);
        })
    }
    /// A Coons patch.
    pub fn draw_patch(
        &self,
        cubics: Points,
        colors: Option<Colors>,
        texs: Option<Points>,
        mode: Option<i32>,
        paint: P,
    ) -> Result<(), String> {
        let cubics = cubics.0.try_into().map_err(|_| "a patch has 12 points")?;
        let colors = colors.map(|c| four_colors(&c.0)).transpose()?;
        let texs = texs.map(|t| t.0.try_into().map_err(|_| "a patch has 4 textures")).transpose()?;
        let mode = mode.map(enums::blend).transpose()?.unwrap_or(default_blend(colors.is_some()));
        let paint = paint_of(&paint).unwrap_or_default();
        self.draw(|c| {
            c.draw_patch(&cubics, colors.as_ref(), texs.as_ref(), mode, &paint);
        })
    }
    /// Sprites of an image.
    pub fn draw_atlas(
        &self,
        atlas: &SkImage,
        sprites: Rects,
        transforms: Transforms,
        paint: &SkPaint,
        mode: Option<i32>,
        colors: Option<Colors>,
        sampling: Option<Value>,
    ) -> Result<(), String> {
        let mode = mode.map(enums::blend).transpose()?;
        let colors = colors.map(|c| c.0);
        let sampling = sampling_of(sampling.as_ref())?;
        self.draw(|c| atlas_on(c, &atlas.0, &sprites.0, &transforms.0, colors.as_deref(), mode, sampling, &paint.0))?
    }
    /// An image.
    pub fn draw_image(&self, image: &SkImage, x: f32, y: f32, paint: P) -> Result<(), String> {
        let paint = paint_of(&paint);
        self.draw(|c| {
            c.draw_image(&image.0, (x, y), paint.as_ref());
        })
    }
    /// An image, sampled so.
    pub fn draw_image_options(
        &self,
        image: &SkImage,
        x: f32,
        y: f32,
        filter: i32,
        mipmap: i32,
        paint: P,
    ) -> Result<(), String> {
        let (paint, sampling) = (paint_of(&paint), enums::sampling(Some(filter), Some(mipmap))?);
        self.draw(|c| {
            c.draw_image_with_sampling_options(&image.0, (x, y), sampling, paint.as_ref());
        })
    }
    /// An image, sampled by a cubic.
    pub fn draw_image_cubic(&self, image: &SkImage, x: f32, y: f32, b: f32, c: f32, paint: P) -> Result<(), String> {
        let paint = paint_of(&paint);
        self.draw(|canvas| {
            canvas.draw_image_with_sampling_options(&image.0, (x, y), CubicResampler { b, c }, paint.as_ref());
        })
    }
    /// A part of an image, in a rectangle.
    pub fn draw_image_rect(
        &self,
        image: &SkImage,
        src: RectArg,
        dest: RectArg,
        paint: &SkPaint,
        fast_sample: Option<bool>,
    ) -> Result<(), String> {
        let constraint = if fast_sample == Some(true) { SrcRectConstraint::Fast } else { SrcRectConstraint::Strict };
        self.draw(|c| {
            c.draw_image_rect(&image.0, Some((&src.0, constraint)), dest.0, &paint.0);
        })
    }
    /// A part of an image, in a rectangle, sampled so.
    pub fn draw_image_rect_options(
        &self,
        image: &SkImage,
        src: RectArg,
        dest: RectArg,
        filter: i32,
        mipmap: i32,
        paint: P,
    ) -> Result<(), String> {
        let sampling = enums::sampling(Some(filter), Some(mipmap))?;
        let paint = paint_of(&paint).unwrap_or_default();
        let src = Some((&src.0, SrcRectConstraint::Strict));
        self.draw(|c| {
            c.draw_image_rect_with_sampling_options(&image.0, src, dest.0, sampling, &paint);
        })
    }
    /// A part of an image, in a rectangle, sampled by a cubic.
    pub fn draw_image_rect_cubic(
        &self,
        image: &SkImage,
        src: RectArg,
        dest: RectArg,
        b: f32,
        c: f32,
        paint: P,
    ) -> Result<(), String> {
        let paint = paint_of(&paint).unwrap_or_default();
        let src = Some((&src.0, SrcRectConstraint::Strict));
        self.draw(|canvas| {
            let sampling = CubicResampler { b, c };
            canvas.draw_image_rect_with_sampling_options(&image.0, src, dest.0, sampling, &paint);
        })
    }
    /// An image whose edges do not stretch.
    pub fn draw_image_nine(
        &self,
        image: &SkImage,
        center: RectArg,
        dest: RectArg,
        filter: i32,
        paint: P,
    ) -> Result<(), String> {
        let (paint, filter) = (paint_of(&paint), enums::filter(filter)?);
        self.draw(|c| {
            c.draw_image_nine(&image.0, center.0.round(), dest.0, filter, paint.as_ref());
        })
    }
    /// A line of text.
    pub fn draw_text(&self, text: String, x: f32, y: f32, paint: &SkPaint, font: &SkFont) -> Result<(), String> {
        self.draw(|c| {
            c.draw_str(text, (x, y), &font.0, &paint.0);
        })
    }
    /// A text blob.
    pub fn draw_text_blob(&self, blob: &SkTextBlob, x: f32, y: f32, paint: &SkPaint) -> Result<(), String> {
        self.draw(|c| {
            c.draw_text_blob(&blob.0, (x, y), &paint.0);
        })
    }
    /// Glyphs.
    pub fn draw_glyphs(
        &self,
        glyphs: Floats,
        positions: Points,
        x: f32,
        y: f32,
        font: &SkFont,
        paint: &SkPaint,
    ) -> Result<(), String> {
        if glyphs.0.len() != positions.0.len() {
            return Err("glyphs need one position each".into());
        }
        let glyphs: Vec<u16> = glyphs.0.iter().map(|g| *g as u16).collect();
        self.draw(|c| c.draw_glyphs_at(&glyphs, &positions.0[..], (x, y), &font.0, &paint.0))
    }
    /// An SVG document.
    pub fn draw_svg(&self, svg: &SkSvg, width: Option<f32>, height: Option<f32>) -> Result<(), String> {
        self.draw(|c| svg_on(c, &svg.0, width, height))
    }
    /// A picture.
    pub fn draw_picture(&self, picture: &SkPicture) -> Result<(), String> {
        self.draw(|c| {
            c.draw_picture(&picture.0, None, None);
        })
    }
    /// Saves the matrix and the clip.
    pub fn save(&self) -> Result<usize, String> {
        self.draw(Canvas::save)
    }
    /// Saves, and draws on a layer of its own until the restore.
    pub fn save_layer(
        &self,
        paint: P,
        bounds: Option<RectArg>,
        backdrop: Option<Instance<SkImageFilter>>,
        flags: Option<u32>,
    ) -> Result<usize, String> {
        let (paint, backdrop) = (paint_of(&paint), inner(&backdrop, |b| &b.0));
        let bounds = bounds.map(|b| b.0);
        let mut rec = SaveLayerRec::default().flags(SaveLayerFlags::from_bits_truncate(flags.unwrap_or(0)));
        if let Some(p) = &paint {
            rec = rec.paint(p);
        }
        if let Some(b) = &bounds {
            rec = rec.bounds(b);
        }
        if let Some(b) = &backdrop {
            rec = rec.backdrop(b);
        }
        self.draw(|c| c.save_layer(&rec))
    }
    /// Restores the last save.
    pub fn restore(&self) -> Result<(), String> {
        self.draw(|c| {
            c.restore();
        })
    }
    /// Restores to a save.
    pub fn restore_to_count(&self, count: usize) -> Result<(), String> {
        self.draw(|c| {
            c.restore_to_count(count);
        })
    }
    /// How many saves there are.
    pub fn get_save_count(&self) -> Result<usize, String> {
        self.draw(Canvas::save_count)
    }
    /// Rotates by degrees around a point.
    pub fn rotate(&self, degrees: f32, rx: f32, ry: f32) -> Result<(), String> {
        self.draw(|c| {
            c.rotate(degrees, Some((rx, ry).into()));
        })
    }
    /// Scales.
    pub fn scale(&self, sx: f32, sy: f32) -> Result<(), String> {
        self.draw(|c| {
            c.scale((sx, sy));
        })
    }
    /// Skews.
    pub fn skew(&self, sx: f32, sy: f32) -> Result<(), String> {
        self.draw(|c| {
            c.skew((sx, sy));
        })
    }
    /// Moves.
    pub fn translate(&self, dx: f32, dy: f32) -> Result<(), String> {
        self.draw(|c| {
            c.translate((dx, dy));
        })
    }
    /// Multiplies the matrix.
    pub fn concat(&self, matrix: MatrixArg) -> Result<(), String> {
        self.draw(|c| {
            c.concat(&matrix.0);
        })
    }
    /// The matrix.
    pub fn get_total_matrix(&self) -> Result<SkMatrix, String> {
        self.draw(|c| c.local_to_device_as_3x3().into())
    }
    /// Clips to a rectangle.
    pub fn clip_rect(&self, rect: RectArg, op: i32, anti_alias: bool) -> Result<(), String> {
        let op = enums::clip_op(op)?;
        self.draw(|c| {
            c.clip_rect(rect.0, op, anti_alias);
        })
    }
    /// Clips to a rounded rectangle.
    pub fn clip_r_rect(&self, rrect: RRectArg, op: i32, anti_alias: bool) -> Result<(), String> {
        let op = enums::clip_op(op)?;
        self.draw(|c| {
            c.clip_rrect(rrect.0, op, anti_alias);
        })
    }
    /// Clips to a path.
    pub fn clip_path(&self, path: &SkPath, op: i32, anti_alias: bool) -> Result<(), String> {
        let op = enums::clip_op(op)?;
        self.draw(|c| {
            c.clip_path(&path.0, op, anti_alias);
        })
    }
}

pub(crate) fn default_blend(colors: bool) -> skia_safe::BlendMode {
    if colors { skia_safe::BlendMode::DstOver } else { skia_safe::BlendMode::SrcOver }
}

pub(crate) fn four_colors(colors: &[skia_safe::Color4f]) -> Result<[skia_safe::Color; 4], String> {
    match colors {
        [a, b, c, d] => Ok([a.to_color(), b.to_color(), c.to_color(), d.to_color()]),
        _ => Err("a patch has 4 colors".into()),
    }
}

pub(crate) fn atlas_on(
    canvas: &Canvas,
    image: &skia_safe::Image,
    sprites: &[Rect],
    transforms: &[skia_safe::RSXform],
    colors: Option<&[skia_safe::Color4f]>,
    mode: Option<skia_safe::BlendMode>,
    sampling: skia_safe::SamplingOptions,
    paint: &Paint,
) -> Result<(), String> {
    if sprites.len() != transforms.len() || colors.is_some_and(|c| c.len() != sprites.len()) {
        return Err("an atlas needs as many transforms (and colors) as sprites".into());
    }
    let colors: Option<Vec<_>> = colors.map(|c| c.iter().map(|c| c.to_color()).collect());
    let mode = mode.unwrap_or(skia_safe::BlendMode::DstOver);
    canvas.draw_atlas(image, transforms, sprites, colors.as_deref(), mode, sampling, None, paint);
    Ok(())
}

pub(crate) fn svg_on(canvas: &Canvas, svg: &skia_safe::svg::Dom, width: Option<f32>, height: Option<f32>) {
    let mut svg = svg.clone();
    if let (Some(w), Some(h)) = (width, height) {
        svg.set_container_size((w, h));
    }
    svg.render(canvas);
}

/// Pixels to draw on, in memory.
#[class]
pub struct SkSurface {
    target: Shared,
    width: i32,
    height: i32,
}

pub(crate) fn raster(width: i32, height: i32) -> Result<Surface, String> {
    let info = ImageInfo::new((width, height), ColorType::RGBA8888, AlphaType::Premul, None);
    surfaces::raster(&info, None, None).ok_or_else(bad("surface"))
}

#[class]
impl SkSurface {
    /// A surface of `width` by `height` pixels (`null`: too large, or empty).
    pub fn make(width: i32, height: i32) -> Option<Instance<SkSurface>> {
        let surface = raster(width, height).ok()?;
        Some(Instance::new(SkSurface { target: Rc::new(RefCell::new(Target::Surface(surface))), width, height }))
    }
    /// What draws on it.
    pub fn get_canvas(&self) -> SkCanvas {
        SkCanvas(self.target.clone())
    }
    /// What was drawn.
    pub fn make_image_snapshot(&self, bounds: Option<RectArg>) -> Result<SkImage, String> {
        let Target::Surface(s) = &mut *self.target.borrow_mut() else { return Err("no surface".into()) };
        match bounds {
            None => Ok(s.image_snapshot().into()),
            Some(b) => s.image_snapshot_with_bounds(b.0.round()).map(Into::into).ok_or_else(bad("snapshot")),
        }
    }
    /// What was drawn, RGBA 8888 unpremultiplied, row by row.
    pub fn pixels(&self) -> Result<Vec<u8>, String> {
        let Target::Surface(s) = &mut *self.target.borrow_mut() else { return Err("no surface".into()) };
        read_pixels(&s.image_snapshot())
    }
    /// Nothing: a surface in memory draws at once.
    pub fn flush(&self) {}
    /// Its width.
    pub fn width(&self) -> i32 {
        self.width
    }
    /// Its height.
    pub fn height(&self) -> i32 {
        self.height
    }
}

/// Records drawing.
#[class]
pub struct SkPictureRecorder(Shared);

/// No bounds: what Skia takes as "everything".
pub(crate) const EVERYTHING: Rect = Rect { left: -1e9, top: -1e9, right: 1e9, bottom: 1e9 };

pub(crate) fn record(bounds: Rect, f: impl FnOnce(&Canvas) -> Result<(), String>) -> Result<Picture, String> {
    let mut recorder = PictureRecorder::new();
    f(recorder.begin_recording(bounds, false))?;
    recorder.finish_recording_as_picture(None).ok_or_else(bad("picture"))
}

#[class]
impl SkPictureRecorder {
    /// A recorder.
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Target::Done)))
    }
    /// Starts.
    pub fn begin_recording(&mut self, bounds: Option<RectArg>) -> SkCanvas {
        let mut recorder = PictureRecorder::new();
        recorder.begin_recording(bounds.map_or(EVERYTHING, |b| b.0), false);
        // A canvas of the last recording must not draw on this one.
        self.0 = Rc::new(RefCell::new(Target::Recorder(recorder)));
        SkCanvas(self.0.clone())
    }
    /// What was recorded.
    pub fn finish_recording_as_picture(&self) -> Result<SkPicture, String> {
        match self.0.replace(Target::Done) {
            Target::Recorder(mut r) => r.finish_recording_as_picture(None).map(Into::into).ok_or_else(bad("picture")),
            _ => Err("nothing is being recorded".into()),
        }
    }
}
