//! Canvas views: what shows a picture on screen.
//!
//! The JS thread records a picture and hands it over with [`show`]; the
//! platform's view draws the last one on its own thread. A canvas goes by
//! an id the JS side gets from [`SkiaHost::allocate`].

#[cfg(target_os = "linux")]
pub(crate) mod linux;
#[cfg(target_os = "macos")]
pub(crate) mod macos;

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

use nsh_core::{element::ImageSrc, geometry::Size, prelude::*};
use skia_safe::Picture;

/// The last picture of each canvas.
static PICTURES: Mutex<Option<HashMap<u32, Picture>>> = Mutex::new(None);

/// The canvas's last picture.
#[allow(dead_code, reason = "only platforms with a GPU view draw it")]
pub(crate) fn picture(id: u32) -> Option<Picture> {
    PICTURES.lock().unwrap_or_else(|e| e.into_inner()).as_ref()?.get(&id).cloned()
}

/// Makes `picture` the canvas's, and has its view draw it.
pub(crate) fn show(id: u32, picture: Picture) {
    PICTURES.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(id, picture);
    #[cfg(target_os = "macos")]
    macos::redraw(id);
    #[cfg(target_os = "linux")]
    linux::redraw(id);
}

/// What canvases need of the platform.
#[module]
pub struct SkiaHost;

#[module]
impl SkiaHost {
    /// A new canvas's id.
    pub fn allocate() -> u32 {
        static NEXT: AtomicU32 = AtomicU32::new(1);
        NEXT.fetch_add(1, Ordering::Relaxed)
    }
    /// Forgets a canvas that is gone.
    pub fn release(id: u32) {
        if let Some(p) = PICTURES.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
            p.remove(&id);
        }
    }
    /// Whether this platform has a GPU canvas view (else `SkiaImage` shows pixels drawn in memory).
    pub fn has_gpu_view() -> bool {
        cfg!(any(target_os = "macos", target_os = "linux"))
    }
    /// How many frames canvas views have put on screen, in this process.
    pub fn frames() -> u32 {
        FRAMES.load(Ordering::Relaxed)
    }
}

/// The frames drawn by GPU views.
pub(crate) static FRAMES: AtomicU32 = AtomicU32::new(0);

/// The props of [`SkiaImage`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct SkiaImageProps {
    /// The canvas's id.
    pub canvas: Prop<u32>,
    /// Changes each frame.
    pub frame: Prop<u32>,
    /// The canvas's width.
    pub width: Prop<f64>,
    /// The canvas's height.
    pub height: Prop<f64>,
    /// Pixels per point.
    pub scale: Prop<f64>,
}

/// Shows a canvas's last picture, drawn in memory, with NSH's own image (where there is no GPU view).
#[component]
pub fn SkiaImage(props: SkiaImageProps) -> impl IntoElement {
    let src = move || {
        props.frame.get();
        let (w, h, scale) = (props.width.get() as f32, props.height.get() as f32, props.scale.get() as f32);
        let (pw, ph, pixels) = pixels(props.canvas.get(), w, h, scale).unwrap_or((1, 1, vec![0; 4]));
        ImageSrc::rgba(pw, ph, Pixels(Arc::new(pixels))).size(Size::new(px(w), px(h)))
    };
    image(src)
}

fn pixels(id: u32, w: f32, h: f32, scale: f32) -> Option<(u32, u32, Vec<u8>)> {
    let picture = picture(id)?;
    let (pw, ph) = ((w * scale).round().max(1.0) as i32, (h * scale).round().max(1.0) as i32);
    let mut surface = crate::canvas::raster(pw, ph).ok()?;
    surface.canvas().scale((scale, scale)).draw_picture(&picture, None, None);
    let pixels = crate::objects::read_pixels(&surface.image_snapshot()).ok()?;
    Some((pw as u32, ph as u32, pixels))
}

struct Pixels(Arc<Vec<u8>>);

impl AsRef<[u8]> for Pixels {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
