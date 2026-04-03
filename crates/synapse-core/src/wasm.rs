#![cfg(target_arch = "wasm32")]

use crate::{quadtree::Quadtree, types::{AABB, NodeData, Vec2}};
use wasm_bindgen::prelude::*;
use js_sys::Object;
use std::sync::OnceLock;
use parking_lot::Mutex;

static QUADTREE: OnceLock<Mutex<Option<Quadtree>>> = OnceLock::new();

fn get_quadtree() -> &'static Mutex<Option<Quadtree>> {
    QUADTREE.get_or_init(|| Mutex::new(None))
}

/// Initialize the quadtree with given bounds
#[wasm_bindgen]
pub fn init_quadtree(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> u32 {
    let bounds = AABB::new(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y));
    *get_quadtree().lock() = Some(Quadtree::new(bounds));
    0 // Success code
}

/// Insert a node into the quadtree
#[wasm_bindgen]
pub fn insert_node(
    id: u32,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    label: String,
) -> u32 {
    if let Some(ref qt) = &*get_quadtree().lock() {
        let bounds = AABB::new(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y));
        let node = NodeData::new(id, bounds, label);
        match qt.insert(node) {
            Ok(_) => 0,
            Err(_) => 1,
        }
    } else {
        2 // Quadtree not initialized
    }
}

/// Query nodes in a region - returns JSON array
#[wasm_bindgen]
pub fn query_nodes(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> String {
    if let Some(ref qt) = &*get_quadtree().lock() {
        let bounds = AABB::new(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y));
        let results = qt.query(&bounds);
        
        let mut json = String::from("[");
        for (i, node) in results.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!(
                r#"{{"id":{},"label":"{}","bounds":{{"min":{{"x":{},"y":{}}},"max":{{"x":{},"y":{}}}}}}}"#,
                node.id, node.label,
                node.bounds.min.x, node.bounds.min.y,
                node.bounds.max.x, node.bounds.max.y
            ));
        }
        json.push(']');
        json
    } else {
        "[]".to_string()
    }
}

/// Hit test at a point - returns node ID or 0
#[wasm_bindgen]
pub fn hit_test(x: f32, y: f32) -> u32 {
    if let Some(ref qt) = &*get_quadtree().lock() {
        qt.hit_test(Vec2::new(x, y))
            .map(|node| node.id)
            .unwrap_or(0)
    } else {
        0
    }
}

/// Hit test with radius - returns JSON array of node IDs
#[wasm_bindgen]
pub fn hit_test_radius(x: f32, y: f32, radius: f32) -> String {
    if let Some(ref qt) = &*get_quadtree().lock() {
        let results = qt.hit_test_radius(Vec2::new(x, y), radius);
        let ids: Vec<String> = results.iter().map(|n| n.id.to_string()).collect();
        format!("[{}]", ids.join(","))
    } else {
        "[]".to_string()
    }
}

/// Remove a node from the quadtree
#[wasm_bindgen]
pub fn remove_node(id: u32) -> u32 {
    if let Some(ref qt) = &*get_quadtree().lock() {
        match qt.remove(id) {
            Ok(_) => 0,
            Err(_) => 1,
        }
    } else {
        2
    }
}

/// Update a node's bounds
#[wasm_bindgen]
pub fn update_node(
    id: u32,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
) -> u32 {
    if let Some(ref qt) = &*get_quadtree().lock() {
        let new_bounds = AABB::new(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y));
        match qt.update(id, new_bounds) {
            Ok(_) => 0,
            Err(_) => 1,
        }
    } else {
        2
    }
}

/// Get all nodes - returns JSON array
#[wasm_bindgen]
pub fn get_all_nodes() -> String {
    if let Some(ref qt) = &*get_quadtree().lock() {
        let results = qt.get_all();
        
        let mut json = String::from("[");
        for (i, node) in results.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!(
                r#"{{"id":{},"label":"{}","bounds":{{"min":{{"x":{},"y":{}}},"max":{{"x":{},"y":{}}}}}}}"#,
                node.id, node.label,
                node.bounds.min.x, node.bounds.min.y,
                node.bounds.max.x, node.bounds.max.y
            ));
        }
        json.push(']');
        json
    } else {
        "[]".to_string()
    }
}

/// Clear all nodes
#[wasm_bindgen]
pub fn clear_quadtree() -> u32 {
    if let Some(ref qt) = &*get_quadtree().lock() {
        qt.clear();
        0
    } else {
        2
    }
}

/// Calculate bezier curve point
#[wasm_bindgen]
pub fn calculate_bezier(
    p0x: f32, p0y: f32,
    p1x: f32, p1y: f32,
    p2x: f32, p2y: f32,
    p3x: f32, p3y: f32,
    t: f32,
) -> js_sys::Float32Array {
    let p0 = Vec2::new(p0x, p0y);
    let p1 = Vec2::new(p1x, p1y);
    let p2 = Vec2::new(p2x, p2y);
    let p3 = Vec2::new(p3x, p3y);

    let point = crate::math::calculate_bezier_cubic(p0, p1, p2, p3, t);
    
    let data = vec![point.x, point.y];
    js_sys::Float32Array::from(&data[..])
}

/// Generate bezier path
#[wasm_bindgen]
pub fn generate_bezier_path(
    p0x: f32, p0y: f32,
    p1x: f32, p1y: f32,
    p2x: f32, p2y: f32,
    p3x: f32, p3y: f32,
    subdivisions: u32,
) -> String {
    let p0 = Vec2::new(p0x, p0y);
    let p1 = Vec2::new(p1x, p1y);
    let p2 = Vec2::new(p2x, p2y);
    let p3 = Vec2::new(p3x, p3y);

    crate::math::generate_bezier_path(p0, p1, p2, p3, subdivisions)
}
