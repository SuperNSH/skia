//! What the API's functions take and give: points, rectangles, colors,
//! matrices, as the plain objects and arrays react-native-skia uses.

use nsh_core::{
    element::{JsHost, JsIn, JsKind, JsOwned, PropError, TsTypes},
    prelude::*,
};
use skia_safe::{Color4f, Matrix, Point, RRect, RSXform, Rect, Vector};

use crate::{
    objects::{SkMatrix, get},
    value::floats,
};

pub(crate) fn num(value: JsIn<'_>, key: &str) -> Result<f32, PropError> {
    let field = value.field(key)?;
    let field = field.get();
    field.as_number().map(|n| n as f32).ok_or_else(|| field.expected(&format!("a number for `{key}`")))
}

fn num_or(value: JsIn<'_>, key: &str, default: f32) -> Result<f32, PropError> {
    Ok(value.field(key)?.get().as_number().map_or(default, |n| n as f32))
}

/// A type that only comes in.
macro_rules! input {
    ($(#[$doc:meta])* $name:ident($inner:ty), $ts:expr, $read:expr) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name(pub $inner);
        impl JsValue for $name {
            fn ts_type(_types: &mut TsTypes) -> String {
                $ts.into()
            }
            fn from_js(value: JsIn<'_>) -> Result<Self, PropError> {
                let read: fn(JsIn<'_>) -> Result<$inner, PropError> = $read;
                read(value).map($name)
            }
            fn to_js<'a>(&self, host: &'a dyn JsHost) -> Result<JsOwned<'a>, PropError> {
                Ok(JsOwned::new(host, host.null()))
            }
        }
    };
}

pub(crate) fn point(value: JsIn<'_>) -> Result<Point, PropError> {
    Ok(Point::new(num(value, "x")?, num(value, "y")?))
}

pub(crate) fn rect(value: JsIn<'_>) -> Result<Rect, PropError> {
    if value.kind() != JsKind::Object {
        return Err(value.expected("a rectangle ({ x, y, width, height })"));
    }
    Ok(Rect::from_xywh(num_or(value, "x", 0.0)?, num_or(value, "y", 0.0)?, num(value, "width")?, num(value, "height")?))
}

pub(crate) fn rrect(value: JsIn<'_>) -> Result<RRect, PropError> {
    let inner = value.field("rect")?;
    if inner.get().kind() != JsKind::Object {
        return Ok(RRect::new_rect(rect(value)?));
    }
    let bounds = rect(inner.get())?;
    let corner = |key: &str| -> Result<Vector, PropError> { point(value.field(key)?.get()) };
    if value.field("topLeft")?.get().kind() == JsKind::Object {
        let radii = [corner("topLeft")?, corner("topRight")?, corner("bottomRight")?, corner("bottomLeft")?];
        return Ok(RRect::new_rect_radii(bounds, &radii));
    }
    Ok(RRect::new_rect_xy(bounds, num_or(value, "rx", 0.0)?, num_or(value, "ry", 0.0)?))
}

/// A matrix of 9 numbers, or the 2D part of 16 (row-major, as
/// react-native-skia's are).
pub(crate) fn matrix_of(m: &[f32]) -> Option<Matrix> {
    let m = match m.len() {
        9 => [m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8]],
        16 => [m[0], m[1], m[3], m[4], m[5], m[7], m[12], m[13], m[15]],
        _ => return None,
    };
    Some(Matrix::new_all(m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8]))
}

pub(crate) fn matrix(value: JsIn<'_>) -> Result<Matrix, PropError> {
    if let Some(m) = get::<SkMatrix>(value) {
        return Ok(m.with(|m| m.0));
    }
    floats(value)?.as_deref().and_then(matrix_of).ok_or_else(|| value.expected("a matrix (SkMatrix, 9 or 16 numbers)"))
}

/// A CSS color, as Skia's unpremultiplied floats.
pub(crate) fn parse_color(text: &str) -> Option<Color4f> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("transparent") {
        return Some(Color4f::new(0.0, 0.0, 0.0, 0.0));
    }
    let rgba = nsh_core::element::Color::parse(text)?.0;
    let part = |shift: u32| ((rgba >> shift) & 0xff) as f32 / 255.0;
    Some(Color4f::new(part(24), part(16), part(8), part(0)))
}

/// `0xAARRGGBB`.
pub(crate) fn argb(n: u32) -> Color4f {
    Color4f::from(skia_safe::Color::from(n))
}

pub(crate) fn color(value: JsIn<'_>) -> Result<Color4f, PropError> {
    if let Some(n) = value.as_number() {
        // As `>>> 0` does: a negative int is a color with a high alpha.
        return Ok(argb(n as i64 as u32));
    }
    if let Some(text) = value.as_string() {
        return parse_color(&text).ok_or_else(|| value.expected("a color"));
    }
    match floats(value)?.as_deref() {
        Some([r, g, b]) => Ok(Color4f::new(*r, *g, *b, 1.0)),
        Some([r, g, b, a]) => Ok(Color4f::new(*r, *g, *b, *a)),
        _ => Err(value.expected("a color (a CSS string, 0xAARRGGBB or [r, g, b, a])")),
    }
}

fn list<T>(value: JsIn<'_>, read: fn(JsIn<'_>) -> Result<T, PropError>) -> Result<Vec<T>, PropError> {
    let len = value.array_len()?.ok_or_else(|| value.expected("an array"))?;
    (0..len).map(|i| read(value.index(i)?.get())).collect()
}

/// Points, or a Float32Array of x, y pairs.
pub(crate) fn points(value: JsIn<'_>) -> Result<Vec<Point>, PropError> {
    if value.array_len()?.is_none()
        && let Some(f) = floats(value)?
    {
        return Ok(f.as_chunks::<2>().0.iter().map(|p| Point::new(p[0], p[1])).collect());
    }
    list(value, point)
}

fn rsxform(value: JsIn<'_>) -> Result<RSXform, PropError> {
    Ok(RSXform::new(num(value, "scos")?, num(value, "ssin")?, (num(value, "tx")?, num(value, "ty")?)))
}

input!(/** A point. */ Pt(Point), "{ readonly x: number; readonly y: number }", point);
input!(/** A rectangle. */ RectArg(Rect), "{ readonly x: number; readonly y: number; readonly width: number; readonly height: number }", rect);
input!(/** A rounded rectangle, or a rectangle. */ RRectArg(RRect), "{ readonly rect: { readonly x: number; readonly y: number; readonly width: number; readonly height: number }; readonly rx?: number; readonly ry?: number } | { readonly x: number; readonly y: number; readonly width: number; readonly height: number }", rrect);
input!(/** A color. */ ColorArg(Color4f), "string | number | readonly number[] | Float32Array", color);
input!(/** Colors. */ Colors(Vec<Color4f>), "readonly (string | number | readonly number[] | Float32Array)[]", |v| list(v, color));
input!(/** Points. */ Points(Vec<Point>), "readonly { readonly x: number; readonly y: number }[] | Float32Array", points);
input!(/** Rectangles. */ Rects(Vec<Rect>), "readonly { readonly x: number; readonly y: number; readonly width: number; readonly height: number }[]", |v| list(v, rect));
input!(/** Rotations, scales and translations. */ Transforms(Vec<RSXform>), "readonly { readonly scos: number; readonly ssin: number; readonly tx: number; readonly ty: number }[]", |v| list(v, rsxform));
input!(/** A matrix. */ MatrixArg(Matrix), "SkMatrix | readonly number[] | Float32Array", matrix);
input!(/** Numbers. */ Floats(Vec<f32>), "readonly number[] | Float32Array", |v| {
    floats(v)?.ok_or_else(|| v.expected("numbers"))
});
input!(/** Bytes. */ Bytes(Vec<u8>), "ArrayBuffer | ArrayBufferView", |v| {
    v.as_buffer().ok_or_else(|| v.expected("an ArrayBuffer or a typed array"))
});

/// A point, given.
#[derive(Debug, Clone, Copy, PartialEq, JsObject)]
pub struct PointOut {
    /// Its x.
    pub x: f32,
    /// Its y.
    pub y: f32,
}

impl From<Point> for PointOut {
    fn from(p: Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

/// A rectangle, given.
#[derive(Debug, Clone, Copy, PartialEq, JsObject)]
pub struct RectOut {
    /// Its left.
    pub x: f32,
    /// Its top.
    pub y: f32,
    /// Its width.
    pub width: f32,
    /// Its height.
    pub height: f32,
}

impl From<Rect> for RectOut {
    fn from(r: Rect) -> Self {
        Self { x: r.left, y: r.top, width: r.width(), height: r.height() }
    }
}

// --- Enums, by react-native-skia's numbers ----------------------------------

pub(crate) mod enums {
    use skia_safe::{
        BlendMode, BlurStyle, ClipOp, ColorChannel, EncodedImageFormat, FilterMode, MipmapMode, PathFillType, PathOp,
        SamplingOptions, TileMode, canvas::PointMode, paint, path_1d_path_effect, vertices::VertexMode,
    };

    macro_rules! by_number {
        ($($name:ident -> $ty:ty { $first:expr $(, $rest:expr)* $(,)? })*) => {$(
            pub(crate) fn $name(n: i32) -> Result<$ty, String> {
                const ALL: &[$ty] = &[$first $(, $rest)*];
                usize::try_from(n).ok().and_then(|n| ALL.get(n)).copied()
                    .ok_or_else(|| format!(concat!("not a ", stringify!($ty), ": {}"), n))
            }
        )*};
    }

    by_number! {
        tile -> TileMode { TileMode::Clamp, TileMode::Repeat, TileMode::Mirror, TileMode::Decal }
        blur_style -> BlurStyle { BlurStyle::Normal, BlurStyle::Solid, BlurStyle::Outer, BlurStyle::Inner }
        clip_op -> ClipOp { ClipOp::Difference, ClipOp::Intersect }
        channel -> ColorChannel { ColorChannel::R, ColorChannel::G, ColorChannel::B, ColorChannel::A }
        filter -> FilterMode { FilterMode::Nearest, FilterMode::Linear }
        mipmap -> MipmapMode { MipmapMode::None, MipmapMode::Nearest, MipmapMode::Linear }
        fill_type -> PathFillType { PathFillType::Winding, PathFillType::EvenOdd, PathFillType::InverseWinding, PathFillType::InverseEvenOdd }
        path_op -> PathOp { PathOp::Difference, PathOp::Intersect, PathOp::Union, PathOp::XOR, PathOp::ReverseDifference }
        point_mode -> PointMode { PointMode::Points, PointMode::Lines, PointMode::Polygon }
        vertex_mode -> VertexMode { VertexMode::Triangles, VertexMode::TriangleStrip, VertexMode::TriangleFan }
        paint_style -> paint::Style { paint::Style::Fill, paint::Style::Stroke }
        cap -> paint::Cap { paint::Cap::Butt, paint::Cap::Round, paint::Cap::Square }
        join -> paint::Join { paint::Join::Miter, paint::Join::Round, paint::Join::Bevel }
        path_1d -> path_1d_path_effect::Style { path_1d_path_effect::Style::Translate, path_1d_path_effect::Style::Rotate, path_1d_path_effect::Style::Morph }
        blend -> BlendMode {
            BlendMode::Clear, BlendMode::Src, BlendMode::Dst, BlendMode::SrcOver, BlendMode::DstOver, BlendMode::SrcIn,
            BlendMode::DstIn, BlendMode::SrcOut, BlendMode::DstOut, BlendMode::SrcATop, BlendMode::DstATop,
            BlendMode::Xor, BlendMode::Plus, BlendMode::Modulate, BlendMode::Screen, BlendMode::Overlay,
            BlendMode::Darken, BlendMode::Lighten, BlendMode::ColorDodge, BlendMode::ColorBurn, BlendMode::HardLight,
            BlendMode::SoftLight, BlendMode::Difference, BlendMode::Exclusion, BlendMode::Multiply, BlendMode::Hue,
            BlendMode::Saturation, BlendMode::Color, BlendMode::Luminosity,
        }
    }

    pub(crate) fn format(n: Option<i32>) -> EncodedImageFormat {
        match n {
            Some(3) => EncodedImageFormat::JPEG,
            Some(6) => EncodedImageFormat::WEBP,
            _ => EncodedImageFormat::PNG,
        }
    }

    pub(crate) fn sampling(fm: Option<i32>, mm: Option<i32>) -> Result<SamplingOptions, String> {
        Ok(SamplingOptions::new(filter(fm.unwrap_or(1))?, mipmap(mm.unwrap_or(0))?))
    }
}
