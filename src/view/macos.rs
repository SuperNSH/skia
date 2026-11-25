//! macOS: `skia.canvas`, an NSView over a CAMetalLayer that Skia Ganesh
//! draws into, on a thread of its own (`nextDrawable` waits for the display).

use std::{
    cell::Cell,
    collections::{BTreeSet, HashMap},
    rc::Rc,
    sync::{Condvar, Mutex, OnceLock, atomic::Ordering},
};

use nsh_core::{JsonValue, prelude::*};
use nsh_platform_macos::{ComponentContext, Components, NativeComponent};
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
    sel,
};
use objc2_app_kit::{NSResponder, NSView, NSWindowDidChangeOcclusionStateNotification, NSWindowOcclusionState};
use objc2_core_foundation::CGSize;
use objc2_core_graphics::{CGColorSpace, kCGColorSpaceSRGB};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSObject, NSObjectProtocol, NSRect, NSSize};
use objc2_metal::{MTLCommandBuffer, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice, MTLPixelFormat};
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
use skia_safe::{
    Color4f, ColorType,
    gpu::{self, DirectContext, SurfaceOrigin, backend_render_targets, direct_contexts, mtl},
};

struct Layer(Retained<CAMetalLayer>);
// SAFETY: CAMetalLayer's drawables may be taken on another thread than the
// one that resizes it.
unsafe impl Send for Layer {}

#[derive(Default)]
struct Work {
    /// The layers of the canvases on screen, with their pixels per point.
    layers: HashMap<u32, (Layer, f64)>,
    /// The canvases to draw.
    dirty: BTreeSet<u32>,
}

fn work() -> &'static (Mutex<Work>, Condvar) {
    static WORK: OnceLock<(Mutex<Work>, Condvar)> = OnceLock::new();
    WORK.get_or_init(|| {
        std::thread::Builder::new().name("nsh-skia".into()).spawn(render_loop).expect("a thread");
        Default::default()
    })
}

/// Has the canvas's view draw its last picture.
pub(crate) fn redraw(id: u32) {
    let (lock, ready) = work();
    lock.lock().unwrap_or_else(|e| e.into_inner()).dirty.insert(id);
    ready.notify_one();
}

/// The next canvas to draw: one that is dirty and on screen.
fn next(work: &mut Work) -> Option<(u32, Retained<CAMetalLayer>, f64)> {
    // A canvas without a view yet is drawn when it gets one.
    while let Some(id) = work.dirty.pop_first() {
        if let Some((layer, scale)) = work.layers.get(&id) {
            return Some((id, layer.0.clone(), *scale));
        }
    }
    None
}

fn render_loop() {
    let mut gpu = None;
    loop {
        let (lock, ready) = work();
        let (id, layer, scale) = {
            let mut work = lock.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                if let Some(found) = next(&mut work) {
                    break found;
                }
                work = ready.wait(work).unwrap_or_else(|e| e.into_inner());
            }
        };
        // Made when the first canvas is on screen: headless tests need none.
        if gpu.is_none() {
            gpu = Gpu::new();
        }
        match &mut gpu {
            Some(gpu) => objc2::rc::autoreleasepool(|_| gpu.draw(id, &layer, scale)),
            None => {
                eprintln!("nsh-skia: no Metal device: canvases draw nothing");
                return;
            }
        };
    }
}

fn device() -> Option<&'static ProtocolObject<dyn MTLDevice>> {
    static DEVICE: OnceLock<Option<Retained<ProtocolObject<dyn MTLDevice>>>> = OnceLock::new();
    DEVICE.get_or_init(|| MTLCreateSystemDefaultDevice()).as_deref()
}

struct Gpu {
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    context: DirectContext,
}

impl Gpu {
    fn new() -> Option<Self> {
        let device = device()?;
        let queue = device.newCommandQueue()?;
        // SAFETY: live objects, which Skia retains.
        let backend = unsafe {
            mtl::BackendContext::new(std::ptr::from_ref(device) as mtl::Handle, Retained::as_ptr(&queue) as mtl::Handle)
        };
        Some(Self { context: direct_contexts::make_metal(&backend, None)?, queue })
    }

    /// Draws the canvas's last picture into its layer.
    fn draw(&mut self, id: u32, layer: &CAMetalLayer, scale: f64) -> Option<()> {
        let drawable = layer.nextDrawable()?;
        // After the wait for a drawable: the latest picture there is.
        let picture = super::picture(id)?;
        let texture = drawable.texture();
        let size =
            (objc2_metal::MTLTexture::width(&*texture) as i32, objc2_metal::MTLTexture::height(&*texture) as i32);
        // SAFETY: the texture lives until the command buffer presenting it is done.
        let info = unsafe { mtl::TextureInfo::new(Retained::as_ptr(&texture) as mtl::Handle) };
        let target = backend_render_targets::make_mtl(size, &info);
        let mut surface = gpu::surfaces::wrap_backend_render_target(
            &mut self.context,
            &target,
            SurfaceOrigin::TopLeft,
            ColorType::BGRA8888,
            None,
            None,
        )?;
        let canvas = surface.canvas();
        canvas.clear(Color4f::new(0.0, 0.0, 0.0, 0.0));
        canvas.scale((scale as f32, scale as f32));
        canvas.draw_picture(&picture, None, None);
        self.context.flush_and_submit().ok()?;
        drop(surface);
        let commands = self.queue.commandBuffer()?;
        commands.presentDrawable(ProtocolObject::from_ref(&*drawable));
        commands.commit();
        super::FRAMES.fetch_add(1, Ordering::Relaxed);
        Some(())
    }
}

#[derive(Default)]
struct Ivars {
    id: Cell<u32>,
}

define_class!(
    #[unsafe(super(NSView, NSResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "NshSkiaCanvasView"]
    #[ivars = Ivars]
    struct CanvasView;

    unsafe impl NSObjectProtocol for CanvasView {}

    impl CanvasView {
        #[unsafe(method(setFrameSize:))]
        fn set_frame_size(&self, size: NSSize) {
            // SAFETY: NSView's own.
            let _: () = unsafe { msg_send![super(self), setFrameSize: size] };
            self.changed();
        }
        #[unsafe(method(viewDidMoveToWindow))]
        fn did_move_to_window(&self) {
            // SAFETY: NSView's own.
            let _: () = unsafe { msg_send![super(self), viewDidMoveToWindow] };
            let center = NSNotificationCenter::defaultCenter();
            // SAFETY: this view's method, which takes a notification; the center forgets a view that goes.
            unsafe {
                let name = NSWindowDidChangeOcclusionStateNotification;
                center.removeObserver_name_object(self, Some(name), None);
                if let Some(window) = self.window() {
                    center.addObserver_selector_name_object(
                        self,
                        sel!(occlusionChanged:),
                        Some(name),
                        Some(&window),
                    );
                }
            }
            self.changed();
        }
        #[unsafe(method(occlusionChanged:))]
        fn occlusion_changed(&self, _: &NSNotification) {
            self.changed();
        }
        #[unsafe(method(viewDidChangeBackingProperties))]
        fn did_change_backing(&self) {
            // SAFETY: NSView's own.
            let _: () = unsafe { msg_send![super(self), viewDidChangeBackingProperties] };
            self.changed();
        }
    }
);

impl CanvasView {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this: Allocated<Self> = Self::alloc(mtm);
        let this = this.set_ivars(Ivars::default());
        // SAFETY: NSView's designated initializer.
        let view: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: NSRect::ZERO] };
        let layer = CAMetalLayer::new();
        layer.setDevice(device());
        layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        layer.setFramebufferOnly(false);
        layer.setOpaque(false);
        // SAFETY: a constant of CoreGraphics.
        let srgb = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }));
        layer.setColorspace(srgb.as_deref());
        view.setLayer(Some(&layer));
        view.setWantsLayer(true);
        view
    }

    /// The size, the window, its scale or whether it can be seen changed: draws again.
    fn changed(&self) {
        let Some(layer) = self.layer().and_then(|l| l.downcast::<CAMetalLayer>().ok()) else { return };
        let id = self.ivars().id.get();
        let size = self.frame().size;
        let window = self.window();
        // `nextDrawable` waits up to a second for a window that is covered: such a canvas is not drawn.
        let visible = window.as_ref().is_some_and(|w| w.occlusionState().contains(NSWindowOcclusionState::Visible));
        let shown = id != 0 && visible && size.width >= 1.0 && size.height >= 1.0;
        let (lock, ready) = work();
        let mut work = lock.lock().unwrap_or_else(|e| e.into_inner());
        if !shown {
            work.layers.remove(&id);
            return;
        }
        let scale = window.map_or(1.0, |w| w.backingScaleFactor());
        layer.setContentsScale(scale);
        layer.setDrawableSize(CGSize::new((size.width * scale).round(), (size.height * scale).round()));
        work.layers.insert(id, (Layer(layer), scale));
        work.dirty.insert(id);
        ready.notify_one();
    }

    fn set_id(&self, id: u32) {
        let old = self.ivars().id.replace(id);
        if old != id {
            forget(old);
            self.changed();
        }
    }
}

fn forget(id: u32) {
    work().0.lock().unwrap_or_else(|e| e.into_inner()).layers.remove(&id);
}

struct CanvasComponent(Retained<CanvasView>);

impl NativeComponent for CanvasComponent {
    fn view(&self) -> Retained<NSView> {
        Retained::into_super(self.0.clone())
    }
    fn set_prop(&mut self, name: &str, value: Option<&JsonValue>) {
        if name == "canvas" {
            self.0.set_id(value.and_then(JsonValue::as_u64).unwrap_or(0) as u32);
        }
    }
}

impl Drop for CanvasComponent {
    fn drop(&mut self) {
        forget(self.0.ivars().id.get());
    }
}

#[native_components(macos)]
fn components(components: &Components) {
    components.register(
        "skia.canvas",
        Rc::new(|_: ComponentContext| -> Box<dyn NativeComponent> {
            Box::new(CanvasComponent(CanvasView::new(MainThreadMarker::new().expect("main thread"))))
        }),
    );
}
