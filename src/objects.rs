//! Skia's immutable objects: shaders, filters, effects, images, fonts, pictures.

use nsh_core::{
    element::{Arg, Class, JsIn},
    prelude::*,
};
use skia_safe::{
    AlphaType, Color4f, ColorType, Data, Font, FontHinting, FontMgr, FontStyle, GlyphId, ISize, Image, ImageFilter,
    ImageInfo, MaskFilter, Matrix, PathEffect, Picture, Point, RuntimeEffect, Shader, TextBlob, Typeface, Vertices,
    color_filters, dash_path_effect, font, font_style, gradient, image_filters, images, path_1d_path_effect,
    path_2d_path_effect, runtime_effect, shaders, svg, vertices::VertexMode,
};

use crate::{
    args::{Bytes, ColorArg, Colors, Floats, MatrixArg, Points, Pt, RectArg, RectOut, Transforms, enums},
    paint::SkPaint,
    path::bad,
};

/// The instance of `C` a JS value is, if it is one.
pub(crate) fn get<C: Class>(value: JsIn<'_>) -> Option<Instance<C>> {
    value.host().instance(value.raw())?;
    <Instance<C> as Arg>::from_js(value).ok()
}

macro_rules! wrapper {
    ($(#[$doc:meta])* $name:ident($inner:ty)) => {
        $(#[$doc])*
        #[class]
        pub struct $name(pub(crate) $inner);
        impl From<$inner> for $name {
            fn from(inner: $inner) -> Self {
                Self(inner)
            }
        }
    };
}
pub(crate) use wrapper;

/// The Skia value inside an optional argument.
pub(crate) fn inner<C: Class, T: Clone>(arg: &Option<Instance<C>>, field: fn(&C) -> &T) -> Option<T> {
    arg.as_ref().map(|i| i.with(|c| field(c).clone()))
}

wrapper!(/** Bytes held by Skia. */ SkData(Data));
wrapper!(/** A matrix: 3 rows of 3 numbers. */ SkMatrix(Matrix));
wrapper!(/** A shader: the color of each pixel. */ SkShader(Shader));
wrapper!(/** A filter of colors. */ SkColorFilter(skia_safe::ColorFilter));
wrapper!(/** A filter of what is drawn. */ SkImageFilter(ImageFilter));
wrapper!(/** A filter of a drawing's coverage. */ SkMaskFilter(MaskFilter));
wrapper!(/** An effect on a path. */ SkPathEffect(PathEffect));
wrapper!(/** An image. */ SkImage(Image));
wrapper!(/** Recorded drawing. */ SkPicture(Picture));
wrapper!(/** A typeface. */ SkTypeface(Typeface));
wrapper!(/** The fonts of the system. */ SkFontMgr(FontMgr));
wrapper!(/** A typeface at a size. */ SkFont(Font));
wrapper!(/** Positioned glyphs. */ SkTextBlob(TextBlob));
wrapper!(/** A mesh. */ SkVertices(Vertices));
wrapper!(/** An SVG document. */ SkSvg(svg::Dom));

#[class]
impl SkData {
    /// A copy of `bytes`.
    pub fn from_bytes(bytes: Bytes) -> SkData {
        Data::new_copy(&bytes.0).into()
    }
    /// The bytes.
    pub fn bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

#[class]
impl SkMatrix {
    /// The identity, or the matrix of 9 or 16 numbers.
    pub fn new(matrix: Option<MatrixArg>) -> Self {
        Self(matrix.map_or_else(Matrix::default, |m| m.0))
    }
    /// This matrix times `matrix`.
    pub fn concat(this: Instance<Self>, matrix: MatrixArg) -> Instance<Self> {
        this.with_mut(|m| m.0.pre_concat(&matrix.0).to_owned());
        this
    }
    /// Moves, before this matrix.
    pub fn translate(this: Instance<Self>, x: f32, y: f32) -> Instance<Self> {
        this.with_mut(|m| m.0.pre_translate((x, y)).to_owned());
        this
    }
    /// Scales, before this matrix.
    pub fn scale(this: Instance<Self>, x: f32, y: Option<f32>) -> Instance<Self> {
        this.with_mut(|m| m.0.pre_scale((x, y.unwrap_or(x)), None).to_owned());
        this
    }
    /// Skews, before this matrix.
    pub fn skew(this: Instance<Self>, x: f32, y: f32) -> Instance<Self> {
        this.with_mut(|m| m.0.pre_skew((x, y), None).to_owned());
        this
    }
    /// Rotates by radians, before this matrix.
    pub fn rotate(this: Instance<Self>, theta: f32) -> Instance<Self> {
        this.with_mut(|m| m.0.pre_rotate(theta.to_degrees(), None).to_owned());
        this
    }
    /// Moves, after this matrix.
    pub fn post_translate(this: Instance<Self>, x: f32, y: f32) -> Instance<Self> {
        this.with_mut(|m| m.0.post_translate((x, y)).to_owned());
        this
    }
    /// Scales, after this matrix.
    pub fn post_scale(this: Instance<Self>, x: f32, y: Option<f32>) -> Instance<Self> {
        this.with_mut(|m| m.0.post_scale((x, y.unwrap_or(x)), None).to_owned());
        this
    }
    /// Skews, after this matrix.
    pub fn post_skew(this: Instance<Self>, x: f32, y: f32) -> Instance<Self> {
        this.with_mut(|m| m.0.post_skew((x, y), None).to_owned());
        this
    }
    /// Rotates by radians, after this matrix.
    pub fn post_rotate(this: Instance<Self>, theta: f32) -> Instance<Self> {
        this.with_mut(|m| m.0.post_rotate(theta.to_degrees(), None).to_owned());
        this
    }
    /// Makes it the identity.
    pub fn identity(this: Instance<Self>) -> Instance<Self> {
        this.with_mut(|m| m.0 = Matrix::default());
        this
    }
    /// Its 9 numbers, row by row.
    pub fn get(&self) -> Vec<f32> {
        let mut out = [0.0; 9];
        self.0.get_9(&mut out);
        out.to_vec()
    }
}

/// A gradient's colors as Skia takes them.
pub(crate) fn with_gradient(
    colors: &[Color4f],
    pos: Option<&[f32]>,
    mode: skia_safe::TileMode,
    flags: u32,
    make: impl FnOnce(&gradient::Gradient<'_>) -> Option<Shader>,
) -> Result<Shader, String> {
    if pos.is_some_and(|p| p.len() != colors.len()) {
        return Err("a gradient needs one position per color".into());
    }
    let colors = gradient::Colors::new(colors, pos, mode, None);
    make(&gradient::Gradient::new(colors, gradient::Interpolation::from_flags(flags))).ok_or_else(bad("gradient"))
}

#[class]
impl SkShader {
    /// One color.
    pub fn make_color(color: ColorArg) -> SkShader {
        shaders::color_in_space(color.0, skia_safe::ColorSpace::new_srgb()).into()
    }
    /// Colors along a line.
    pub fn make_linear_gradient(
        start: Pt,
        end: Pt,
        colors: Colors,
        pos: Option<Floats>,
        mode: i32,
        local_matrix: Option<MatrixArg>,
        flags: Option<u32>,
    ) -> Result<SkShader, String> {
        let m = local_matrix.map(|m| m.0);
        with_gradient(&colors.0, pos.as_ref().map(|p| &p.0[..]), enums::tile(mode)?, flags.unwrap_or(0), |g| {
            gradient::shaders::linear_gradient((start.0, end.0), g, m.as_ref())
        })
        .map(Into::into)
    }
    /// Colors from a center out.
    pub fn make_radial_gradient(
        center: Pt,
        radius: f32,
        colors: Colors,
        pos: Option<Floats>,
        mode: i32,
        local_matrix: Option<MatrixArg>,
        flags: Option<u32>,
    ) -> Result<SkShader, String> {
        let m = local_matrix.map(|m| m.0);
        with_gradient(&colors.0, pos.as_ref().map(|p| &p.0[..]), enums::tile(mode)?, flags.unwrap_or(0), |g| {
            gradient::shaders::radial_gradient((center.0, radius), g, m.as_ref())
        })
        .map(Into::into)
    }
    /// Colors between two circles.
    pub fn make_two_point_conical_gradient(
        start: Pt,
        start_radius: f32,
        end: Pt,
        end_radius: f32,
        colors: Colors,
        pos: Option<Floats>,
        mode: i32,
        local_matrix: Option<MatrixArg>,
        flags: Option<u32>,
    ) -> Result<SkShader, String> {
        let m = local_matrix.map(|m| m.0);
        with_gradient(&colors.0, pos.as_ref().map(|p| &p.0[..]), enums::tile(mode)?, flags.unwrap_or(0), |g| {
            gradient::shaders::two_point_conical_gradient((start.0, start_radius), (end.0, end_radius), g, m.as_ref())
        })
        .map(Into::into)
    }
    /// Colors around a center.
    pub fn make_sweep_gradient(
        cx: f32,
        cy: f32,
        colors: Colors,
        pos: Option<Floats>,
        mode: i32,
        local_matrix: Option<MatrixArg>,
        flags: Option<u32>,
        start_angle: Option<f32>,
        end_angle: Option<f32>,
    ) -> Result<SkShader, String> {
        let m = local_matrix.map(|m| m.0);
        let angles = (start_angle.unwrap_or(0.0), end_angle.unwrap_or(360.0));
        with_gradient(&colors.0, pos.as_ref().map(|p| &p.0[..]), enums::tile(mode)?, flags.unwrap_or(0), |g| {
            gradient::shaders::sweep_gradient((cx, cy), angles, g, m.as_ref())
        })
        .map(Into::into)
    }
    /// Perlin turbulence.
    pub fn make_turbulence(x: f32, y: f32, octaves: u32, seed: f32, w: f32, h: f32) -> Result<SkShader, String> {
        noise(false, x, y, octaves, seed, w, h).map(Into::into)
    }
    /// Perlin fractal noise.
    pub fn make_fractal_noise(x: f32, y: f32, octaves: u32, seed: f32, w: f32, h: f32) -> Result<SkShader, String> {
        noise(true, x, y, octaves, seed, w, h).map(Into::into)
    }
    /// `one` under `two`.
    pub fn make_blend(mode: i32, one: &SkShader, two: &SkShader) -> Result<SkShader, String> {
        Ok(shaders::blend(enums::blend(mode)?, one.0.clone(), two.0.clone()).into())
    }
}

pub(crate) fn noise(fractal: bool, x: f32, y: f32, octaves: u32, seed: f32, w: f32, h: f32) -> Result<Shader, String> {
    let tile = (w > 0.0 && h > 0.0).then(|| ISize::new(w as i32, h as i32));
    let make = if fractal { shaders::fractal_noise } else { shaders::turbulence };
    make((x, y), octaves as usize, seed, tile).ok_or_else(bad("noise"))
}

#[class]
impl SkColorFilter {
    /// Multiplies by a 4x5 matrix.
    pub fn make_matrix(matrix: Floats) -> Result<SkColorFilter, String> {
        let m: [f32; 20] = matrix.0.try_into().map_err(|_| "a color matrix has 20 numbers")?;
        Ok(color_filters::matrix_row_major(&m, None).into())
    }
    /// Blends with a color.
    pub fn make_blend(color: ColorArg, mode: i32) -> Result<SkColorFilter, String> {
        color_filters::blend_with_color_space(color.0, None, enums::blend(mode)?)
            .map(Into::into)
            .ok_or_else(bad("color filter"))
    }
    /// `inner`, then `outer`.
    pub fn make_compose(outer: &SkColorFilter, inner: &SkColorFilter) -> Result<SkColorFilter, String> {
        color_filters::compose(outer.0.clone(), inner.0.clone()).map(Into::into).ok_or_else(bad("color filter"))
    }
    /// Between two filters.
    pub fn make_lerp(t: f32, dst: &SkColorFilter, src: &SkColorFilter) -> Result<SkColorFilter, String> {
        color_filters::lerp(t, dst.0.clone(), src.0.clone()).map(Into::into).ok_or_else(bad("color filter"))
    }
    /// Linear to sRGB.
    #[allow(non_snake_case)]
    pub fn make_linear_to_srgb_gamma() -> SkColorFilter {
        color_filters::linear_to_srgb_gamma().into()
    }
    /// sRGB to linear.
    pub fn make_srgb_to_linear_gamma() -> SkColorFilter {
        color_filters::srgb_to_linear_gamma().into()
    }
    /// Luminance to alpha.
    pub fn make_luma() -> SkColorFilter {
        skia_safe::luma_color_filter::new().into()
    }
}

#[class]
impl SkMaskFilter {
    /// A blur.
    pub fn make_blur(style: i32, sigma: f32, respect_ctm: bool) -> Result<SkMaskFilter, String> {
        MaskFilter::blur(enums::blur_style(style)?, sigma, respect_ctm).map(Into::into).ok_or_else(bad("mask filter"))
    }
}

#[class]
impl SkPathEffect {
    /// Rounds corners.
    pub fn make_corner(radius: f32) -> Option<Instance<SkPathEffect>> {
        PathEffect::corner_path(radius).map(|p| Instance::new(p.into()))
    }
    /// Dashes.
    pub fn make_dash(intervals: Floats, phase: Option<f32>) -> Result<SkPathEffect, String> {
        dash_path_effect::new(&intervals.0, phase.unwrap_or(0.0)).map(Into::into).ok_or_else(bad("dash"))
    }
    /// Breaks lines up.
    pub fn make_discrete(length: f32, deviation: f32, seed: u32) -> Result<SkPathEffect, String> {
        PathEffect::discrete(length, deviation, seed).map(Into::into).ok_or_else(bad("path effect"))
    }
    /// `inner`, then `outer`.
    pub fn make_compose(outer: &SkPathEffect, inner: &SkPathEffect) -> SkPathEffect {
        PathEffect::compose(outer.0.clone(), inner.0.clone()).into()
    }
    /// Both.
    pub fn make_sum(one: &SkPathEffect, two: &SkPathEffect) -> SkPathEffect {
        PathEffect::sum(one.0.clone(), two.0.clone()).into()
    }
    /// Lines.
    pub fn make_line_2d(width: f32, matrix: MatrixArg) -> Option<Instance<SkPathEffect>> {
        PathEffect::line_2d(width, &matrix.0).map(|p| Instance::new(p.into()))
    }
    /// A path along the path.
    pub fn make_path_1d(
        path: &crate::path::SkPath,
        advance: f32,
        phase: f32,
        style: i32,
    ) -> Result<Option<Instance<SkPathEffect>>, String> {
        let effect = path_1d_path_effect::new(&path.0, advance, phase, enums::path_1d(style)?);
        Ok(effect.map(|p| Instance::new(p.into())))
    }
    /// A path as a pattern.
    pub fn make_path_2d(matrix: MatrixArg, path: &crate::path::SkPath) -> Option<Instance<SkPathEffect>> {
        Some(Instance::new(path_2d_path_effect::new(&matrix.0, &path.0).into()))
    }
}

fn filter(made: Option<ImageFilter>) -> Result<SkImageFilter, String> {
    made.map(Into::into).ok_or_else(bad("image filter"))
}

fn crop(rect: Option<RectArg>) -> image_filters::CropRect {
    rect.map_or(image_filters::CropRect::NO_CROP_RECT, |r| r.0.into())
}

type Input = Option<Instance<SkImageFilter>>;

fn input(arg: &Input) -> Option<ImageFilter> {
    inner(arg, |f| &f.0)
}

#[class]
impl SkImageFilter {
    /// Moves.
    pub fn make_offset(dx: f32, dy: f32, source: Input, crop_rect: Option<RectArg>) -> Result<SkImageFilter, String> {
        filter(image_filters::offset((dx, dy), input(&source), crop(crop_rect)))
    }
    /// Moves each pixel by another image's.
    pub fn make_displacement_map(
        channel_x: i32,
        channel_y: i32,
        scale: f32,
        map: &SkImageFilter,
        source: Input,
        crop_rect: Option<RectArg>,
    ) -> Result<SkImageFilter, String> {
        let channels = (enums::channel(channel_x)?, enums::channel(channel_y)?);
        filter(image_filters::displacement_map(channels, scale, map.0.clone(), input(&source), crop(crop_rect)))
    }
    /// A shader's pixels.
    pub fn make_shader(
        shader: &SkShader,
        dither: Option<bool>,
        crop_rect: Option<RectArg>,
    ) -> Result<SkImageFilter, String> {
        let dither = if dither == Some(true) { image_filters::Dither::Yes } else { image_filters::Dither::No };
        filter(image_filters::shader_with_dither(shader.0.clone(), dither, crop(crop_rect)))
    }
    /// Blurs.
    pub fn make_blur(
        sigma_x: f32,
        sigma_y: f32,
        mode: i32,
        source: Input,
        crop_rect: Option<RectArg>,
    ) -> Result<SkImageFilter, String> {
        filter(image_filters::blur((sigma_x, sigma_y), enums::tile(mode)?, input(&source), crop(crop_rect)))
    }
    /// A color filter.
    pub fn make_color_filter(
        color_filter: &SkColorFilter,
        source: Input,
        crop_rect: Option<RectArg>,
    ) -> Result<SkImageFilter, String> {
        filter(image_filters::color_filter(color_filter.0.clone(), input(&source), crop(crop_rect)))
    }
    /// `inner`, then `outer`.
    pub fn make_compose(outer: Input, inner: Input) -> Result<SkImageFilter, String> {
        match (input(&outer), input(&inner)) {
            (Some(outer), Some(inner)) => filter(image_filters::compose(outer, inner)),
            (Some(one), None) | (None, Some(one)) => Ok(one.into()),
            (None, None) => Err("nothing to compose".into()),
        }
    }
    /// A shadow under it.
    pub fn make_drop_shadow(
        dx: f32,
        dy: f32,
        sigma_x: f32,
        sigma_y: f32,
        color: ColorArg,
        source: Input,
        crop_rect: Option<RectArg>,
    ) -> Result<SkImageFilter, String> {
        filter(image_filters::drop_shadow((dx, dy), (sigma_x, sigma_y), color.0, None, input(&source), crop(crop_rect)))
    }
    /// Its shadow only.
    pub fn make_drop_shadow_only(
        dx: f32,
        dy: f32,
        sigma_x: f32,
        sigma_y: f32,
        color: ColorArg,
        source: Input,
        crop_rect: Option<RectArg>,
    ) -> Result<SkImageFilter, String> {
        filter(image_filters::drop_shadow_only(
            (dx, dy),
            (sigma_x, sigma_y),
            color.0,
            None,
            input(&source),
            crop(crop_rect),
        ))
    }
    /// Shrinks.
    pub fn make_erode(rx: f32, ry: f32, source: Input, crop_rect: Option<RectArg>) -> Result<SkImageFilter, String> {
        filter(image_filters::erode((rx, ry), input(&source), crop(crop_rect)))
    }
    /// Grows.
    pub fn make_dilate(rx: f32, ry: f32, source: Input, crop_rect: Option<RectArg>) -> Result<SkImageFilter, String> {
        filter(image_filters::dilate((rx, ry), input(&source), crop(crop_rect)))
    }
    /// Blends two.
    pub fn make_blend(
        mode: i32,
        background: &SkImageFilter,
        foreground: Input,
        crop_rect: Option<RectArg>,
    ) -> Result<SkImageFilter, String> {
        filter(image_filters::blend(enums::blend(mode)?, background.0.clone(), input(&foreground), crop(crop_rect)))
    }
    /// Draws nothing.
    pub fn make_empty() -> SkImageFilter {
        image_filters::empty().into()
    }
    /// Cuts to a rectangle.
    pub fn make_crop(rect: RectArg, tile_mode: Option<i32>, source: Input) -> Result<SkImageFilter, String> {
        let tile = tile_mode.map(enums::tile).transpose()?;
        filter(image_filters::crop(rect.0, tile, input(&source)))
    }
    /// Combines two by 4 factors.
    pub fn make_arithmetic(
        k1: f32,
        k2: f32,
        k3: f32,
        k4: f32,
        enforce_pm_color: bool,
        background: Input,
        foreground: Input,
        crop_rect: Option<RectArg>,
    ) -> Result<SkImageFilter, String> {
        filter(image_filters::arithmetic(
            k1,
            k2,
            k3,
            k4,
            enforce_pm_color,
            input(&background),
            input(&foreground),
            crop(crop_rect),
        ))
    }
    /// A runtime shader of what is drawn.
    pub fn make_runtime_shader(
        builder: &SkRuntimeShaderBuilder,
        child_shader_name: Option<String>,
        source: Input,
    ) -> Result<SkImageFilter, String> {
        let name = child_shader_name.unwrap_or_default();
        filter(image_filters::runtime_shader(&builder.0, name, input(&source)))
    }
}

#[class]
impl SkImage {
    /// Decodes PNG, JPEG, WebP, GIF or BMP bytes.
    pub fn from_encoded(data: &SkData) -> Option<Instance<SkImage>> {
        Image::from_encoded(data.0.clone())
            .and_then(|i| i.make_raster_image(None, None))
            .map(|i| Instance::new(i.into()))
    }
    /// An image of raw pixels (RGBA 8888, or BGRA when `bgra`).
    pub fn from_pixels(
        width: i32,
        height: i32,
        bgra: bool,
        alpha_type: i32,
        data: &SkData,
        bytes_per_row: usize,
    ) -> Option<Instance<SkImage>> {
        let color = if bgra { ColorType::BGRA8888 } else { ColorType::RGBA8888 };
        let alpha = match alpha_type {
            1 => AlphaType::Opaque,
            3 => AlphaType::Unpremul,
            _ => AlphaType::Premul,
        };
        let info = ImageInfo::new((width, height), color, alpha, None);
        images::raster_from_data(&info, data.0.clone(), bytes_per_row).map(|i| Instance::new(i.into()))
    }
    /// Its width in pixels.
    pub fn width(&self) -> i32 {
        self.0.width()
    }
    /// Its height in pixels.
    pub fn height(&self) -> i32 {
        self.0.height()
    }
    /// A shader of it.
    pub fn make_shader_options(
        &self,
        tx: i32,
        ty: i32,
        filter: Option<i32>,
        mipmap: Option<i32>,
        local_matrix: Option<MatrixArg>,
    ) -> Result<SkShader, String> {
        let tiles = (enums::tile(tx)?, enums::tile(ty)?);
        let m = local_matrix.map(|m| m.0);
        self.0.to_shader(tiles, enums::sampling(filter, mipmap)?, m.as_ref()).map(Into::into).ok_or_else(bad("shader"))
    }
    /// A shader of it, sampled by a cubic.
    pub fn make_shader_cubic(
        &self,
        tx: i32,
        ty: i32,
        b: f32,
        c: f32,
        local_matrix: Option<MatrixArg>,
    ) -> Result<SkShader, String> {
        let tiles = (enums::tile(tx)?, enums::tile(ty)?);
        let m = local_matrix.map(|m| m.0);
        let sampling = skia_safe::CubicResampler { b, c };
        self.0.to_shader(tiles, sampling, m.as_ref()).map(Into::into).ok_or_else(bad("shader"))
    }
    /// Its bytes as PNG (4, the default), JPEG (3) or WebP (6).
    pub fn encode(&self, format: Option<i32>, quality: Option<u32>) -> Result<Vec<u8>, String> {
        let data =
            self.0.encode(None::<&mut skia_safe::gpu::DirectContext>, enums::format(format), quality.unwrap_or(100));
        data.map(|d| d.as_bytes().to_vec()).ok_or_else(bad("encoding"))
    }
    /// Its pixels, RGBA 8888 unpremultiplied, row by row.
    pub fn pixels(&self) -> Result<Vec<u8>, String> {
        read_pixels(&self.0)
    }
}

pub(crate) fn read_pixels(image: &Image) -> Result<Vec<u8>, String> {
    let info = ImageInfo::new(image.dimensions(), ColorType::RGBA8888, AlphaType::Unpremul, None);
    let row = image.width() as usize * 4;
    let mut out = vec![0u8; row * image.height() as usize];
    image
        .read_pixels(&info, &mut out, row, (0, 0), skia_safe::image::CachingHint::Allow)
        .then_some(out)
        .ok_or_else(bad("pixels"))
}

#[class]
impl SkPicture {
    /// A picture `serialize()` gave.
    pub fn from_bytes(bytes: Bytes) -> Option<Instance<SkPicture>> {
        Picture::from_bytes(&bytes.0).map(|p| Instance::new(p.into()))
    }
    /// Its bytes.
    pub fn serialize(&self) -> Vec<u8> {
        self.0.serialize().as_bytes().to_vec()
    }
    /// A shader of it.
    pub fn make_shader(
        &self,
        tx: i32,
        ty: i32,
        mode: i32,
        local_matrix: Option<MatrixArg>,
        tile_rect: Option<RectArg>,
    ) -> Result<SkShader, String> {
        let m = local_matrix.map(|m| m.0);
        let r = tile_rect.map(|r| r.0);
        let tiles = (enums::tile(tx)?, enums::tile(ty)?);
        Ok(self.0.to_shader(tiles, enums::filter(mode)?, m.as_ref(), r.as_ref()).into())
    }
}

#[class]
impl SkTypeface {
    /// The typeface of a font file's bytes.
    pub fn from_data(data: &SkData) -> Option<Instance<SkTypeface>> {
        font_mgr().new_from_data(data.0.clone(), None).map(|t| Instance::new(t.into()))
    }
    /// The system's default typeface, so styled.
    pub fn system(weight: Option<i32>, width: Option<i32>, slant: Option<i32>) -> Option<Instance<SkTypeface>> {
        default_typeface(font_style(weight, width, slant)).map(|t| Instance::new(t.into()))
    }
    /// The glyph of each character of `text`.
    pub fn get_glyph_i_ds(&self, text: String) -> Vec<u32> {
        Font::new(self.0.clone(), None).str_to_glyphs_vec(text).into_iter().map(u32::from).collect()
    }
}

/// The system's fonts (a manager costs a scan of them: one per thread).
pub(crate) fn font_mgr() -> FontMgr {
    thread_local!(static MGR: FontMgr = FontMgr::new());
    MGR.with(FontMgr::clone)
}

pub(crate) fn font_style(weight: Option<i32>, width: Option<i32>, slant: Option<i32>) -> FontStyle {
    let slant = match slant {
        Some(1) => font_style::Slant::Italic,
        Some(2) => font_style::Slant::Oblique,
        _ => font_style::Slant::Upright,
    };
    FontStyle::new(weight.unwrap_or(400).into(), width.unwrap_or(5).into(), slant)
}

#[class]
impl SkFontMgr {
    /// The system's fonts.
    pub fn system() -> SkFontMgr {
        font_mgr().into()
    }
    /// How many families there are.
    pub fn count_families(&self) -> usize {
        self.0.count_families()
    }
    /// The name of family `index`.
    pub fn get_family_name(&self, index: usize) -> Result<String, String> {
        if index >= self.0.count_families() {
            return Err(format!("there is no family {index}"));
        }
        Ok(self.0.family_name(index))
    }
    /// A typeface of a family.
    pub fn match_family_style_numbers(
        &self,
        name: String,
        weight: Option<i32>,
        width: Option<i32>,
        slant: Option<i32>,
    ) -> Option<Instance<SkTypeface>> {
        self.0.match_family_style(name, font_style(weight, width, slant)).map(|t| Instance::new(t.into()))
    }
}

/// The font's typeface when none is asked for.
pub(crate) fn default_typeface(style: FontStyle) -> Option<Typeface> {
    font_mgr().legacy_make_typeface(None::<&str>, style)
}

/// A font's metrics.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct FontMetricsOut {
    /// Above the baseline (negative).
    pub ascent: f32,
    /// Below the baseline.
    pub descent: f32,
    /// Between lines.
    pub leading: f32,
    /// What holds every glyph.
    pub bounds: RectOut,
}

fn glyphs(ids: &Floats) -> Vec<GlyphId> {
    ids.0.iter().map(|g| *g as GlyphId).collect()
}

#[class]
impl SkFont {
    /// A typeface (the system's default) at a size (14).
    pub fn new(typeface: Option<Instance<SkTypeface>>, size: Option<f32>) -> Self {
        let face = inner(&typeface, |t| &t.0).or_else(|| default_typeface(FontStyle::normal()));
        let mut font = face.map_or_else(Font::default, |f| Font::new(f, None));
        font.set_size(size.unwrap_or(14.0));
        font.set_edging(font::Edging::AntiAlias);
        font.set_subpixel(true);
        Self(font)
    }
    /// The bounds of `text`.
    pub fn measure_text(&self, text: String, paint: Option<Instance<SkPaint>>) -> RectOut {
        let paint = inner(&paint, |p| &p.0);
        self.0.measure_str(text, paint.as_ref()).1.into()
    }
    /// How far `text` advances.
    pub fn get_text_width(&self, text: String, paint: Option<Instance<SkPaint>>) -> f32 {
        let paint = inner(&paint, |p| &p.0);
        self.0.measure_str(text, paint.as_ref()).0
    }
    /// The glyph of each character.
    pub fn get_glyph_i_ds(&self, text: String) -> Vec<u32> {
        self.0.str_to_glyphs_vec(text).into_iter().map(u32::from).collect()
    }
    /// How far each glyph advances.
    pub fn get_glyph_widths(&self, ids: Floats) -> Vec<f32> {
        let ids = glyphs(&ids);
        let mut out = vec![0.0; ids.len()];
        self.0.get_widths(&ids, &mut out);
        out
    }
    /// Its metrics.
    pub fn get_metrics(&self) -> FontMetricsOut {
        let m = self.0.metrics().1;
        FontMetricsOut {
            ascent: m.ascent,
            descent: m.descent,
            leading: m.leading,
            bounds: skia_safe::Rect::new(m.x_min, m.top, m.x_max, m.bottom).into(),
        }
    }
    /// Its size.
    pub fn get_size(&self) -> f32 {
        self.0.size()
    }
    /// Its horizontal scale.
    pub fn get_scale_x(&self) -> f32 {
        self.0.scale_x()
    }
    /// Its slant.
    pub fn get_skew_x(&self) -> f32 {
        self.0.skew_x()
    }
    /// Whether it is made bolder.
    pub fn is_embolden(&self) -> bool {
        self.0.is_embolden()
    }
    /// Its typeface.
    pub fn get_typeface(&self) -> SkTypeface {
        self.0.typeface().into()
    }
    /// Sets the size.
    pub fn set_size(&mut self, size: f32) {
        self.0.set_size(size);
    }
    /// Sets the horizontal scale.
    pub fn set_scale_x(&mut self, x: f32) {
        self.0.set_scale_x(x);
    }
    /// Sets the slant.
    pub fn set_skew_x(&mut self, x: f32) {
        self.0.set_skew_x(x);
    }
    /// Makes it bolder.
    pub fn set_embolden(&mut self, on: bool) {
        self.0.set_embolden(on);
    }
    /// Places glyphs between pixels.
    pub fn set_subpixel(&mut self, on: bool) {
        self.0.set_subpixel(on);
    }
    /// Scales metrics linearly.
    pub fn set_linear_metrics(&mut self, on: bool) {
        self.0.set_linear_metrics(on);
    }
    /// Uses bitmaps in the font.
    pub fn set_embedded_bitmaps(&mut self, on: bool) {
        self.0.set_embedded_bitmaps(on);
    }
    /// How edges are drawn.
    pub fn set_edging(&mut self, edging: i32) {
        self.0.set_edging(match edging {
            0 => font::Edging::Alias,
            2 => font::Edging::SubpixelAntiAlias,
            _ => font::Edging::AntiAlias,
        });
    }
    /// How outlines fit pixels.
    pub fn set_hinting(&mut self, hinting: i32) {
        self.0.set_hinting(match hinting {
            0 => FontHinting::None,
            1 => FontHinting::Slight,
            3 => FontHinting::Full,
            _ => FontHinting::Normal,
        });
    }
    /// Sets the typeface.
    pub fn set_typeface(&mut self, typeface: Option<Instance<SkTypeface>>) {
        if let Some(face) = inner(&typeface, |t| &t.0).or_else(|| default_typeface(FontStyle::normal())) {
            self.0.set_typeface(face);
        }
    }
}

#[class]
impl SkTextBlob {
    /// A line of text.
    pub fn from_text(text: String, font: &SkFont) -> Result<SkTextBlob, String> {
        TextBlob::from_str(text, &font.0).map(Into::into).ok_or_else(bad("text blob"))
    }
    /// Glyphs, one after the other.
    pub fn from_glyphs(ids: Floats, font: &SkFont) -> Result<SkTextBlob, String> {
        TextBlob::from_text(&glyphs(&ids)[..], &font.0).map(Into::into).ok_or_else(bad("text blob"))
    }
    /// Characters, each with its transform.
    pub fn from_rs_xform(text: String, transforms: Transforms, font: &SkFont) -> Result<SkTextBlob, String> {
        if font.0.count_str(&text) != transforms.0.len() {
            return Err("a text blob needs one transform per glyph".into());
        }
        TextBlob::from_rsxform(text.as_str(), &transforms.0, &font.0).map(Into::into).ok_or_else(bad("text blob"))
    }
    /// Glyphs, each with its transform.
    pub fn from_rs_xform_glyphs(ids: Floats, transforms: Transforms, font: &SkFont) -> Result<SkTextBlob, String> {
        if ids.0.len() != transforms.0.len() {
            return Err("a text blob needs one transform per glyph".into());
        }
        TextBlob::from_rsxform(&glyphs(&ids)[..], &transforms.0, &font.0).map(Into::into).ok_or_else(bad("text blob"))
    }
}

pub(crate) fn vertices(
    mode: VertexMode,
    positions: &[Point],
    textures: Option<&[Point]>,
    colors: Option<&[Color4f]>,
    indices: Option<&[f32]>,
) -> Result<Vertices, String> {
    let n = positions.len();
    if n == 0 {
        return Err("vertices need positions".into());
    }
    if textures.is_some_and(|t| t.len() != n) || colors.is_some_and(|c| c.len() != n) {
        return Err("vertices need as many textures and colors as positions".into());
    }
    if indices.is_some_and(|i| i.iter().any(|i| *i < 0.0 || *i as usize >= n)) {
        return Err("a vertex index is out of range".into());
    }
    if n > usize::from(u16::MAX) && indices.is_some() {
        return Err("indexed vertices are 65535 at most".into());
    }
    if let (VertexMode::TriangleFan, Some(fan)) = (mode, indices) {
        // The builder keeps a fan's indices elsewhere: the triangles it stands for.
        let triangles: Vec<f32> = fan.windows(2).skip(1).flat_map(|edge| [fan[0], edge[0], edge[1]]).collect();
        return vertices(VertexMode::Triangles, positions, textures, colors, Some(&triangles));
    }
    // The builder: `Vertices::new_copy` takes no mesh without textures or colors.
    use skia_safe::vertices::BuilderFlags;
    let mut flags = BuilderFlags::empty();
    flags.set(BuilderFlags::HAS_TEX_COORDS, textures.is_some());
    flags.set(BuilderFlags::HAS_COLORS, colors.is_some());
    let mut builder = skia_safe::vertices::Builder::new(mode, n, indices.map_or(0, <[f32]>::len), flags);
    builder.positions().copy_from_slice(positions);
    if let (Some(to), Some(from)) = (builder.tex_coords(), textures) {
        to.copy_from_slice(from);
    }
    if let (Some(to), Some(from)) = (builder.colors(), colors) {
        to.iter_mut().zip(from).for_each(|(to, from)| *to = from.to_color());
    }
    if let (Some(to), Some(from)) = (builder.indices(), indices) {
        to.iter_mut().zip(from).for_each(|(to, from)| *to = *from as u16);
    }
    Ok(builder.detach())
}

#[class]
impl SkVertices {
    /// A mesh.
    pub fn make(
        mode: i32,
        positions: Points,
        textures: Option<Points>,
        colors: Option<Colors>,
        indices: Option<Floats>,
    ) -> Result<SkVertices, String> {
        vertices(
            enums::vertex_mode(mode)?,
            &positions.0,
            textures.as_ref().map(|t| &t.0[..]),
            colors.as_ref().map(|c| &c.0[..]),
            indices.as_ref().map(|i| &i.0[..]),
        )
        .map(Into::into)
    }
}

#[class]
impl SkSvg {
    /// The document of an SVG string (`null`: it cannot be read).
    pub fn from_text(text: String) -> Option<Instance<SkSvg>> {
        svg::Dom::from_str(text, font_mgr()).ok().map(|d| Instance::new(d.into()))
    }
    /// Its width.
    pub fn width(&self) -> f32 {
        self.0.root().intrinsic_size().width
    }
    /// Its height.
    pub fn height(&self) -> f32 {
        self.0.root().intrinsic_size().height
    }
}

/// A SkSL program.
#[class]
pub struct SkRuntimeEffect {
    pub(crate) effect: RuntimeEffect,
    source: String,
}

/// A uniform of a program.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct UniformOut {
    /// Its columns.
    pub columns: i32,
    /// Its rows.
    pub rows: i32,
    /// Its first float.
    pub slot: i32,
    /// Whether its numbers are integers.
    pub is_integer: bool,
}

/// The columns, rows and kind of a uniform's type.
fn shape(ty: runtime_effect::uniform::Type) -> (i32, i32, bool) {
    use runtime_effect::uniform::Type;
    match ty {
        Type::Float => (1, 1, false),
        Type::Float2 => (1, 2, false),
        Type::Float3 => (1, 3, false),
        Type::Float4 => (1, 4, false),
        Type::Float2x2 => (2, 2, false),
        Type::Float3x3 => (3, 3, false),
        Type::Float4x4 => (4, 4, false),
        Type::Int => (1, 1, true),
        Type::Int2 => (1, 2, true),
        Type::Int3 => (1, 3, true),
        Type::Int4 => (1, 4, true),
    }
}

/// Uniforms' numbers as a program's bytes: integers are integers.
pub(crate) fn uniform_data(effect: &RuntimeEffect, floats: &[f32]) -> Result<Data, String> {
    let size = effect.uniform_size();
    if floats.len() * 4 != size {
        return Err(format!("the program's uniforms are {} numbers, not {}", size / 4, floats.len()));
    }
    let mut bytes = Vec::with_capacity(size);
    let mut integer = vec![false; floats.len()];
    for u in effect.uniforms() {
        if shape(u.ty()).2 {
            integer[u.offset() / 4..(u.offset() + u.size_in_bytes()) / 4].fill(true);
        }
    }
    for (f, int) in floats.iter().zip(integer) {
        bytes.extend(if int { (*f as i32).to_ne_bytes() } else { f.to_ne_bytes() });
    }
    Ok(Data::new_copy(&bytes))
}

#[class]
impl SkRuntimeEffect {
    /// Compiles a SkSL shader; throws what the compiler says.
    pub fn make(sksl: String) -> Result<SkRuntimeEffect, String> {
        let effect = RuntimeEffect::make_for_shader(&sksl, None)?;
        Ok(SkRuntimeEffect { effect, source: sksl })
    }
    /// A shader of it.
    pub fn make_shader(&self, uniforms: Floats, local_matrix: Option<MatrixArg>) -> Result<SkShader, String> {
        self.shader(&uniforms.0, &[], local_matrix.map(|m| m.0).as_ref()).map(Into::into)
    }
    /// A shader of it and of `children`.
    pub fn make_shader_with_children(
        &self,
        uniforms: Floats,
        children: Shaders,
        local_matrix: Option<MatrixArg>,
    ) -> Result<SkShader, String> {
        self.shader(&uniforms.0, &children.0, local_matrix.map(|m| m.0).as_ref()).map(Into::into)
    }
    /// How many uniforms it has.
    pub fn get_uniform_count(&self) -> usize {
        self.effect.uniforms().len()
    }
    /// How many numbers they are.
    pub fn get_uniform_float_count(&self) -> usize {
        self.effect.uniform_size() / 4
    }
    /// The name of uniform `index`.
    pub fn get_uniform_name(&self, index: usize) -> Result<String, String> {
        self.effect.uniforms().get(index).map(|u| u.name().to_owned()).ok_or_else(|| format!("no uniform {index}"))
    }
    /// Uniform `index`.
    pub fn get_uniform(&self, index: usize) -> Result<UniformOut, String> {
        let uniforms = self.effect.uniforms();
        let u = uniforms.get(index).ok_or_else(|| format!("no uniform {index}"))?;
        let (columns, rows, is_integer) = shape(u.ty());
        Ok(UniformOut { columns, rows, slot: (u.offset() / 4) as i32, is_integer })
    }
    /// Its SkSL.
    pub fn source(&self) -> String {
        self.source.clone()
    }
}

impl SkRuntimeEffect {
    pub(crate) fn shader(
        &self,
        uniforms: &[f32],
        children: &[Shader],
        matrix: Option<&Matrix>,
    ) -> Result<Shader, String> {
        let wanted = self.effect.children().len();
        if children.len() != wanted {
            return Err(format!("the program takes {wanted} child shaders, not {}", children.len()));
        }
        let children: Vec<_> = children.iter().cloned().map(runtime_effect::ChildPtr::Shader).collect();
        let data = uniform_data(&self.effect, uniforms)?;
        self.effect.make_shader(data, &children, matrix).ok_or_else(bad("shader"))
    }
}

wrapper!(/** Sets a program's uniforms by name. */ SkRuntimeShaderBuilder(skia_safe::runtime_effect::RuntimeShaderBuilder));

#[class]
impl SkRuntimeShaderBuilder {
    /// Of `effect`.
    pub fn new(effect: &SkRuntimeEffect) -> Self {
        Self(skia_safe::runtime_effect::RuntimeShaderBuilder::new(effect.effect.clone()))
    }
    /// Sets uniform `name`.
    pub fn set_uniform(&mut self, name: String, value: Floats) -> Result<(), String> {
        set_uniform(&mut self.0, &name, &value.0)
    }
}

pub(crate) fn set_uniform(
    builder: &mut skia_safe::runtime_effect::RuntimeShaderBuilder,
    name: &str,
    value: &[f32],
) -> Result<(), String> {
    builder.set_uniform_float(name, value).map_err(|e| format!("uniform `{name}`: {e:?}"))
}

/// Shaders, in an array.
#[derive(Debug, Clone)]
pub struct Shaders(pub(crate) Vec<Shader>);

impl JsValue for Shaders {
    fn ts_type(_types: &mut nsh_core::element::TsTypes) -> String {
        "readonly SkShader[]".into()
    }
    fn from_js(value: JsIn<'_>) -> Result<Self, nsh_core::element::PropError> {
        let len = value.array_len()?.ok_or_else(|| value.expected("an array of shaders"))?;
        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            let item = value.index(i)?;
            let shader = get::<SkShader>(item.get()).ok_or_else(|| item.get().expected("a shader"))?;
            out.push(shader.with(|s| s.0.clone()));
        }
        Ok(Self(out))
    }
    fn to_js<'a>(
        &self,
        host: &'a dyn nsh_core::element::JsHost,
    ) -> Result<nsh_core::element::JsOwned<'a>, nsh_core::element::PropError> {
        Ok(nsh_core::element::JsOwned::new(host, host.null()))
    }
}

pub use crate::path::SkPath;
