/*!
 * Synapse-Ultra: High-Performance Spatial Engine (WASM)
 * Core library providing O(log N) spatial queries and sub-millisecond hit testing
 */
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod quadtree;
pub mod types;
pub mod math;
pub mod error;

pub use quadtree::{Quadtree, QuadtreeNode};
pub use types::{AABB, Vec2, NodeData};
pub use error::QuadtreeError;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

// Panic hook for debugging in WASM environment
#[cfg(target_arch = "wasm32")]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
