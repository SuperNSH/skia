//! Uses nsh's crates at 0.7 (AGENTS.md "Toolchain"), and the gallery's evidence: CPU time, and the
//! app's own window as a PNG.
use nsh_core::prelude::*;

#[cfg(target_os = "macos")]
mod capture;

/// What the gallery measures.
#[module]
pub struct GalleryHost;

#[module]
impl GalleryHost {
    /// The process's CPU time (user and system) in milliseconds.
    pub fn cpu() -> f64 {
        #[repr(C)]
        struct Timespec(i64, i64);
        unsafe extern "C" {
            fn clock_gettime(id: i32, t: *mut Timespec) -> i32;
        }
        let mut t = Timespec(0, 0);
        // SAFETY: t is valid; CLOCK_PROCESS_CPUTIME_ID.
        unsafe { clock_gettime(if cfg!(target_os = "macos") { 12 } else { 2 }, &mut t) };
        t.0 as f64 * 1000.0 + t.1 as f64 / 1e6
    }
    /// Saves the app's own window as `name` in the directory `NSH_SKIA_EVIDENCE` names, if it does (macOS).
    pub fn capture(name: String) {
        if let Ok(dir) = std::env::var("NSH_SKIA_EVIDENCE") {
            let path = format!("{dir}/{name}");
            #[cfg(target_os = "macos")]
            capture::capture(path);
            #[cfg(not(target_os = "macos"))]
            let _ = path;
        }
    }
}

#[native_module("skia-gallery/native")]
fn native(registry: &mut Registry) {
    registry.module::<GalleryHost>();
}
