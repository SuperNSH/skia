//! Uses nsh's crates at 0.7 (AGENTS.md "Toolchain").
use nsh_core::prelude::*;

#[native_module("skia-headless/native")]
fn native(_registry: &mut Registry) {}
