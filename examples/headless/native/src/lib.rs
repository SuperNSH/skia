//! Pins nsh's crates at the nightly's commit (AGENTS.md "Toolchain").
use nsh_core::prelude::*;

#[native_module("skia-headless/native")]
fn native(_registry: &mut Registry) {}
