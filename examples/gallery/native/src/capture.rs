//! Evidence only: this process's own window as a PNG (no Screen Recording permission is needed for one's own).
use std::ffi::{CStr, c_void};

use objc2::MainThreadMarker;
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use skia_safe::{AlphaType, ColorType, Data, EncodedImageFormat, ImageInfo, images};

unsafe extern "C" {
    fn dlsym(lib: *mut c_void, name: *const std::ffi::c_char) -> *mut c_void;
    fn CGImageGetWidth(i: *const c_void) -> usize;
    fn CGImageGetHeight(i: *const c_void) -> usize;
    fn CGImageGetBytesPerRow(i: *const c_void) -> usize;
    fn CGImageGetDataProvider(i: *const c_void) -> *const c_void;
    fn CGDataProviderCopyData(p: *const c_void) -> *const c_void;
    fn CFDataGetBytePtr(d: *const c_void) -> *const u8;
    fn CFDataGetLength(d: *const c_void) -> isize;
}

/// Captures the app's largest window into `path`.
pub(crate) fn capture(path: String) {
    dispatch2::DispatchQueue::main().exec_async(move || {
        eprintln!("CAPTURE {path}: {:?}", now(&path));
    });
}

fn now(path: &str) -> Result<(usize, usize), &'static str> {
    let app = objc2_app_kit::NSApplication::sharedApplication(MainThreadMarker::new().ok_or("thread")?);
    let window = app.windows().iter().filter(|w| w.isVisible()).max_by_key(|w| w.frame().size.width as i64).ok_or("no window")?;
    type Create = unsafe extern "C" fn(CGRect, u32, u32, u32) -> *const c_void;
    const NAME: &CStr = c"CGWindowListCreateImage";
    // SAFETY: CoreGraphics' function of this type (looked up, as the SDK no longer declares it).
    unsafe {
        let f = dlsym(-2isize as *mut c_void, NAME.as_ptr());
        if f.is_null() {
            return Err("no CGWindowListCreateImage");
        }
        let create: Create = std::mem::transmute(f);
        let null = CGRect::new(CGPoint::new(f64::INFINITY, f64::INFINITY), CGSize::new(0.0, 0.0));
        let image = create(null, 8, window.windowNumber() as u32, 1 | 16);
        if image.is_null() {
            return Err("no image");
        }
        let (w, h, stride) = (CGImageGetWidth(image), CGImageGetHeight(image), CGImageGetBytesPerRow(image));
        let data = CGDataProviderCopyData(CGImageGetDataProvider(image));
        let bytes = std::slice::from_raw_parts(CFDataGetBytePtr(data), CFDataGetLength(data) as usize);
        let info = ImageInfo::new((w as i32, h as i32), ColorType::BGRA8888, AlphaType::Premul, None);
        let image = images::raster_from_data(&info, Data::new_copy(bytes), stride).ok_or("raster")?;
        let png = image.encode(None::<&mut skia_safe::gpu::DirectContext>, EncodedImageFormat::PNG, None).ok_or("png")?;
        std::fs::write(path, png.as_bytes()).map_err(|_| "write")?;
        Ok((w, h))
    }
}
