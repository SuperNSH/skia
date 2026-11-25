//! The declarative scene: a tree of nodes kept in Rust, drawn as
//! react-native-skia's recorder and player draw it (src/sksg/Recorder at
//! commit 6ef585b: Visitor.ts, Player.ts, DrawingContext.ts, commands/*).

use std::{cell::RefCell, rc::Rc};

use nsh_core::{
    element::{JsHost, JsIn, JsOwned, PropError, TsTypes},
    prelude::*,
};
use skia_safe::{
    BlendMode, Canvas, ClipOp, Color4f, ColorFilter, ImageFilter, MaskFilter, Matrix, Paint, PathEffect, Picture,
    Point, RRect, RSXform, Rect, SamplingOptions, Shader, TextBlob,
    canvas::{SaveLayerRec, SrcRectConstraint},
    color_filters, dash_path_effect, gradient, image_filters, path_1d_path_effect, path_2d_path_effect, shaders,
};

use crate::{
    args::enums,
    canvas::{SkCanvas, atlas_on, default_blend, four_colors, raster, record, svg_on},
    objects::{SkImage, SkPicture, SkRuntimeEffect, get, noise, set_uniform, vertices, with_gradient},
    paint::new_paint,
    path::{Stroke, done, stroked, trimmed},
    value::{Value, sampling_of},
};

type Res<T> = Result<T, String>;

macro_rules! kinds {
    ($($group:ident { $($kind:ident)* })*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum Kind { $($($kind,)*)* }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Family { $($group,)* }

        impl Kind {
            fn parse(name: &str) -> Option<Self> {
                Some(match name {
                    $($(stringify!($kind) => Kind::$kind,)*)*
                    _ => return None,
                })
            }
            fn family(self) -> Family {
                match self {
                    $($(Kind::$kind => Family::$group,)*)*
                }
            }
        }
    };
}

kinds! {
    Drawing {
        Group Layer Circle Fill Image Points Path Rect RRect Oval Line Patch Vertices DiffRect Text TextPath TextBlob
        Glyphs Picture ImageSVG Atlas Paragraph Box BoxShadow BackdropFilter
    }
    Paints { Paint }
    Shaders {
        Shader ImageShader ColorShader Turbulence FractalNoise LinearGradient RadialGradient SweepGradient
        TwoPointConicalGradient
    }
    Mixed { Blend }
    MaskFilters { BlurMaskFilter }
    PathEffects {
        DiscretePathEffect DashPathEffect Path1DPathEffect Path2DPathEffect CornerPathEffect SumPathEffect
        Line2DPathEffect
    }
    ColorFilters {
        MatrixColorFilter BlendColorFilter LinearToSRGBGammaColorFilter SRGBToLinearGammaColorFilter LumaColorFilter
        LerpColorFilter
    }
    ImageFilters {
        ImageFilter OffsetImageFilter DisplacementMapImageFilter BlurImageFilter DropShadowImageFilter
        MorphologyImageFilter RuntimeShaderImageFilter
    }
}

pub(crate) struct Node {
    kind: Kind,
    props: Vec<(String, Value)>,
    children: Vec<NodeRc>,
    /// Of a root: what its last drawing could not draw.
    errors: Vec<String>,
}

pub(crate) type NodeRc = Rc<RefCell<Node>>;

/// A node of a scene.
#[class]
pub struct SkNode(NodeRc);

/// Nodes, in an array.
#[derive(Clone)]
pub struct Nodes(Vec<NodeRc>);

impl JsValue for Nodes {
    fn ts_type(_types: &mut TsTypes) -> String {
        "readonly SkNode[]".into()
    }
    fn from_js(value: JsIn<'_>) -> Result<Self, PropError> {
        let len = value.array_len()?.ok_or_else(|| value.expected("an array of nodes"))?;
        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            let item = value.index(i)?;
            let node = get::<SkNode>(item.get()).ok_or_else(|| item.get().expected("a node"))?;
            out.push(node.with(|n| n.0.clone()));
        }
        Ok(Self(out))
    }
    fn to_js<'a>(&self, host: &'a dyn JsHost) -> Result<JsOwned<'a>, PropError> {
        Ok(JsOwned::new(host, host.null()))
    }
}

/// What drawing a scene gave.
fn picture_of(root: &NodeRc, width: f32, height: f32) -> Res<(Picture, Vec<String>)> {
    let mut errors = Vec::new();
    let picture = record(Rect::from_wh(width, height), |canvas| {
        let mut ctx = Ctx { canvas, declarations: Vec::new(), errors: &mut errors };
        let base = Level { paint: new_paint(), opacity: 1.0 };
        ctx.children(root.borrow().children.iter(), &base);
        Ok(())
    })?;
    root.borrow_mut().errors.clone_from(&errors);
    Ok((picture, errors))
}

#[class]
impl SkNode {
    /// A node of a kind (`"Circle"`).
    pub fn new(kind: String) -> Result<SkNode, String> {
        let kind = Kind::parse(&kind).ok_or_else(|| format!("there is no node `{kind}`"))?;
        Ok(SkNode(Rc::new(RefCell::new(Node { kind, props: Vec::new(), children: Vec::new(), errors: Vec::new() }))))
    }
    /// Sets a prop.
    pub fn set(&self, name: String, value: Value) -> Result<(), String> {
        let value = value.prepared(&name)?;
        let mut node = self.0.borrow_mut();
        node.props.retain(|(k, _)| *k != name);
        if !matches!(value, Value::Null) {
            node.props.push((name, value));
        }
        Ok(())
    }
    /// Sets the children.
    pub fn set_children(&self, children: Nodes) {
        self.0.borrow_mut().children = children.0;
    }
    /// Records the children as a picture. Throws what the first node that cannot be drawn says.
    pub fn picture(&self, width: f32, height: f32) -> Result<SkPicture, String> {
        let (picture, errors) = picture_of(&self.0, width, height)?;
        errors.into_iter().next().map_or(Ok(picture.into()), Err)
    }
    /// Draws the children on a canvas. Throws what the first node that cannot be drawn says.
    pub fn draw_on(&self, canvas: Instance<SkCanvas>, width: f32, height: f32) -> Result<(), String> {
        let (picture, errors) = picture_of(&self.0, width, height)?;
        canvas.with(|c| {
            c.draw(|c| {
                c.draw_picture(&picture, None, None);
            })
        })?;
        errors.into_iter().next().map_or(Ok(()), Err)
    }
    /// What could not be drawn the last time.
    #[getter]
    pub fn errors(&self) -> Vec<String> {
        self.0.borrow().errors.clone()
    }
    /// Draws the children into an image of `width * scale` by `height * scale` pixels.
    pub fn snapshot(&self, width: f32, height: f32, scale: f32) -> Result<SkImage, String> {
        let (picture, _) = picture_of(&self.0, width, height)?;
        let mut surface = raster((width * scale).round().max(1.0) as i32, (height * scale).round().max(1.0) as i32)?;
        surface.canvas().scale((scale, scale)).draw_picture(&picture, None, None);
        Ok(surface.image_snapshot().into())
    }
    /// Shows the children in the canvas view `id`; the errors of nodes that cannot be drawn.
    pub fn present(&self, id: u32, width: f32, height: f32) -> Result<Vec<String>, String> {
        let (picture, errors) = picture_of(&self.0, width, height)?;
        crate::view::show(id, picture);
        Ok(errors)
    }
}

impl Node {
    fn get(&self, name: &str) -> Option<&Value> {
        self.props.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }
    fn num(&self, name: &str) -> Option<f32> {
        self.get(name)?.number()
    }
    fn num_or(&self, name: &str, default: f32) -> f32 {
        self.num(name).unwrap_or(default)
    }
    fn need(&self, name: &str) -> Res<f32> {
        self.num(name).ok_or_else(|| self.missing(name, "a number"))
    }
    fn missing(&self, name: &str, what: &str) -> String {
        format!("<{:?}>: `{name}` must be {what}", self.kind)
    }
    fn text(&self, name: &str) -> Option<&str> {
        self.get(name)?.text()
    }
    fn flag(&self, name: &str) -> Option<bool> {
        match self.get(name)? {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    fn point(&self, name: &str) -> Res<Point> {
        self.get(name).and_then(Value::point).ok_or_else(|| self.missing(name, "a point"))
    }
    /// A prop of one of `names`, as its number.
    fn one_of<T>(&self, name: &str, names: &[&str], make: fn(i32) -> Result<T, String>) -> Res<Option<T>> {
        let Some(text) = self.text(name) else { return Ok(None) };
        let at = names
            .iter()
            .position(|n| *n == text)
            .ok_or_else(|| self.missing(name, &format!("one of {}", names.join(", "))))?;
        make(at as i32).map(Some)
    }
    fn blend(&self, name: &str) -> Res<Option<BlendMode>> {
        self.one_of(name, BLENDS, enums::blend)
    }
    fn tile(&self, name: &str, default: skia_safe::TileMode) -> Res<skia_safe::TileMode> {
        Ok(self.one_of(name, TILES, enums::tile)?.unwrap_or(default))
    }
    fn rect(&self) -> Res<Rect> {
        if let Some(r) = self.get("rect") {
            return r.rect().ok_or_else(|| self.missing("rect", "a rectangle"));
        }
        let (x, y) = (self.num_or("x", 0.0), self.num_or("y", 0.0));
        Ok(Rect::from_xywh(x, y, self.need("width")?, self.need("height")?))
    }
    fn rrect(&self) -> Res<RRect> {
        if let Some(r) = self.get("rect") {
            return r.rrect().ok_or_else(|| self.missing("rect", "a rounded rectangle"));
        }
        let r = self.get("r").and_then(Value::radius).unwrap_or_default();
        Ok(RRect::new_rect_xy(self.rect()?, r.x, r.y))
    }
    fn path(&self) -> Res<skia_safe::Path> {
        self.get("path").and_then(Value::path).ok_or_else(|| self.missing("path", "a path or an SVG path string"))
    }
    fn colors(&self) -> Res<Option<Vec<Color4f>>> {
        match self.get("colors") {
            None => Ok(None),
            Some(Value::Colors(c)) => Ok(Some(c.clone())),
            Some(_) => Err(self.missing("colors", "a list of colors")),
        }
    }
    fn color(&self) -> Option<Color4f> {
        self.get("color").and_then(Value::color)
    }
    fn floats(&self, name: &str) -> Res<Option<Vec<f32>>> {
        match self.get(name) {
            None => Ok(None),
            Some(Value::Floats(f)) => Ok(Some(f.clone())),
            Some(_) => Err(self.missing(name, "a list of numbers")),
        }
    }
    /// The `matrix` or `transform` prop, around `origin`.
    fn matrix(&self) -> Res<Option<Matrix>> {
        let m = if let Some(m) = self.get("matrix") {
            m.matrix().ok_or_else(|| self.missing("matrix", "a matrix"))?
        } else if let Some(t) = self.get("transform") {
            transform(t).ok_or_else(|| self.missing("transform", "a list of transforms"))?
        } else {
            return Ok(None);
        };
        Ok(Some(match self.get("origin").and_then(Value::point) {
            None => m,
            Some(o) => {
                let mut out = Matrix::translate(o);
                out.pre_concat(&m).pre_translate((-o.x, -o.y));
                out
            }
        }))
    }
    fn sampling(&self) -> Res<SamplingOptions> {
        sampling_of(self.get("sampling"))
    }
    /// Its kind: a `<Blend>` of image filters is one, else it is a shader.
    fn family(&self) -> Family {
        match self.kind.family() {
            Family::Mixed => match self.children.first().map(|c| c.borrow().family()) {
                Some(Family::ImageFilters) => Family::ImageFilters,
                _ => Family::Shaders,
            },
            family => family,
        }
    }
    fn of(&self, family: Family) -> impl Iterator<Item = &NodeRc> {
        self.children.iter().filter(move |c| c.borrow().family() == family)
    }
}

const BLENDS: &[&str] = &[
    "clear",
    "src",
    "dst",
    "srcOver",
    "dstOver",
    "srcIn",
    "dstIn",
    "srcOut",
    "dstOut",
    "srcATop",
    "dstATop",
    "xor",
    "plus",
    "modulate",
    "screen",
    "overlay",
    "darken",
    "lighten",
    "colorDodge",
    "colorBurn",
    "hardLight",
    "softLight",
    "difference",
    "exclusion",
    "multiply",
    "hue",
    "saturation",
    "color",
    "luminosity",
];
const TILES: &[&str] = &["clamp", "repeat", "mirror", "decal"];
const BLURS: &[&str] = &["normal", "solid", "outer", "inner"];
const CHANNELS: &[&str] = &["r", "g", "b", "a"];
const FILLS: &[&str] = &["winding", "evenOdd", "inverseWinding", "inverseEvenOdd"];
const CAPS: &[&str] = &["butt", "round", "square"];
const JOINS: &[&str] = &["miter", "round", "bevel"];

// --- Transforms ------------------------------------------------------------------

type M4 = [f32; 16];

fn mul(a: &M4, b: &M4) -> M4 {
    std::array::from_fn(|i| (0..4).map(|k| a[i / 4 * 4 + k] * b[k * 4 + i % 4]).sum())
}

fn translate4(x: f32, y: f32, z: f32) -> M4 {
    [1., 0., 0., x, 0., 1., 0., y, 0., 0., 1., z, 0., 0., 0., 1.]
}

fn scale4(x: f32, y: f32) -> M4 {
    [x, 0., 0., 0., 0., y, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.]
}

fn rotate4(axis: usize, angle: f32) -> M4 {
    let (s, c) = angle.sin_cos();
    match axis {
        0 => [1., 0., 0., 0., 0., c, -s, 0., 0., s, c, 0., 0., 0., 0., 1.],
        1 => [c, 0., s, 0., 0., 1., 0., 0., -s, 0., c, 0., 0., 0., 0., 1.],
        _ => [c, -s, 0., 0., s, c, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.],
    }
}

/// A list of `{ rotate: 1 }`s as a matrix, multiplied in order in 3D.
pub(crate) fn transform(value: &Value) -> Option<Matrix> {
    let list = match value {
        Value::List(l) => &l[..],
        Value::Floats(f) if f.is_empty() => &[],
        _ => return None,
    };
    let mut m = translate4(0., 0., 0.);
    for item in list {
        let Value::Fields(fields) = item else { return None };
        for (key, v) in fields {
            let n = || v.number();
            let step = match key.as_str() {
                "translateX" => translate4(n()?, 0., 0.),
                "translateY" => translate4(0., n()?, 0.),
                "translateZ" => translate4(0., 0., n()?),
                "translate" => match v {
                    Value::Floats(f) if f.len() >= 2 => translate4(f[0], f[1], f.get(2).copied().unwrap_or(0.)),
                    _ => return None,
                },
                "scale" => scale4(n()?, n()?),
                "scaleX" => scale4(n()?, 1.),
                "scaleY" => scale4(1., n()?),
                "skewX" => [1., n()?.tan(), 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.],
                "skewY" => [1., 0., 0., 0., n()?.tan(), 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.],
                "rotate" | "rotateZ" => rotate4(2, n()?),
                "rotateX" => rotate4(0, n()?),
                "rotateY" => rotate4(1, n()?),
                "perspective" => [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., -1. / n()?, 1.],
                "matrix" => match v {
                    Value::Floats(f) => f[..].try_into().ok()?,
                    _ => return None,
                },
                _ => return None,
            };
            m = mul(&m, &step);
        }
    }
    crate::args::matrix_of(&m)
}

// --- Fitting ---------------------------------------------------------------------

/// The part of `rect` and the place in `dst` a fit takes.
fn fit_rects(fit: &str, rect: Rect, dst: Rect) -> Res<(Rect, Rect)> {
    type Size = (f32, f32);
    let (input, output): (Size, Size) = ((rect.width(), rect.height()), (dst.width(), dst.height()));
    let empty = input.0 <= 0. || input.1 <= 0. || output.0 <= 0. || output.1 <= 0.;
    let wider = output.0 / output.1 > input.0 / input.1;
    let (src, to): (Size, Size) = match fit {
        _ if empty => ((0., 0.), (0., 0.)),
        "fill" => (input, output),
        "contain" if wider => (input, (input.0 * output.1 / input.1, output.1)),
        "contain" => (input, (output.0, input.1 * output.0 / input.0)),
        "cover" if wider => ((input.0, input.0 * output.1 / output.0), output),
        "cover" => ((input.1 * output.0 / output.1, input.1), output),
        "fitWidth" => {
            let src = (input.0, input.0 * output.1 / output.0);
            (src, (output.0, src.1 * output.0 / src.0))
        }
        "fitHeight" => {
            let src = (input.1 * output.0 / output.1, input.1);
            (src, (src.0 * output.1 / src.1, output.1))
        }
        "none" => {
            let s = (input.0.min(output.0), input.1.min(output.1));
            (s, s)
        }
        "scaleDown" => {
            let ratio = input.0 / input.1;
            let mut to = input;
            if to.1 > output.1 {
                to = (output.1 * ratio, output.1);
            }
            if to.0 > output.0 {
                to = (output.0, output.0 / ratio);
            }
            (input, to)
        }
        _ => return Err(format!("`fit`: there is no {fit}")),
    };
    let inscribe =
        |s: Size, r: Rect| Rect::from_xywh(r.left + (r.width() - s.0) / 2., r.top + (r.height() - s.1) / 2., s.0, s.1);
    Ok((inscribe(src, rect), inscribe(to, dst)))
}

// --- Filters, effects and shaders ------------------------------------------------

fn none(what: &str, node: &Node) -> String {
    format!("<{:?}>: Skia made no {what} of these props", node.kind)
}

/// Pops the last two, and pushes what `f` makes of them.
fn compose<T>(stack: &mut Vec<T>, f: impl FnOnce(T, T) -> Option<T>) {
    if stack.len() > 1 {
        let (Some(outer), Some(inner)) = (stack.pop(), stack.pop()) else { return };
        stack.extend(f(outer, inner));
    }
}

/// All of them, from the last to the first.
fn fold<T>(stack: Vec<T>, f: impl Fn(T, T) -> Option<T>) -> Option<T> {
    let mut items = stack.into_iter().rev();
    let last = items.next()?;
    items.try_fold(last, |inner, outer| f(outer, inner))
}

fn push_color_filters<'a>(nodes: impl Iterator<Item = &'a NodeRc>, stack: &mut Vec<ColorFilter>) -> Res<()> {
    for node in nodes {
        let node = node.borrow();
        push_color_filters(node.children.iter(), stack)?;
        let made = match node.kind {
            Kind::MatrixColorFilter => {
                let m: [f32; 20] = node
                    .floats("matrix")?
                    .and_then(|m| m.try_into().ok())
                    .ok_or_else(|| node.missing("matrix", "20 numbers"))?;
                Some(color_filters::matrix_row_major(&m, None))
            }
            Kind::BlendColorFilter => {
                let color = node.color().ok_or_else(|| node.missing("color", "a color"))?;
                let mode = node.blend("mode")?.ok_or_else(|| node.missing("mode", "a blend mode"))?;
                color_filters::blend_with_color_space(color, None, mode)
            }
            Kind::LerpColorFilter => {
                let (Some(second), Some(first)) = (stack.pop(), stack.pop()) else {
                    return Err("<Lerp> needs two color filters".into());
                };
                color_filters::lerp(node.need("t")?, first, second)
            }
            Kind::LinearToSRGBGammaColorFilter => Some(color_filters::linear_to_srgb_gamma()),
            Kind::SRGBToLinearGammaColorFilter => Some(color_filters::srgb_to_linear_gamma()),
            _ => Some(skia_safe::luma_color_filter::new()),
        };
        stack.push(made.ok_or_else(|| none("color filter", &node))?);
        if node.kind != Kind::LerpColorFilter && !node.children.is_empty() {
            compose(stack, color_filters::compose);
        }
    }
    Ok(())
}

fn push_path_effects<'a>(nodes: impl Iterator<Item = &'a NodeRc>, stack: &mut Vec<PathEffect>) -> Res<()> {
    for node in nodes {
        let node = node.borrow();
        push_path_effects(node.children.iter(), stack)?;
        let matrix = || node.get("matrix").and_then(Value::matrix).ok_or_else(|| node.missing("matrix", "a matrix"));
        let made = match node.kind {
            Kind::DiscretePathEffect => {
                PathEffect::discrete(node.need("length")?, node.need("deviation")?, node.num_or("seed", 0.) as u32)
            }
            Kind::DashPathEffect => {
                let intervals =
                    node.floats("intervals")?.ok_or_else(|| node.missing("intervals", "a list of numbers"))?;
                dash_path_effect::new(&intervals, node.num_or("phase", 0.))
            }
            Kind::CornerPathEffect => PathEffect::corner_path(node.need("r")?),
            Kind::Line2DPathEffect => PathEffect::line_2d(node.need("width")?, &matrix()?),
            Kind::Path2DPathEffect => Some(path_2d_path_effect::new(&matrix()?, &node.path()?)),
            Kind::Path1DPathEffect => {
                let style = node
                    .one_of("style", &["translate", "rotate", "morph"], enums::path_1d)?
                    .unwrap_or(path_1d_path_effect::Style::Translate);
                path_1d_path_effect::new(&node.path()?, node.need("advance")?, node.num_or("phase", 0.), style)
            }
            _ => fold(std::mem::take(stack), |outer, inner| Some(PathEffect::sum(outer, inner))),
        };
        stack.push(made.ok_or_else(|| none("path effect", &node))?);
        if node.kind != Kind::SumPathEffect && !node.children.is_empty() {
            compose(stack, |outer, inner| Some(PathEffect::compose(outer, inner)));
        }
    }
    Ok(())
}

fn push_shaders<'a>(nodes: impl Iterator<Item = &'a NodeRc>, stack: &mut Vec<Shader>) -> Res<()> {
    for node in nodes {
        let node = node.borrow();
        let first = stack.len();
        push_shaders(node.children.iter(), stack)?;
        let children = stack.split_off(first);
        if let Some(shader) = shader(&node, children)? {
            stack.push(shader);
        }
    }
    Ok(())
}

fn gradient_of(
    node: &Node,
    make: impl FnOnce(&gradient::Gradient<'_>, Option<&Matrix>) -> Option<Shader>,
) -> Res<Shader> {
    let colors = node.colors()?.ok_or_else(|| node.missing("colors", "a list of colors"))?;
    let positions = node.floats("positions")?;
    let mode = node.tile("mode", skia_safe::TileMode::Clamp)?;
    let flags = node.num_or("flags", 0.) as u32;
    let m = node.matrix()?;
    with_gradient(&colors, positions.as_deref(), mode, flags, |g| make(g, m.as_ref()))
}

/// The numbers of a program's uniforms, by name.
fn uniforms(effect: &SkRuntimeEffect, values: Option<&Value>) -> Res<Vec<f32>> {
    let mut out = Vec::new();
    for u in effect.effect.uniforms() {
        let value = values
            .and_then(|v| v.field(u.name()))
            .ok_or_else(|| format!("the program's uniform `{}` is missing from `uniforms`", u.name()))?;
        value.flatten(&mut out);
    }
    Ok(out)
}

fn shader(node: &Node, children: Vec<Shader>) -> Res<Option<Shader>> {
    let made = match node.kind {
        Kind::Shader => {
            let Some(Value::Effect(effect)) = node.get("source") else {
                return Err(node.missing("source", "a runtime effect"));
            };
            let m = node.matrix()?;
            return effect.with(|e| {
                let numbers = uniforms(e, node.get("uniforms"))?;
                e.shader(&numbers, &children, m.as_ref()).map(Some)
            });
        }
        Kind::ColorShader => {
            let color = node.color().ok_or_else(|| node.missing("color", "a color"))?;
            Some(shaders::color_in_space(color, skia_safe::ColorSpace::new_srgb()))
        }
        Kind::Turbulence | Kind::FractalNoise => Some(noise(
            node.kind == Kind::FractalNoise,
            node.need("freqX")?,
            node.need("freqY")?,
            node.need("octaves")? as u32,
            node.num_or("seed", 0.),
            node.num_or("tileWidth", 0.),
            node.num_or("tileHeight", 0.),
        )?),
        Kind::LinearGradient => {
            let points = (node.point("start")?, node.point("end")?);
            Some(gradient_of(node, |g, m| gradient::shaders::linear_gradient(points, g, m))?)
        }
        Kind::RadialGradient => {
            let circle = (node.point("c")?, node.need("r")?);
            Some(gradient_of(node, |g, m| gradient::shaders::radial_gradient(circle, g, m))?)
        }
        Kind::SweepGradient => {
            let (c, angles) = (node.point("c")?, (node.num_or("start", 0.), node.num_or("end", 360.)));
            Some(gradient_of(node, |g, m| gradient::shaders::sweep_gradient(c, angles, g, m))?)
        }
        Kind::TwoPointConicalGradient => {
            let start = (node.point("start")?, node.need("startR")?);
            let end = (node.point("end")?, node.need("endR")?);
            Some(gradient_of(node, |g, m| gradient::shaders::two_point_conical_gradient(start, end, g, m))?)
        }
        Kind::ImageShader => {
            let Some(Value::Image(image)) = node.get("image") else { return Ok(None) };
            let image = image.with(|i| i.0.clone());
            let mut m = Matrix::default();
            let has_rect = node.get("rect").is_some() || (node.num("width").is_some() && node.num("height").is_some());
            if has_rect {
                let bounds = Rect::from_iwh(image.width(), image.height());
                let (src, dst) = fit_rects(node.text("fit").unwrap_or("none"), bounds, node.rect()?)?;
                let (sx, sy) = (dst.width() / src.width(), dst.height() / src.height());
                m.pre_translate((dst.left - src.left * sx, dst.top - src.top * sy)).pre_scale((sx, sy), None);
            }
            if let Some(own) = node.matrix()? {
                m.pre_concat(&own);
            }
            let decal = skia_safe::TileMode::Decal;
            let tiles = (node.tile("tx", decal)?, node.tile("ty", decal)?);
            image.to_shader(tiles, node.sampling()?, &m)
        }
        Kind::Blend => {
            let mode = node.blend("mode")?.ok_or_else(|| node.missing("mode", "a blend mode"))?;
            match fold(children, |outer, inner| Some(shaders::blend(mode, outer, inner))) {
                None => return Ok(None),
                made => made,
            }
        }
        _ => return Err(format!("<{:?}> is no shader", node.kind)),
    };
    made.map(Some).ok_or_else(|| none("shader", node))
}

/// The shadow inside a shape.
fn inner_shadow(shadow_only: bool, offset: (f32, f32), sigma: f32, color: Color4f) -> Option<ImageFilter> {
    let black = Color4f::new(0., 0., 0., 1.);
    let of = |mode| color_filters::blend_with_color_space(black, None, mode);
    let alpha = image_filters::color_filter(of(BlendMode::SrcIn)?, None, None)?;
    // The silhouette: any pixel covered at all is opaque.
    let mut saturate = [0.0; 20];
    saturate[18] = 255.0;
    let coverage = image_filters::color_filter(color_filters::matrix_row_major(&saturate, None), None, None)?;
    let out = color_filters::blend_with_color_space(color, None, BlendMode::SrcOut)?;
    let f = image_filters::color_filter(out, coverage, None)?;
    let f = image_filters::offset(offset, f, None)?;
    let f = image_filters::blur((sigma, sigma), skia_safe::TileMode::Decal, f, None)?;
    let f = image_filters::blend(BlendMode::SrcIn, alpha, f, None)?;
    if shadow_only { Some(f) } else { image_filters::blend(BlendMode::SrcOver, None, f, None) }
}

fn push_image_filters<'a>(
    nodes: impl Iterator<Item = &'a NodeRc>,
    stack: &mut Vec<ImageFilter>,
    shader_stack: &mut Vec<Shader>,
) -> Res<()> {
    for node in nodes {
        let node = node.borrow();
        if node.family() == Family::Shaders {
            // A shader among the filters: the input of a displacement map.
            let first = shader_stack.len();
            push_shaders(node.children.iter(), shader_stack)?;
            let children = shader_stack.split_off(first);
            shader_stack.extend(shader(&node, children)?);
            continue;
        }
        push_image_filters(node.children.iter(), stack, shader_stack)?;
        let made = match node.kind {
            Kind::ImageFilter => {
                let Some(Value::ImageFilter(f)) = node.get("filter") else {
                    return Err(node.missing("filter", "an image filter"));
                };
                Some(f.with(|f| f.0.clone()))
            }
            Kind::BlurImageFilter => {
                let s = node
                    .get("blur")
                    .and_then(Value::radius)
                    .ok_or_else(|| node.missing("blur", "a number or a point"))?;
                image_filters::blur((s.x, s.y), node.tile("mode", skia_safe::TileMode::Decal)?, None, None)
            }
            Kind::OffsetImageFilter => image_filters::offset((node.num_or("x", 0.), node.num_or("y", 0.)), None, None),
            Kind::MorphologyImageFilter => {
                let r = node
                    .get("radius")
                    .and_then(Value::radius)
                    .ok_or_else(|| node.missing("radius", "a number or a point"))?;
                if node.text("operator") == Some("erode") {
                    image_filters::erode((r.x, r.y), None, None)
                } else {
                    image_filters::dilate((r.x, r.y), None, None)
                }
            }
            Kind::DropShadowImageFilter => {
                let offset = (node.need("dx")?, node.need("dy")?);
                let (sigma, color) = (node.need("blur")?, node.color().unwrap_or(Color4f::new(0., 0., 0., 1.)));
                let only = node.flag("shadowOnly") == Some(true);
                if node.flag("inner") == Some(true) {
                    inner_shadow(only, offset, sigma, color)
                } else if only {
                    image_filters::drop_shadow_only(offset, (sigma, sigma), color, None, None, None)
                } else {
                    image_filters::drop_shadow(offset, (sigma, sigma), color, None, None, None)
                }
            }
            Kind::DisplacementMapImageFilter => {
                let channel = |name| -> Res<_> {
                    node.one_of(name, CHANNELS, enums::channel)?.ok_or_else(|| node.missing(name, "r, g, b or a"))
                };
                let channels = (channel("channelX")?, channel("channelY")?);
                let map = shader_stack.pop().ok_or("<DisplacementMap> needs a shader as a child")?;
                let map = image_filters::shader(map, None);
                image_filters::displacement_map(channels, node.need("scale")?, map, None, None)
            }
            Kind::RuntimeShaderImageFilter => {
                let Some(Value::Effect(effect)) = node.get("source") else {
                    return Err(node.missing("source", "a runtime effect"));
                };
                effect.with(|e| -> Res<_> {
                    let mut builder = skia_safe::runtime_effect::RuntimeShaderBuilder::new(e.effect.clone());
                    let values = node.get("uniforms");
                    for u in e.effect.uniforms() {
                        if let Some(value) = values.and_then(|v| v.field(u.name())) {
                            let mut numbers = Vec::new();
                            value.flatten(&mut numbers);
                            set_uniform(&mut builder, u.name(), &numbers)?;
                        }
                    }
                    Ok(image_filters::runtime_shader(&builder, "", None))
                })?
            }
            _ => {
                let mode = node.blend("mode")?.ok_or_else(|| node.missing("mode", "a blend mode"))?;
                fold(std::mem::take(stack), |outer, inner| image_filters::blend(mode, outer, inner, None))
            }
        };
        stack.push(made.ok_or_else(|| none("image filter", &node))?);
        if node.kind != Kind::Blend && !node.children.is_empty() {
            compose(stack, image_filters::compose);
        }
    }
    Ok(())
}

fn mask_filter(node: &Node) -> Res<Option<MaskFilter>> {
    let Some(mask) = node.of(Family::MaskFilters).last() else { return Ok(None) };
    let mask = mask.borrow();
    let style = mask.one_of("style", BLURS, enums::blur_style)?.unwrap_or(skia_safe::BlurStyle::Normal);
    let respect = mask.flag("respectCTM").unwrap_or(true);
    // A blur of 0 is none.
    Ok(MaskFilter::blur(style, mask.need("blur")?, respect))
}

// --- Painting --------------------------------------------------------------------

#[derive(Clone)]
struct Level {
    paint: Paint,
    opacity: f32,
}

const PAINT_PROPS: &[&str] = &[
    "opacity",
    "color",
    "strokeWidth",
    "blendMode",
    "style",
    "strokeJoin",
    "strokeCap",
    "strokeMiter",
    "antiAlias",
    "dither",
    "paint",
];

fn set_paint(level: &mut Level, node: &Node, standalone: bool) -> Res<()> {
    if let Some(Value::Paint(paint)) = node.get("paint") {
        level.paint = paint.with(|p| p.0.clone());
        let alpha = level.paint.alpha_f();
        level.paint.set_alpha_f((alpha * level.opacity).clamp(0., 1.));
        return Ok(());
    }
    let paint = &mut level.paint;
    if let Some(opacity) = node.num("opacity") {
        if !standalone {
            level.opacity = (level.opacity * opacity).clamp(0., 1.);
        }
        paint.set_alpha_f((paint.alpha_f() * opacity).clamp(0., 1.));
    }
    if let Some(color) = node.color() {
        paint.set_shader(None).set_color4f(color, None);
        if !standalone {
            paint.set_alpha_f((color.a * level.opacity).clamp(0., 1.));
        }
    }
    if let Some(mode) = node.blend("blendMode")? {
        paint.set_blend_mode(mode);
    }
    if let Some(width) = node.num("strokeWidth") {
        paint.set_stroke_width(width);
    }
    if let Some(style) = node.one_of("style", &["fill", "stroke"], enums::paint_style)? {
        paint.set_style(style);
    }
    if let Some(join) = node.one_of("strokeJoin", JOINS, enums::join)? {
        paint.set_stroke_join(join);
    }
    if let Some(cap) = node.one_of("strokeCap", CAPS, enums::cap)? {
        paint.set_stroke_cap(cap);
    }
    if let Some(miter) = node.num("strokeMiter") {
        paint.set_stroke_miter(miter);
    }
    if let Some(on) = node.flag("antiAlias") {
        paint.set_anti_alias(on);
    }
    if let Some(on) = node.flag("dither") {
        paint.set_dither(on);
    }
    Ok(())
}

/// What a node's filter, effect and shader children make.
struct Effects {
    color_filters: Vec<ColorFilter>,
    image_filters: Vec<ImageFilter>,
    mask: Option<MaskFilter>,
    shaders: Vec<Shader>,
    path_effects: Vec<PathEffect>,
}

impl Effects {
    fn of(node: &Node) -> Res<Self> {
        let mut e = Effects {
            color_filters: Vec::new(),
            image_filters: Vec::new(),
            mask: None,
            shaders: Vec::new(),
            path_effects: Vec::new(),
        };
        push_color_filters(node.of(Family::ColorFilters), &mut e.color_filters)?;
        push_image_filters(node.of(Family::ImageFilters), &mut e.image_filters, &mut e.shaders)?;
        e.mask = mask_filter(node)?;
        push_shaders(node.of(Family::Shaders), &mut e.shaders)?;
        push_path_effects(node.of(Family::PathEffects), &mut e.path_effects)?;
        Ok(e)
    }

    fn apply(self, paint: &mut Paint) {
        if let Some(mask) = self.mask {
            paint.set_mask_filter(mask);
        }
        if let Some(f) = fold(self.color_filters, color_filters::compose) {
            paint.set_color_filter(f);
        }
        if let Some(shader) = self.shaders.into_iter().next_back() {
            paint.set_shader(shader);
        }
        if let Some(f) = fold(self.image_filters, image_filters::compose) {
            paint.set_image_filter(f);
        }
        if let Some(e) = fold(self.path_effects, |outer, inner| Some(PathEffect::compose(outer, inner))) {
            paint.set_path_effect(e);
        }
    }
}

struct Ctx<'a> {
    canvas: &'a Canvas,
    /// The paints of `<Paint>` children, for the next drawing.
    declarations: Vec<Paint>,
    errors: &'a mut Vec<String>,
}

impl Ctx<'_> {
    /// Draws sibling nodes, by `zIndex`.
    fn children<'n>(&mut self, nodes: impl Iterator<Item = &'n NodeRc>, level: &Level) {
        let mut nodes: Vec<(f32, &NodeRc)> = nodes.map(|n| (n.borrow().num_or("zIndex", 0.), n)).collect();
        if nodes.iter().any(|(z, _)| *z != 0.0) {
            nodes.sort_by(|a, b| a.0.total_cmp(&b.0));
        }
        for (_, node) in nodes {
            let saves = self.canvas.save_count();
            if let Err(e) = self.node(&node.borrow(), level) {
                // What failed is left out; its siblings still draw.
                self.canvas.restore_to_count(saves);
                self.errors.push(e);
            }
        }
    }

    fn node(&mut self, node: &Node, parent: &Level) -> Res<()> {
        let mut own = None;
        let has_effects =
            node.children.iter().any(|c| !matches!(c.borrow().family(), Family::Drawing | Family::Paints));
        if has_effects || node.props.iter().any(|(k, _)| PAINT_PROPS.contains(&k.as_str())) {
            let mut level = parent.clone();
            set_paint(&mut level, node, false)?;
            let mut effects = Effects::of(node)?;
            if node.kind == Kind::BackdropFilter {
                self.backdrop(&mut effects);
            } else {
                effects.apply(&mut level.paint);
                own = Some(level);
            }
        }
        let level = own.as_ref().unwrap_or(parent);
        for paint in node.of(Family::Paints) {
            let paint = paint.borrow();
            let mut declared = Level { paint: new_paint(), opacity: level.opacity };
            set_paint(&mut declared, &paint, true)?;
            Effects::of(&paint)?.apply(&mut declared.paint);
            self.declarations.push(declared.paint);
        }
        let saves = self.canvas.save_count();
        if node.kind == Kind::Layer {
            let paint = self.declarations.pop();
            save_layer(self.canvas, paint.as_ref());
        }
        self.ctm(node)?;
        self.drawing(node, level)?;
        self.children(node.of(Family::Drawing), level);
        self.canvas.restore_to_count(saves);
        Ok(())
    }

    fn backdrop(&self, effects: &mut Effects) {
        let filter = effects
            .image_filters
            .pop()
            .or_else(|| image_filters::color_filter(effects.color_filters.pop()?, None, None));
        if let Some(filter) = filter {
            self.canvas.save_layer(&SaveLayerRec::default().backdrop(&filter));
            self.canvas.restore();
        }
    }

    fn ctm(&self, node: &Node) -> Res<()> {
        let matrix = node.matrix()?;
        let clip = node.get("clip").filter(|c| !matches!(c, Value::Bool(_)));
        let layer = match node.get("layer") {
            Some(Value::Paint(p)) => Some(Some(p.with(|p| p.0.clone()))),
            Some(Value::Bool(true)) => Some(None),
            _ => None,
        };
        match &layer {
            Some(paint) => {
                save_layer(self.canvas, paint.as_ref());
            }
            None if matrix.is_some() || clip.is_some() => {
                self.canvas.save();
            }
            None => {}
        }
        if let Some(m) = matrix {
            self.canvas.concat(&m);
        }
        if let Some(clip) = clip {
            let op = if node.flag("invertClip") == Some(true) { ClipOp::Difference } else { ClipOp::Intersect };
            let wrong = || node.missing("clip", "a path, a rectangle or a rounded rectangle");
            if let Some(path) = clip.path() {
                self.canvas.clip_path(&path, op, true);
            } else if clip.is_rrect() {
                self.canvas.clip_rrect(clip.rrect().ok_or_else(wrong)?, op, true);
            } else {
                self.canvas.clip_rect(clip.rect().ok_or_else(wrong)?, op, true);
            }
        }
        Ok(())
    }

    /// Draws with the paint, then with each paint declared.
    fn drawing(&mut self, node: &Node, level: &Level) -> Res<()> {
        if matches!(node.kind, Kind::Group | Kind::Layer | Kind::BoxShadow | Kind::BackdropFilter) {
            return Ok(());
        }
        let declarations = std::mem::take(&mut self.declarations);
        self.draw(node, &level.paint)?;
        declarations.iter().try_for_each(|paint| self.draw(node, paint))
    }

    fn draw(&self, node: &Node, paint: &Paint) -> Res<()> {
        let canvas = self.canvas;
        match node.kind {
            Kind::Fill => done(canvas.draw_paint(paint)),
            Kind::Circle => {
                let c = match (node.num("cx"), node.num("cy")) {
                    (None, None) => node.get("c").and_then(Value::point).unwrap_or_default(),
                    (x, y) => Point::new(x.unwrap_or(0.), y.unwrap_or(0.)),
                };
                canvas.draw_circle(c, node.need("r")?, paint);
            }
            Kind::Rect => done(canvas.draw_rect(node.rect()?, paint)),
            Kind::RRect => done(canvas.draw_rrect(node.rrect()?, paint)),
            Kind::Oval => done(canvas.draw_oval(node.rect()?, paint)),
            Kind::Line => done(canvas.draw_line(node.point("p1")?, node.point("p2")?, paint)),
            Kind::Points => {
                let points = self.points(node, "points")?.ok_or_else(|| node.missing("points", "a list of points"))?;
                let mode = node
                    .one_of("mode", &["points", "lines", "polygon"], enums::point_mode)?
                    .unwrap_or(skia_safe::canvas::PointMode::Points);
                canvas.draw_points(mode, &points, paint);
            }
            Kind::Path => done(canvas.draw_path(&path_drawn(node)?, paint)),
            Kind::DiffRect => {
                let rrect = |name| {
                    node.get(name).and_then(Value::rrect).ok_or_else(|| node.missing(name, "a rounded rectangle"))
                };
                canvas.draw_drrect(rrect("outer")?, rrect("inner")?, paint);
            }
            Kind::Image => {
                let Some(Value::Image(image)) = node.get("image") else { return Ok(()) };
                let image = image.with(|i| i.0.clone());
                let bounds = Rect::from_iwh(image.width(), image.height());
                let (src, dst) = fit_rects(node.text("fit").unwrap_or("contain"), bounds, node.rect()?)?;
                let src = Some((&src, SrcRectConstraint::Strict));
                canvas.draw_image_rect_with_sampling_options(&image, src, dst, node.sampling()?, paint);
            }
            Kind::Text => {
                let Some(Value::Font(font)) = node.get("font") else { return Ok(()) };
                let text = match node.get("text") {
                    Some(Value::Number(n)) => n.to_string(),
                    v => v.and_then(Value::text).ok_or_else(|| node.missing("text", "a string"))?.to_owned(),
                };
                let at = (node.num_or("x", 0.), node.num_or("y", 0.));
                font.with(|f| done(canvas.draw_str(text, at, &f.0, paint)));
            }
            Kind::TextPath => {
                let Some(Value::Font(font)) = node.get("font") else { return Ok(()) };
                let text = node.text("text").ok_or_else(|| node.missing("text", "a string"))?;
                let path = node.path()?;
                let offset = node.num_or("initialOffset", 0.);
                if let Some(blob) = font.with(|f| text_on_path(text, &path, &f.0, offset)) {
                    canvas.draw_text_blob(&blob, (0., 0.), paint);
                }
            }
            Kind::TextBlob => {
                let Some(Value::Blob(blob)) = node.get("blob") else { return Err(node.missing("blob", "a text blob")) };
                let at = (node.num_or("x", 0.), node.num_or("y", 0.));
                blob.with(|b| done(canvas.draw_text_blob(&b.0, at, paint)));
            }
            Kind::Glyphs => {
                let Some(Value::Font(font)) = node.get("font") else { return Ok(()) };
                let glyphs = match node.get("glyphs") {
                    Some(Value::List(l)) => &l[..],
                    Some(Value::Floats(f)) if f.is_empty() => &[],
                    _ => return Err(node.missing("glyphs", "a list of glyphs")),
                };
                let ids: Vec<u16> = glyphs.iter().filter_map(|g| g.num_field("id")).map(|g| g as u16).collect();
                let at: Vec<Point> = glyphs.iter().filter_map(|g| g.field("pos")?.point()).collect();
                if ids.len() != glyphs.len() || at.len() != glyphs.len() {
                    return Err(node.missing("glyphs", "a list of { id, pos }"));
                }
                let origin = (node.num_or("x", 0.), node.num_or("y", 0.));
                font.with(|f| canvas.draw_glyphs_at(&ids, &at[..], origin, &f.0, paint));
            }
            Kind::Picture => {
                let Some(Value::Picture(p)) = node.get("picture") else {
                    return Err(node.missing("picture", "a picture"));
                };
                p.with(|p| done(canvas.draw_picture(&p.0, None, None)));
            }
            Kind::Paragraph => {
                let Some(Value::Paragraph(p)) = node.get("paragraph") else { return Ok(()) };
                let at = (node.num_or("x", 0.), node.num_or("y", 0.));
                let width = node.need("width")?;
                p.with(|p| {
                    p.0.borrow_mut().layout(width);
                    p.0.borrow().paint(canvas, at);
                });
            }
            Kind::ImageSVG => {
                let Some(Value::SvgDom(svg)) = node.get("svg") else { return Ok(()) };
                let of = |name| node.get("rect").map_or_else(|| node.num(name), |r| r.num_field(name));
                canvas.save();
                if let (Some(x), Some(y)) = (of("x"), of("y")) {
                    canvas.translate((x, y));
                }
                svg.with(|s| svg_on(canvas, &s.0, of("width"), of("height")));
                canvas.restore();
            }
            Kind::Vertices => {
                let positions =
                    self.points(node, "vertices")?.ok_or_else(|| node.missing("vertices", "a list of points"))?;
                let textures = self.points(node, "textures")?;
                let colors = node.colors()?;
                let indices = node.floats("indices")?;
                let mode = node
                    .one_of("mode", &["triangles", "trianglesStrip", "triangleFan"], enums::vertex_mode)?
                    .unwrap_or(skia_safe::vertices::VertexMode::Triangles);
                let mesh = vertices(mode, &positions, textures.as_deref(), colors.as_deref(), indices.as_deref())?;
                let blend = node.blend("blendMode")?.unwrap_or(default_blend(colors.is_some()));
                canvas.draw_vertices(&mesh, blend, paint);
            }
            Kind::Patch => {
                let Some(Value::List(corners)) = node.get("patch") else {
                    return Err(node.missing("patch", "4 bezier handles"));
                };
                let at = |i: usize, key: &str| {
                    corners
                        .get(i)
                        .and_then(|c| c.field(key)?.point())
                        .ok_or_else(|| node.missing("patch", "4 bezier handles"))
                };
                let cubics = [
                    at(0, "pos")?,
                    at(0, "c2")?,
                    at(1, "c1")?,
                    at(1, "pos")?,
                    at(1, "c2")?,
                    at(2, "c1")?,
                    at(2, "pos")?,
                    at(2, "c2")?,
                    at(3, "c1")?,
                    at(3, "pos")?,
                    at(3, "c2")?,
                    at(0, "c1")?,
                ];
                let colors = node.colors()?.map(|c| four_colors(&c)).transpose()?;
                let texture: Option<[Point; 4]> = node
                    .get("texture")
                    .map(|t| t.points().and_then(|p| p.try_into().ok()))
                    .map(|t| t.ok_or_else(|| node.missing("texture", "4 points")))
                    .transpose()?;
                let blend = node.blend("blendMode")?.unwrap_or(default_blend(colors.is_some()));
                canvas.draw_patch(&cubics, colors.as_ref(), texture.as_ref(), blend, paint);
            }
            Kind::Atlas => {
                let Some(Value::Image(image)) = node.get("image") else { return Ok(()) };
                let list = |name| match node.get(name) {
                    Some(Value::List(l)) => Ok(&l[..]),
                    Some(Value::Floats(f)) if f.is_empty() => Ok(&[][..]),
                    _ => Err(node.missing(name, "a list")),
                };
                let sprites: Option<Vec<Rect>> = list("sprites")?.iter().map(Value::rect).collect();
                let transforms: Option<Vec<RSXform>> = list("transforms")?
                    .iter()
                    .map(|t| {
                        let at = (t.num_field("tx")?, t.num_field("ty")?);
                        Some(RSXform::new(t.num_field("scos")?, t.num_field("ssin")?, at))
                    })
                    .collect();
                let sprites = sprites.ok_or_else(|| node.missing("sprites", "a list of rectangles"))?;
                let transforms = transforms.ok_or_else(|| node.missing("transforms", "a list of RSXforms"))?;
                let (colors, mode, sampling) = (node.colors()?, node.blend("colorBlendMode")?, node.sampling()?);
                image.with(|i| {
                    atlas_on(canvas, &i.0, &sprites, &transforms, colors.as_deref(), mode, sampling, paint)
                })?;
            }
            Kind::Box => self.draw_box(node, paint)?,
            _ => {}
        }
        Ok(())
    }

    fn points(&self, node: &Node, name: &str) -> Res<Option<Vec<Point>>> {
        node.get(name).map(|v| v.points().ok_or_else(|| node.missing(name, "a list of points"))).transpose()
    }

    fn draw_box(&self, node: &Node, paint: &Paint) -> Res<()> {
        let canvas = self.canvas;
        let shape = node
            .get("box")
            .and_then(Value::rrect)
            .ok_or_else(|| node.missing("box", "a rectangle or a rounded rectangle"))?;
        let grown = |by: (f32, f32), at: (f32, f32)| {
            let r = shape.rect();
            let radii = shape.simple_radii();
            let rect = Rect::from_xywh(
                r.left - by.0 + at.0,
                r.top - by.1 + at.1,
                r.width() + 2. * by.0,
                r.height() + 2. * by.1,
            );
            RRect::new_rect_xy(rect, radii.x + by.0, radii.y + by.1)
        };
        let mut inner = Vec::new();
        for shadow in node.children.iter().filter(|c| c.borrow().kind == Kind::BoxShadow) {
            let s = shadow.borrow();
            let color = s.color().unwrap_or(Color4f::new(0., 0., 0., 1.));
            let mut p = new_paint();
            p.set_color4f(Color4f { a: color.a * paint.alpha_f(), ..color }, None);
            p.set_mask_filter(MaskFilter::blur(skia_safe::BlurStyle::Normal, s.need("blur")?, true));
            let (spread, at) = (s.num_or("spread", 0.), (s.num_or("dx", 0.), s.num_or("dy", 0.)));
            if s.flag("inner") == Some(true) {
                inner.push((p, spread, at));
            } else {
                canvas.draw_rrect(grown((spread, spread), at), &p);
            }
        }
        canvas.draw_rrect(shape, paint);
        for (p, spread, at) in inner {
            canvas.save();
            canvas.clip_rrect(shape, ClipOp::Intersect, false);
            let outer = grown((10. + at.0.abs(), 10. + at.1.abs()), (0., 0.));
            canvas.draw_drrect(outer, grown((-spread, -spread), at), &p);
            canvas.restore();
        }
        Ok(())
    }
}

fn save_layer(canvas: &Canvas, paint: Option<&Paint>) {
    let mut rec = SaveLayerRec::default();
    if let Some(p) = paint {
        rec = rec.paint(p);
    }
    canvas.save_layer(&rec);
}

/// The path a `<Path>` draws: trimmed, then filled so, then stroked.
fn path_drawn(node: &Node) -> Res<skia_safe::Path> {
    let mut path = node.path()?;
    let (start, end) = (node.num_or("start", 0.).clamp(0., 1.), node.num_or("end", 1.).clamp(0., 1.));
    if (start != 0. || end != 1.)
        && let Some(t) = trimmed(&path, start, end, false)
    {
        path = t;
    }
    if let Some(fill) = node.one_of("fillType", FILLS, enums::fill_type)? {
        path = path.with_fill_type(fill);
    }
    if let Some(s) = node.get("stroke") {
        let int = |key| s.num_field(key).map(|n| n as i32);
        let stroke = Stroke {
            width: s.num_field("width").unwrap_or(1.),
            miter: s.num_field("miter_limit").unwrap_or(4.),
            precision: s.num_field("precision").unwrap_or(1.),
            join: int("join").map(enums::join).transpose()?,
            cap: int("cap").map(enums::cap).transpose()?,
        };
        if let Some(out) = stroked(&path, &stroke) {
            path = out;
        }
    }
    Ok(path)
}

/// Text along a path, each glyph turned with it.
fn text_on_path(text: &str, path: &skia_safe::Path, font: &skia_safe::Font, offset: f32) -> Option<TextBlob> {
    let ids = font.str_to_glyphs_vec(text);
    let mut widths = vec![0.; ids.len()];
    font.get_widths(&ids, &mut widths);
    let mut contours = skia_safe::ContourMeasureIter::new(path, false, 1.);
    let mut contour = contours.next()?;
    let mut at = offset;
    let mut transforms = Vec::with_capacity(ids.len());
    for width in &widths {
        at += width / 2.;
        if at > contour.length() {
            let Some(next) = contours.next() else { break };
            contour = next;
            at = width / 2.;
        }
        let (p, t) = contour.pos_tan(at)?;
        transforms.push(RSXform::new(t.x, t.y, (p.x - width / 2. * t.x, p.y - width / 2. * t.y)));
        at += width / 2.;
    }
    TextBlob::from_rsxform(&ids[..transforms.len()], &transforms, font)
}
