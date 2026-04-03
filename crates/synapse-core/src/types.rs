use serde::{Deserialize, Serialize};

/// 2D Vector for coordinates and dimensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    #[inline]
    pub fn add(&self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    #[inline]
    pub fn sub(&self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    #[inline]
    pub fn mul(&self, scalar: f32) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }

    #[inline]
    pub fn distance_squared(&self, other: Vec2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

/// Axis-Aligned Bounding Box for collision detection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABB {
    #[inline]
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size.mul(0.5);
        Self {
            min: center.sub(half_size),
            max: center.add(half_size),
        }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    #[inline]
    pub fn center(&self) -> Vec2 {
        Vec2 {
            x: (self.min.x + self.max.x) * 0.5,
            y: (self.min.y + self.max.y) * 0.5,
        }
    }

    /// Check if point is inside AABB
    #[inline]
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Check if two AABBs intersect
    #[inline]
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Check if AABB contains another AABB
    #[inline]
    pub fn contains_aabb(&self, other: &AABB) -> bool {
        other.min.x >= self.min.x
            && other.max.x <= self.max.x
            && other.min.y >= self.min.y
            && other.max.y <= self.max.y
    }
}

/// Node data stored in the Quadtree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub id: u32,
    pub bounds: AABB,
    pub label: String,
    pub data: Vec<u8>, // Runtime-specific data
}

impl NodeData {
    pub fn new(id: u32, bounds: AABB, label: String) -> Self {
        Self {
            id,
            bounds,
            label,
            data: Vec::new(),
        }
    }
}
