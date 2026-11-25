//! Paths, their builder and their measure.

use nsh_core::prelude::*;
use skia_safe::{
    ContourMeasure, ContourMeasureIter, Matrix, Paint, Path, PathBuilder, PathDirection, PathEffect, Point, Rect,
    StrokeRec, dash_path_effect, paint, path, path_builder, path_utils, stroke_rec, trim_path_effect,
    utils::parse_path,
};

use crate::{
    args::{Floats, MatrixArg, PointOut, Points, Pt, RRectArg, RectArg, RectOut, enums},
    objects::{SkFont, wrapper},
};

/// Nothing, of a chained call's result.
pub(crate) fn done<T>(_: T) {}

/// The error of a Skia call that gave nothing.
pub(crate) fn bad(what: &'static str) -> impl FnOnce() -> String {
    move || format!("Skia made no {what}: an argument is out of its range")
}

wrapper!(/** A path (immutable: make one with `Skia.PathBuilder`). */ SkPath(Path));

fn some(path: Option<Path>) -> Option<Instance<SkPath>> {
    path.map(|p| Instance::new(p.into()))
}

fn direction(ccw: Option<bool>) -> PathDirection {
    if ccw == Some(true) { PathDirection::CCW } else { PathDirection::CW }
}

pub(crate) struct Stroke {
    pub(crate) width: f32,
    pub(crate) miter: f32,
    pub(crate) precision: f32,
    pub(crate) join: Option<paint::Join>,
    pub(crate) cap: Option<paint::Cap>,
}

/// The outline of `path`'s stroke.
pub(crate) fn stroked(path: &Path, s: &Stroke) -> Option<Path> {
    let mut paint = Paint::default();
    paint.set_style(paint::Style::Stroke).set_stroke_width(s.width).set_stroke_miter(s.miter);
    paint.set_stroke_join(s.join.unwrap_or_default()).set_stroke_cap(s.cap.unwrap_or_default());
    let mut out = PathBuilder::new();
    let matrix = Matrix::scale((s.precision, s.precision));
    path_utils::fill_path_with_paint(path, &paint, &mut out, None, Some(matrix)).then(|| out.detach())
}

/// `path` through a path effect.
fn filtered(path: &Path, effect: Option<PathEffect>) -> Option<Path> {
    let rec = StrokeRec::new(stroke_rec::InitStyle::Hairline);
    effect?.filter_path(path, &rec, Rect::default()).map(|(mut p, _)| p.detach())
}

/// The part of `path` between `start` and `end` (0 to 1).
pub(crate) fn trimmed(path: &Path, start: f32, end: f32, complement: bool) -> Option<Path> {
    let (start, end) = (start.clamp(0.0, 1.0), end.clamp(0.0, 1.0));
    if !complement && start <= 0.0 && end >= 1.0 {
        return Some(path.clone());
    }
    let mode = if complement { trim_path_effect::Mode::Inverted } else { trim_path_effect::Mode::Normal };
    filtered(path, trim_path_effect::new(start, end, mode))
}

#[class]
impl SkPath {
    /// An empty path.
    pub fn make() -> SkPath {
        Path::new().into()
    }
    /// The path of an SVG string.
    pub fn from_svg(text: String) -> Option<Instance<SkPath>> {
        some(parse_path::from_svg(&text))
    }
    /// Two paths combined.
    pub fn from_op(one: &SkPath, two: &SkPath, op: i32) -> Result<Option<Instance<SkPath>>, String> {
        Ok(some(skia_safe::op(&one.0, &two.0, enums::path_op(op)?)))
    }
    /// The path of verbs and their numbers, in one list.
    pub fn from_cmds(cmds: Floats) -> Option<Instance<SkPath>> {
        let mut b = PathBuilder::new();
        let mut rest = &cmds.0[..];
        while let Some((verb, tail)) = rest.split_first() {
            let n = match *verb as i32 {
                0 | 1 => 2,
                2 => 4,
                3 => 5,
                4 => 6,
                5 => 0,
                _ => return None,
            };
            let (a, tail) = tail.split_at_checked(n)?;
            match *verb as i32 {
                0 => b.move_to((a[0], a[1])),
                1 => b.line_to((a[0], a[1])),
                2 => b.quad_to((a[0], a[1]), (a[2], a[3])),
                3 => b.conic_to((a[0], a[1]), (a[2], a[3]), a[4]),
                4 => b.cubic_to((a[0], a[1]), (a[2], a[3]), (a[4], a[5])),
                _ => b.close(),
            };
            rest = tail;
        }
        some(Some(b.detach()))
    }
    /// The outlines of text.
    pub fn from_text(text: String, x: f32, y: f32, font: &SkFont) -> Option<Instance<SkPath>> {
        let ids = font.0.str_to_glyphs_vec(text);
        let mut widths = vec![0.0; ids.len()];
        font.0.get_widths(&ids, &mut widths);
        let mut b = PathBuilder::new();
        let mut at = x;
        for (id, width) in ids.iter().zip(widths) {
            if let Some(glyph) = font.0.get_path(*id) {
                b.add_path_with_offset(&glyph, (at, y), None);
            }
            at += width;
        }
        some(Some(b.detach()))
    }
    /// A rectangle.
    pub fn rect(rect: RectArg, ccw: Option<bool>) -> SkPath {
        Path::rect(rect.0, direction(ccw)).into()
    }
    /// An oval.
    pub fn oval(rect: RectArg, ccw: Option<bool>, start_index: Option<usize>) -> SkPath {
        Path::oval_with_start_index(rect.0, direction(ccw), start_index.unwrap_or(1) % 4).into()
    }
    /// A circle.
    pub fn circle(x: f32, y: f32, r: f32) -> SkPath {
        Path::circle((x, y), r, None).into()
    }
    /// A rounded rectangle.
    pub fn r_rect(rrect: RRectArg, ccw: Option<bool>) -> SkPath {
        Path::rrect(rrect.0, direction(ccw)).into()
    }
    /// A line.
    pub fn line(p1: Pt, p2: Pt) -> SkPath {
        Path::line(p1.0, p2.0).into()
    }
    /// Lines between points.
    pub fn polygon(points: Points, close: bool) -> SkPath {
        Path::polygon(&points.0, close, None, None).into()
    }
    /// The outline of a path's stroke.
    pub fn stroke(
        path: &SkPath,
        width: Option<f32>,
        miter_limit: Option<f32>,
        precision: Option<f32>,
        join: Option<i32>,
        cap: Option<i32>,
    ) -> Result<Option<Instance<SkPath>>, String> {
        let stroke = Stroke {
            width: width.unwrap_or(1.0),
            miter: miter_limit.unwrap_or(4.0),
            precision: precision.unwrap_or(1.0),
            join: join.map(enums::join).transpose()?,
            cap: cap.map(enums::cap).transpose()?,
        };
        Ok(some(stroked(&path.0, &stroke)))
    }
    /// The part of a path between `start` and `end` (0 to 1).
    pub fn trim(path: &SkPath, start: f32, end: f32, is_complement: bool) -> Option<Instance<SkPath>> {
        some(trimmed(&path.0, start, end, is_complement))
    }
    /// A path with no overlaps.
    pub fn simplify(path: &SkPath) -> Option<Instance<SkPath>> {
        some(skia_safe::simplify(&path.0))
    }
    /// A path, dashed.
    pub fn dash(path: &SkPath, on: f32, off: f32, phase: f32) -> Option<Instance<SkPath>> {
        some(filtered(&path.0, dash_path_effect::new(&[on, off], phase)))
    }
    /// A path with the winding fill.
    pub fn as_winding(path: &SkPath) -> Option<Instance<SkPath>> {
        some(skia_safe::as_winding(&path.0))
    }
    /// Between two paths of the same verbs.
    pub fn interpolate_between(start: &SkPath, end: &SkPath, weight: f32) -> Option<Instance<SkPath>> {
        some(interpolated(&start.0, &end.0, weight))
    }
    /// Between this path (1) and `end` (0).
    pub fn interpolate(&self, end: Instance<SkPath>, weight: f32) -> Option<Instance<SkPath>> {
        some(self.0.interpolate(&end.with(|e| e.0.clone()), weight))
    }
    /// Whether two paths have the same verbs.
    pub fn is_interpolatable(&self, compare: Instance<SkPath>) -> bool {
        compare.with(|c| self.0.is_interpolatable(&c.0))
    }
    /// How many points it has.
    pub fn count_points(&self) -> usize {
        self.0.count_points()
    }
    /// The bounds of its curves.
    pub fn compute_tight_bounds(&self) -> RectOut {
        self.0.compute_tight_bounds().into()
    }
    /// The bounds of its points.
    pub fn get_bounds(&self) -> RectOut {
        (*self.0.bounds()).into()
    }
    /// Whether its fill holds a point.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.0.contains((x, y))
    }
    /// Another, the same.
    pub fn copy(&self) -> SkPath {
        self.0.clone().into()
    }
    /// Whether two paths are the same.
    pub fn equals(&self, other: Instance<SkPath>) -> bool {
        other.with(|o| self.0 == o.0)
    }
    /// What its fill holds.
    pub fn get_fill_type(&self) -> i32 {
        self.0.fill_type() as i32
    }
    /// Point `index`.
    pub fn get_point(&self, index: usize) -> Result<PointOut, String> {
        self.0.points().get(index).map(|p| (*p).into()).ok_or_else(|| format!("no point {index}"))
    }
    /// Its last point.
    pub fn get_last_pt(&self) -> PointOut {
        self.0.last_pt().unwrap_or_default().into()
    }
    /// Whether it has no verbs.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Whether it is drawn once.
    pub fn is_volatile(&self) -> bool {
        self.0.is_volatile()
    }
    /// It as an SVG string.
    #[allow(non_snake_case)]
    pub fn to_s_v_g_string(&self) -> String {
        parse_path::to_svg(&self.0)
    }
    /// Its verbs, each with its numbers.
    pub fn to_cmds(&self) -> Vec<Vec<f32>> {
        let mut iter = path::Iter::new(&self.0, false);
        let mut out = Vec::new();
        while let Some((verb, points)) = iter.next() {
            let (number, skip) = match verb {
                path::Verb::Move => (0.0, 0),
                path::Verb::Line => (1.0, 1),
                path::Verb::Quad => (2.0, 1),
                path::Verb::Conic => (3.0, 1),
                path::Verb::Cubic => (4.0, 1),
                path::Verb::Close => (5.0, points.len()),
                path::Verb::Done => break,
            };
            let mut cmd = vec![number];
            points.iter().skip(skip).for_each(|p| cmd.extend([p.x, p.y]));
            if verb == path::Verb::Conic {
                cmd.extend(iter.conic_weight());
            }
            out.push(cmd);
        }
        out
    }
}

/// `weight` 0 is `start`, as react-native-skia's `Path.Interpolate` has it.
fn interpolated(start: &Path, end: &Path, weight: f32) -> Option<Path> {
    end.interpolate(start, weight)
}

/// Builds a path.
#[class]
pub struct SkPathBuilder(PathBuilder);

type This = Instance<SkPathBuilder>;

fn with(this: This, f: impl FnOnce(&mut PathBuilder)) -> This {
    this.with_mut(|b| f(&mut b.0));
    this
}

fn arc_size(small: bool) -> path_builder::ArcSize {
    if small { path_builder::ArcSize::Small } else { path_builder::ArcSize::Large }
}

#[class]
impl SkPathBuilder {
    /// Of `path`, or of nothing.
    pub fn new(path: Option<Instance<SkPath>>) -> Self {
        Self(path.map_or_else(PathBuilder::new, |p| p.with(|p| PathBuilder::new_path(&p.0))))
    }
    /// Starts a contour.
    pub fn move_to(this: Instance<Self>, x: f32, y: f32) -> Instance<Self> {
        with(this, |b| done(b.move_to((x, y))))
    }
    /// Starts a contour, from the last point.
    pub fn r_move_to(this: Instance<Self>, x: f32, y: f32) -> Instance<Self> {
        with(this, |b| done(b.r_move_to((x, y))))
    }
    /// A line.
    pub fn line_to(this: Instance<Self>, x: f32, y: f32) -> Instance<Self> {
        with(this, |b| done(b.line_to((x, y))))
    }
    /// A line, from the last point.
    pub fn r_line_to(this: Instance<Self>, x: f32, y: f32) -> Instance<Self> {
        with(this, |b| done(b.r_line_to((x, y))))
    }
    /// A quadratic curve.
    pub fn quad_to(this: Instance<Self>, x1: f32, y1: f32, x2: f32, y2: f32) -> Instance<Self> {
        with(this, |b| done(b.quad_to((x1, y1), (x2, y2))))
    }
    /// A quadratic curve, from the last point.
    pub fn r_quad_to(this: Instance<Self>, x1: f32, y1: f32, x2: f32, y2: f32) -> Instance<Self> {
        with(this, |b| done(b.r_quad_to((x1, y1), (x2, y2))))
    }
    /// A conic curve.
    pub fn conic_to(this: Instance<Self>, x1: f32, y1: f32, x2: f32, y2: f32, w: f32) -> Instance<Self> {
        with(this, |b| done(b.conic_to((x1, y1), (x2, y2), w)))
    }
    /// A conic curve, from the last point.
    pub fn r_conic_to(this: Instance<Self>, x1: f32, y1: f32, x2: f32, y2: f32, w: f32) -> Instance<Self> {
        with(this, |b| done(b.r_conic_to((x1, y1), (x2, y2), w)))
    }
    /// A cubic curve.
    pub fn cubic_to(this: Instance<Self>, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> Instance<Self> {
        with(this, |b| done(b.cubic_to((x1, y1), (x2, y2), (x3, y3))))
    }
    /// A cubic curve, from the last point.
    pub fn r_cubic_to(this: Instance<Self>, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> Instance<Self> {
        with(this, |b| done(b.r_cubic_to((x1, y1), (x2, y2), (x3, y3))))
    }
    /// Closes the contour.
    pub fn close(this: Instance<Self>) -> Instance<Self> {
        with(this, |b| done(b.close()))
    }
    /// An arc of an oval, by degrees.
    pub fn arc_to_oval(
        this: Instance<Self>,
        oval: RectArg,
        start: f32,
        sweep: f32,
        force_move_to: bool,
    ) -> Instance<Self> {
        with(this, |b| done(b.arc_to(oval.0, start, sweep, force_move_to)))
    }
    /// An SVG arc.
    pub fn arc_to_rotated(
        this: Instance<Self>,
        rx: f32,
        ry: f32,
        rotate: f32,
        use_small_arc: bool,
        is_ccw: bool,
        x: f32,
        y: f32,
    ) -> Instance<Self> {
        with(this, |b| {
            done(b.arc_to_radius((rx, ry), rotate, arc_size(use_small_arc), direction(Some(is_ccw)), (x, y)))
        })
    }
    /// An SVG arc, from the last point.
    pub fn r_arc_to(
        this: Instance<Self>,
        rx: f32,
        ry: f32,
        rotate: f32,
        use_small_arc: bool,
        is_ccw: bool,
        dx: f32,
        dy: f32,
    ) -> Instance<Self> {
        with(this, |b| done(b.r_arc_to((rx, ry), rotate, arc_size(use_small_arc), direction(Some(is_ccw)), (dx, dy))))
    }
    /// An arc between two tangents.
    pub fn arc_to_tangent(this: Instance<Self>, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) -> Instance<Self> {
        with(this, |b| done(b.arc_to_tangent((x1, y1), (x2, y2), radius)))
    }
    /// A rectangle.
    pub fn add_rect(this: Instance<Self>, rect: RectArg, is_ccw: Option<bool>) -> Instance<Self> {
        with(this, |b| done(b.add_rect(rect.0, direction(is_ccw), None)))
    }
    /// An oval.
    pub fn add_oval(
        this: Instance<Self>,
        oval: RectArg,
        is_ccw: Option<bool>,
        start_index: Option<usize>,
    ) -> Instance<Self> {
        with(this, |b| done(b.add_oval(oval.0, direction(is_ccw), start_index.map(|i| i % 4))))
    }
    /// An arc of an oval, by degrees, as a contour.
    pub fn add_arc(this: Instance<Self>, oval: RectArg, start: f32, sweep: f32) -> Instance<Self> {
        with(this, |b| done(b.add_arc(oval.0, start, sweep)))
    }
    /// A rounded rectangle.
    pub fn add_r_rect(this: Instance<Self>, rrect: RRectArg, is_ccw: Option<bool>) -> Instance<Self> {
        with(this, |b| done(b.add_rrect(rrect.0, direction(is_ccw), None)))
    }
    /// A circle.
    pub fn add_circle(this: Instance<Self>, x: f32, y: f32, r: f32, is_ccw: Option<bool>) -> Instance<Self> {
        with(this, |b| done(b.add_circle((x, y), r, direction(is_ccw))))
    }
    /// Lines between points.
    pub fn add_poly(this: Instance<Self>, points: Points, close: bool) -> Instance<Self> {
        with(this, |b| done(b.add_polygon(&points.0, close)))
    }
    /// Another path.
    pub fn add_path(
        this: Instance<Self>,
        src: Instance<SkPath>,
        matrix: Option<MatrixArg>,
        extend: Option<bool>,
    ) -> Instance<Self> {
        let src = src.with(|p| p.0.clone());
        let mode = if extend == Some(true) { path::AddPathMode::Extend } else { path::AddPathMode::Append };
        let m = matrix.map_or_else(Matrix::default, |m| m.0);
        with(this, |b| b.add_path_with_transform(&src, &m, mode))
    }
    /// What its fill holds.
    pub fn set_fill_type(this: Instance<Self>, fill: i32) -> Result<Instance<Self>, String> {
        let fill = enums::fill_type(fill)?;
        Ok(with(this, |b| done(b.set_fill_type(fill))))
    }
    /// Whether the path is drawn once.
    pub fn set_is_volatile(this: Instance<Self>, is_volatile: bool) -> Instance<Self> {
        with(this, |b| done(b.set_is_volatile(is_volatile)))
    }
    /// Empties it.
    pub fn reset(this: Instance<Self>) -> Instance<Self> {
        with(this, |b| done(b.reset()))
    }
    /// Moves it.
    pub fn offset(this: Instance<Self>, dx: f32, dy: f32) -> Instance<Self> {
        with(this, |b| done(b.offset((dx, dy))))
    }
    /// Transforms it.
    pub fn transform(this: Instance<Self>, matrix: MatrixArg) -> Instance<Self> {
        with(this, |b| done(b.transform(&matrix.0)))
    }
    /// The bounds of its points.
    pub fn compute_bounds(&self) -> RectOut {
        self.0.compute_finite_bounds().unwrap_or_default().into()
    }
    /// Whether it has no verbs.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Its last point.
    pub fn get_last_pt(&self) -> PointOut {
        self.0.get_last_pt().unwrap_or_default().into()
    }
    /// How many points it has.
    pub fn count_points(&self) -> usize {
        self.0.count_points()
    }
    /// The path so far.
    pub fn build(&self) -> SkPath {
        self.0.snapshot().into()
    }
    /// The path, and empties the builder.
    pub fn detach(&mut self) -> SkPath {
        self.0.detach().into()
    }
}

/// The contours of a path, measured.
#[class]
pub struct SkContourMeasureIter(ContourMeasureIter);

#[class]
impl SkContourMeasureIter {
    /// Of `path`.
    pub fn new(path: &SkPath, force_closed: bool, res_scale: f32) -> Self {
        Self(ContourMeasureIter::new(&path.0, force_closed, res_scale))
    }
    /// The next contour.
    pub fn next(&mut self) -> Option<Instance<SkContourMeasure>> {
        Iterator::next(&mut self.0).map(|c| Instance::new(SkContourMeasure(c)))
    }
}

/// A contour, measured.
#[class]
pub struct SkContourMeasure(ContourMeasure);

#[class]
impl SkContourMeasure {
    /// The position and the tangent at a distance.
    pub fn get_pos_tan(&self, distance: f32) -> Result<Vec<PointOut>, String> {
        let (p, t): (Point, Point) = self.0.pos_tan(distance).ok_or_else(bad("position"))?;
        Ok(vec![p.into(), t.into()])
    }
    /// The part between two distances.
    pub fn get_segment(&self, start: f32, stop: f32, start_with_move_to: bool) -> Result<SkPath, String> {
        let mut out = PathBuilder::new();
        let made = self.0.get_segment(start, stop, &mut out, start_with_move_to);
        made.then(|| out.detach().into()).ok_or_else(bad("segment"))
    }
    /// Whether it is closed.
    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
    /// Its length.
    pub fn length(&self) -> f32 {
        self.0.length()
    }
}
