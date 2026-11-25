//! The value of a scene node's prop, converted once from JS.

use nsh_core::{
    element::{JsClass, JsHost, JsIn, JsKind, JsOwned, PropError, TsTypes},
    prelude::*,
};
use skia_safe::{Color4f, Matrix, Point, RRect, Rect};

use crate::{
    args,
    objects::{SkFont, SkImage, SkImageFilter, SkMatrix, SkPath, SkPicture, SkRuntimeEffect, SkSvg, SkTextBlob, get},
    paint::SkPaint,
    paragraph::SkParagraph,
};

/// The numbers of an array of numbers or a typed array (`None`: neither).
pub(crate) fn floats(value: JsIn<'_>) -> Result<Option<Vec<f32>>, PropError> {
    match value.class() {
        Some(JsClass::Array) => {
            let Some(items) = value.primitives()? else { return Ok(None) };
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    nsh_core::element::JsPrimitive::Number(n) => out.push(n as f32),
                    _ => return Ok(None),
                }
            }
            Ok(Some(out))
        }
        Some(JsClass::View(name)) => {
            let Some(bytes) = value.as_buffer() else { return Ok(None) };
            Ok(match name {
                "Float32Array" => Some(bytes.as_chunks::<4>().0.iter().map(|b| f32::from_ne_bytes(*b)).collect()),
                "Float64Array" => {
                    Some(bytes.as_chunks::<8>().0.iter().map(|b| f64::from_ne_bytes(*b) as f32).collect())
                }
                "Uint16Array" => Some(bytes.as_chunks::<2>().0.iter().map(|b| u16::from_ne_bytes(*b) as f32).collect()),
                "Uint32Array" => Some(bytes.as_chunks::<4>().0.iter().map(|b| u32::from_ne_bytes(*b) as f32).collect()),
                "Int32Array" => Some(bytes.as_chunks::<4>().0.iter().map(|b| i32::from_ne_bytes(*b) as f32).collect()),
                _ => None,
            })
        }
        _ => Ok(None),
    }
}

/// An image's sampling: `{ B, C }` or `{ filter, mipmap }`, linear when left out.
pub(crate) fn sampling_of(value: Option<&Value>) -> Result<skia_safe::SamplingOptions, String> {
    let Some(s) = value else { return args::enums::sampling(None, None) };
    if let (Some(b), Some(c)) = (s.num_field("B"), s.num_field("C")) {
        return Ok(skia_safe::CubicResampler { b, c }.into());
    }
    let int = |key| s.num_field(key).map(|n| n as i32);
    args::enums::sampling(int("filter"), int("mipmap"))
}

/// A prop's value.
#[derive(Debug, Clone)]
pub(crate) enum Value {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    Floats(Vec<f32>),
    List(Vec<Value>),
    Fields(Vec<(String, Value)>),
    Path(Instance<SkPath>),
    /// An SVG path string, parsed.
    Svg(skia_safe::Path),
    Color(Color4f),
    Colors(Vec<Color4f>),
    Points(Vec<Point>),
    Image(Instance<SkImage>),
    Font(Instance<SkFont>),
    Paint(Instance<SkPaint>),
    Picture(Instance<SkPicture>),
    SvgDom(Instance<SkSvg>),
    Blob(Instance<SkTextBlob>),
    Effect(Instance<SkRuntimeEffect>),
    ImageFilter(Instance<SkImageFilter>),
    Matrix(Instance<SkMatrix>),
    Paragraph(Instance<SkParagraph>),
}

impl PartialEq for Value {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl JsValue for Value {
    fn ts_type(_types: &mut TsTypes) -> String {
        "unknown".into()
    }

    fn from_js(value: JsIn<'_>) -> Result<Self, PropError> {
        match value.kind() {
            JsKind::Undefined | JsKind::Null => Ok(Value::Null),
            JsKind::Bool => Ok(Value::Bool(value.as_bool().unwrap_or_default())),
            JsKind::Number => Ok(Value::Number(value.as_number().unwrap_or_default())),
            JsKind::String => Ok(Value::Text(value.as_string().unwrap_or_default())),
            JsKind::Array => {
                if let Some(f) = floats(value)? {
                    return Ok(Value::Floats(f));
                }
                let len = value.array_len()?.unwrap_or(0);
                (0..len).map(|i| Value::from_js(value.index(i)?.get())).collect::<Result<_, _>>().map(Value::List)
            }
            JsKind::Object => {
                if let Some(f) = floats(value)? {
                    return Ok(Value::Floats(f));
                }
                if value.host().instance(value.raw()).is_some() {
                    return instance(value).ok_or_else(|| value.expected("a Skia object"));
                }
                let mut fields = Vec::new();
                for key in value.keys()? {
                    let field = Value::from_js(value.field(&key)?.get())?;
                    fields.push((key, field));
                }
                Ok(Value::Fields(fields))
            }
            _ => Err(value.expected("a prop's value")),
        }
    }

    fn to_js<'a>(&self, host: &'a dyn JsHost) -> Result<JsOwned<'a>, PropError> {
        Ok(JsOwned::new(host, host.null()))
    }
}

/// The Skia object a JS object is, whichever it is.
fn instance(v: JsIn<'_>) -> Option<Value> {
    get(v)
        .map(Value::Path)
        .or_else(|| get(v).map(Value::Image))
        .or_else(|| get(v).map(Value::Font))
        .or_else(|| get(v).map(Value::Paint))
        .or_else(|| get(v).map(Value::Matrix))
        .or_else(|| get(v).map(Value::Effect))
        .or_else(|| get(v).map(Value::Picture))
        .or_else(|| get(v).map(Value::Blob))
        .or_else(|| get(v).map(Value::Paragraph))
        .or_else(|| get(v).map(Value::SvgDom))
        .or_else(|| get(v).map(Value::ImageFilter))
}

impl Value {
    /// What `name`'s value is worth converting once for: colors and paths.
    pub(crate) fn prepared(self, name: &str) -> Result<Self, String> {
        let bad = |what: &str| format!("`{name}`: not {what}");
        Ok(match (name, self) {
            (_, Value::Null) => Value::Null,
            ("color", v) => Value::Color(v.color().ok_or_else(|| bad("a color"))?),
            ("colors", Value::List(list)) => Value::Colors(
                list.iter().map(Value::color).collect::<Option<_>>().ok_or_else(|| bad("a list of colors"))?,
            ),
            ("colors", Value::Floats(list)) => {
                Value::Colors(list.iter().map(|n| args::argb(*n as i64 as u32)).collect())
            }
            ("path" | "clip", Value::Text(text)) => {
                Value::Svg(skia_safe::utils::parse_path::from_svg(&text).ok_or_else(|| bad("an SVG path"))?)
            }
            ("points" | "vertices" | "textures", v @ (Value::Floats(_) | Value::List(_))) => {
                Value::Points(v.points().ok_or_else(|| bad("a list of points"))?)
            }
            (_, v) => v,
        })
    }

    pub(crate) fn number(&self) -> Option<f32> {
        match self {
            Value::Number(n) => Some(*n as f32),
            _ => None,
        }
    }

    pub(crate) fn text(&self) -> Option<&str> {
        match self {
            Value::Text(t) => Some(t),
            _ => None,
        }
    }

    pub(crate) fn field(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Fields(f) => f.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub(crate) fn num_field(&self, key: &str) -> Option<f32> {
        self.field(key)?.number()
    }

    pub(crate) fn color(&self) -> Option<Color4f> {
        match self {
            Value::Color(c) => Some(*c),
            Value::Text(t) => args::parse_color(t),
            Value::Number(n) => Some(args::argb(*n as i64 as u32)),
            Value::Floats(f) => match f[..] {
                [r, g, b] => Some(Color4f::new(r, g, b, 1.0)),
                [r, g, b, a] => Some(Color4f::new(r, g, b, a)),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn point(&self) -> Option<Point> {
        Some(Point::new(self.num_field("x")?, self.num_field("y")?))
    }

    pub(crate) fn points(&self) -> Option<Vec<Point>> {
        match self {
            Value::Points(p) => Some(p.clone()),
            Value::Floats(f) => Some(f.as_chunks::<2>().0.iter().map(|p| Point::new(p[0], p[1])).collect()),
            Value::List(l) => l.iter().map(Value::point).collect(),
            _ => None,
        }
    }

    /// A number (both) or a point.
    pub(crate) fn radius(&self) -> Option<Point> {
        self.number().map(|n| Point::new(n, n)).or_else(|| self.point())
    }

    pub(crate) fn rect(&self) -> Option<Rect> {
        Some(Rect::from_xywh(
            self.num_field("x").unwrap_or(0.0),
            self.num_field("y").unwrap_or(0.0),
            self.num_field("width")?,
            self.num_field("height")?,
        ))
    }

    pub(crate) fn is_rrect(&self) -> bool {
        self.field("rect").is_some()
    }

    pub(crate) fn rrect(&self) -> Option<RRect> {
        let Some(inner) = self.field("rect") else { return self.rect().map(RRect::new_rect) };
        let rect = inner.rect()?;
        if self.field("topLeft").is_some() {
            let corner = |key| self.field(key).and_then(Value::point);
            let radii = [corner("topLeft")?, corner("topRight")?, corner("bottomRight")?, corner("bottomLeft")?];
            return Some(RRect::new_rect_radii(rect, &radii));
        }
        let (rx, ry) = (self.num_field("rx").unwrap_or(0.0), self.num_field("ry").unwrap_or(0.0));
        Some(RRect::new_rect_xy(rect, rx, ry))
    }

    pub(crate) fn path(&self) -> Option<skia_safe::Path> {
        match self {
            Value::Path(p) => Some(p.with(|p| p.0.clone())),
            Value::Svg(p) => Some(p.clone()),
            _ => None,
        }
    }

    pub(crate) fn matrix(&self) -> Option<Matrix> {
        match self {
            Value::Matrix(m) => Some(m.with(|m| m.0)),
            Value::Floats(f) => args::matrix_of(f),
            _ => None,
        }
    }

    /// Every number in it, in order (a uniform's floats).
    pub(crate) fn flatten(&self, out: &mut Vec<f32>) {
        match self {
            Value::Number(n) => out.push(*n as f32),
            Value::Floats(f) => out.extend_from_slice(f),
            Value::List(l) => l.iter().for_each(|v| v.flatten(out)),
            Value::Fields(_) => {
                if let Some(p) = self.point() {
                    out.extend([p.x, p.y]);
                }
            }
            Value::Color(c) => out.extend([c.r, c.g, c.b, c.a]),
            _ => {}
        }
    }
}
