//! Linux: `skia.canvas`, a GtkGLArea that Skia Ganesh (GL) draws into on
//! GTK's thread.

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ffi::{CString, c_char, c_void},
    rc::Rc,
    sync::atomic::Ordering,
};

use nsh_core::{JsonValue, prelude::*};
use nsh_platform_linux::{
    ComponentContext, Components, NativeComponent,
    gtk4::{self as gtk, glib, prelude::*},
};
use skia_safe::{
    Color4f, ColorType,
    gpu::{self, DirectContext, SurfaceOrigin, backend_render_targets, direct_contexts, gl},
};

thread_local! {
    static AREAS: RefCell<HashMap<u32, gtk::GLArea>> = RefCell::default();
}

/// Has the canvas's area draw its last picture, on GTK's main loop.
pub(crate) fn redraw(id: u32) {
    // A source of the main loop's, never run on this thread (`invoke` may be, before the loop runs).
    glib::idle_add_full(glib::Priority::DEFAULT, move || {
        AREAS.with_borrow(|a| {
            if let Some(area) = a.get(&id) {
                area.queue_render();
            }
        });
        glib::ControlFlow::Break
    });
}

unsafe extern "C" {
    fn dlopen(name: *const c_char, flags: i32) -> *mut c_void;
    fn dlsym(lib: *mut c_void, name: *const c_char) -> *mut c_void;
}

/// A function of GL or GLES, whichever GDK's context is (GDK's are EGL's, or GLX's on X11 without EGL).
fn load(name: &str) -> *const c_void {
    type Get = unsafe extern "C" fn(*const c_char) -> *const c_void;
    let Ok(function) = CString::new(name) else { return std::ptr::null() };
    for (lib, get) in [(c"libEGL.so.1", c"eglGetProcAddress"), (c"libGL.so.1", c"glXGetProcAddressARB")] {
        // SAFETY: the libraries' functions have this type; what they are asked for is a string.
        let found = unsafe {
            let lib = dlopen(lib.as_ptr(), 1);
            if lib.is_null() {
                continue;
            }
            let get = dlsym(lib, get.as_ptr());
            if get.is_null() {
                continue;
            }
            std::mem::transmute::<*mut c_void, Get>(get)(function.as_ptr())
        };
        if !found.is_null() {
            return found;
        }
    }
    std::ptr::null()
}

fn gl_int(name: u32) -> Option<i32> {
    type Get = unsafe extern "C" fn(u32, *mut i32);
    let get = load("glGetIntegerv");
    if get.is_null() {
        return None;
    }
    let mut v = 0;
    // SAFETY: a context is current; glGetIntegerv has this type.
    unsafe { std::mem::transmute::<*const c_void, Get>(get)(name, &mut v) };
    Some(v)
}

const GL_DRAW_FRAMEBUFFER_BINDING: u32 = 0x8CA6;

fn draw(area: &gtk::GLArea, context: &mut Option<DirectContext>, id: u32) -> Option<()> {
    let picture = super::picture(id);
    if context.is_none() {
        *context = direct_contexts::make_gl(gl::Interface::new_load_with(load)?, None);
        if context.is_none() {
            eprintln!("nsh-skia: Skia made no GL context: canvases draw nothing");
        }
    }
    let context = context.as_mut()?;
    context.reset(None); // GTK changed the state since.
    let scale = area.scale_factor();
    let size = (area.width() * scale, area.height() * scale);
    let info = gl::FramebufferInfo {
        fboid: gl_int(GL_DRAW_FRAMEBUFFER_BINDING)? as u32,
        format: gl::Format::RGBA8.into(),
        ..Default::default()
    };
    let target = backend_render_targets::make_gl(size, 0, 8, info);
    let mut surface = gpu::surfaces::wrap_backend_render_target(
        context,
        &target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )?;
    let canvas = surface.canvas();
    canvas.clear(Color4f::new(0.0, 0.0, 0.0, 0.0));
    if let Some(picture) = picture {
        canvas.scale((scale as f32, scale as f32));
        canvas.draw_picture(&picture, None, None);
        super::FRAMES.fetch_add(1, Ordering::Relaxed);
    }
    context.flush_and_submit().ok()?;
    Some(())
}

struct CanvasComponent {
    area: gtk::GLArea,
    id: Rc<Cell<u32>>,
}

impl CanvasComponent {
    fn new() -> Self {
        let area =
            gtk::GLArea::builder().hexpand(true).vexpand(true).auto_render(false).has_stencil_buffer(true).build();
        let id = Rc::new(Cell::new(0));
        let context: Rc<RefCell<Option<DirectContext>>> = Rc::default();
        let (shown, drawn) = (id.clone(), context.clone());
        area.connect_render(move |area, _| {
            draw(area, &mut drawn.borrow_mut(), shown.get());
            glib::Propagation::Stop
        });
        // Skia's GL objects go with GTK's context.
        area.connect_unrealize(move |area| {
            area.make_current();
            if let Some(mut context) = context.take() {
                context.release_resources_and_abandon();
            }
        });
        area.connect_realize(|area| {
            if let Some(error) = area.error() {
                eprintln!("nsh-skia: no GL: {error}");
            }
        });
        Self { area, id }
    }
}

impl NativeComponent for CanvasComponent {
    fn widget(&self) -> gtk::Widget {
        self.area.clone().upcast()
    }
    fn set_prop(&mut self, name: &str, value: Option<&JsonValue>) {
        if name == "canvas" {
            let id = value.and_then(JsonValue::as_u64).unwrap_or(0) as u32;
            let old = self.id.replace(id);
            AREAS.with_borrow_mut(|a| {
                a.remove(&old);
                a.insert(id, self.area.clone());
            });
            self.area.queue_render();
        }
    }
}

impl Drop for CanvasComponent {
    fn drop(&mut self) {
        AREAS.with_borrow_mut(|a| a.remove(&self.id.get()));
    }
}

#[native_components(linux)]
fn components(components: &Components) {
    components.register(
        "skia.canvas",
        Rc::new(|_: ComponentContext| -> Box<dyn NativeComponent> { Box::new(CanvasComponent::new()) }),
    );
}
