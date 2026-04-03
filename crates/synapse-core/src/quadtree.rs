use crate::types::{AABB, NodeData, Vec2};
use crate::error::{QuadtreeError, Result};
use smallvec::SmallVec;
use parking_lot::RwLock;
use std::collections::HashMap;

const MAX_NODES_PER_CELL: usize = 8;
const MAX_DEPTH: usize = 16;

/// Z-order (Morton) code for linear quadtree indexing
#[inline]
fn interleave(x: u32, y: u32) -> u64 {
    let mut z = 0u64;
    let mut x_val = x;
    let mut y_val = y;
    
    for i in 0..16 {
        z |= ((x_val & (1 << i)) as u64) << i | ((y_val & (1 << i)) as u64) << (i + 1);
    }
    z
}

/// Quadtree node in the spatial index
pub struct QuadtreeNode {
    pub bounds: AABB,
    pub depth: usize,
    pub morton_code: u64,
    pub nodes: SmallVec<[NodeData; MAX_NODES_PER_CELL]>,
    pub children: [Option<Box<QuadtreeNode>>; 4], // NE, NW, SW, SE
}

impl QuadtreeNode {
    fn new(bounds: AABB, depth: usize, morton_code: u64) -> Self {
        Self {
            bounds,
            depth,
            morton_code,
            nodes: SmallVec::new(),
            children: [None, None, None, None],
        }
    }

    /// Check if node's bounds intersect with query bounds
    #[inline]
    fn intersects(&self, query: &AABB) -> bool {
        self.bounds.intersects(query)
    }

    /// Get child index based on point position
    #[inline]
    fn get_child_index(&self, point: Vec2) -> usize {
        let center = self.bounds.center();
        let right = point.x > center.x;
        let bottom = point.y > center.y;
        
        match (right, bottom) {
            (true, false) => 0,  // NE
            (false, false) => 1, // NW
            (false, true) => 2,  // SW
            (true, true) => 3,   // SE
        }
    }

    /// Subdivide this node into 4 children
    fn subdivide(&mut self, node_to_insert: NodeData) -> Result<()> {
        if self.depth >= MAX_DEPTH {
            return Err(QuadtreeError::InvalidOperation(
                "Max quadtree depth exceeded".to_string(),
            ));
        }

        let center = self.bounds.center();
        let half_width = self.bounds.width() * 0.5;
        let half_height = self.bounds.height() * 0.5;

        let bounds = [
            // NE
            AABB::new(
                Vec2::new(center.x, center.y - half_height),
                Vec2::new(center.x + half_width, center.y),
            ),
            // NW
            AABB::new(
                Vec2::new(center.x - half_width, center.y - half_height),
                Vec2::new(center.x, center.y),
            ),
            // SW
            AABB::new(
                Vec2::new(center.x - half_width, center.y),
                Vec2::new(center.x, center.y + half_height),
            ),
            // SE
            AABB::new(
                Vec2::new(center.x, center.y),
                Vec2::new(center.x + half_width, center.y + half_height),
            ),
        ];

        for (i, &bound) in bounds.iter().enumerate() {
            self.children[i] = Some(Box::new(QuadtreeNode::new(
                bound,
                self.depth + 1,
                interleave(
                    bound.min.x as u32,
                    bound.min.y as u32,
                ),
            )));
        }

        // Redistribute existing nodes
        let nodes_to_redistribute: Vec<NodeData> = self.nodes.drain(..).collect();
        for node in nodes_to_redistribute {
            let idx = self.get_child_index(node.bounds.center());
            if let Some(ref mut child) = self.children[idx] {
                let _ = child.insert(node);
            }
        }

        // Insert the new node
        let idx = self.get_child_index(node_to_insert.bounds.center());
        if let Some(ref mut child) = self.children[idx] {
            child.insert(node_to_insert)?;
        }

        Ok(())
    }

    /// Insert a node into this tree
    pub fn insert(&mut self, node: NodeData) -> Result<()> {
        if !self.bounds.intersects(&node.bounds) {
            return Err(QuadtreeError::InvalidBounds);
        }

        if self.children[0].is_none() {
            // Leaf node
            if self.nodes.len() < MAX_NODES_PER_CELL {
                self.nodes.push(node);
                Ok(())
            } else {
                // Split and redistribute
                self.subdivide(node)
            }
        } else {
            // Internal node - distribute to children
            let idx = self.get_child_index(node.bounds.center());
            if let Some(ref mut child) = self.children[idx] {
                child.insert(node)
            } else {
                Err(QuadtreeError::InvalidOperation(
                    "Child node missing".to_string(),
                ))
            }
        }
    }

    /// Query nodes that intersect with given bounds
    pub fn query(&self, query_bounds: &AABB, results: &mut Vec<NodeData>) {
        if !self.intersects(query_bounds) {
            return;
        }

        // Check nodes at this level
        for node in &self.nodes {
            if node.bounds.intersects(query_bounds) {
                results.push(node.clone());
            }
        }

        // Recurse to children
        for child_opt in &self.children {
            if let Some(ref child) = child_opt {
                child.query(query_bounds, results);
            }
        }
    }

    /// Find a single node by ID
    pub fn find(&self, id: u32) -> Option<NodeData> {
        for node in &self.nodes {
            if node.id == id {
                return Some(node.clone());
            }
        }

        for child_opt in &self.children {
            if let Some(ref child) = child_opt {
                if let Some(found) = child.find(id) {
                    return Some(found);
                }
            }
        }

        None
    }

    /// Remove a node by ID
    pub fn remove(&mut self, id: u32) -> Result<()> {
        self.nodes.retain(|n| n.id != id);

        for child_opt in &mut self.children {
            if let Some(ref mut child) = child_opt {
                let _ = child.remove(id);
            }
        }

        Ok(())
    }
}

/// High-performance spatial index for the node editor
pub struct Quadtree {
    root: RwLock<QuadtreeNode>,
    node_cache: RwLock<HashMap<u32, NodeData>>,
}

impl Quadtree {
    pub fn new(bounds: AABB) -> Self {
        Self {
            root: RwLock::new(QuadtreeNode::new(bounds, 0, 0)),
            node_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Insert a node into the quadtree
    pub fn insert(&self, node: NodeData) -> Result<()> {
        let id = node.id;
        self.root.write().insert(node.clone())?;
        self.node_cache.write().insert(id, node);
        Ok(())
    }

    /// Query all nodes intersecting with bounds - O(log N) operation
    pub fn query(&self, bounds: &AABB) -> Vec<NodeData> {
        let mut results = Vec::new();
        self.root.read().query(bounds, &mut results);
        results
    }

    /// Hit test: find node at exact point - sub-millisecond
    pub fn hit_test(&self, point: Vec2) -> Option<NodeData> {
        let query_bounds = AABB::new(point, point);
        let results = self.query(&query_bounds);
        
        // Return the topmost (smallest) node if multiple overlap
        results.into_iter().min_by(|a, b| {
            a.bounds.width().partial_cmp(&b.bounds.width()).unwrap()
        })
    }

    /// Hit test with radius for fuzzy selection
    pub fn hit_test_radius(&self, point: Vec2, radius: f32) -> Vec<NodeData> {
        let query_bounds = AABB::new(
            Vec2::new(point.x - radius, point.y - radius),
            Vec2::new(point.x + radius, point.y + radius),
        );
        self.query(&query_bounds)
    }

    /// Get all nodes (for debugging or serialization)
    pub fn get_all(&self) -> Vec<NodeData> {
        self.node_cache.read().values().cloned().collect()
    }

    /// Find node by ID - O(1) with cache
    pub fn find(&self, id: u32) -> Option<NodeData> {
        self.node_cache.read().get(&id).cloned()
    }

    /// Remove a node from the quadtree
    pub fn remove(&self, id: u32) -> Result<()> {
        self.root.write().remove(id)?;
        self.node_cache.write().remove(&id);
        Ok(())
    }

    /// Update a node's position/bounds
    pub fn update(&self, id: u32, new_bounds: AABB) -> Result<()> {
        // Remove old, insert new
        if let Some(mut node) = self.node_cache.write().get(&id).cloned() {
            self.remove(id)?;
            node.bounds = new_bounds;
            self.insert(node)?;
        }
        Ok(())
    }

    /// Clear all nodes
    pub fn clear(&self) {
        self.root.write().nodes.clear();
        self.node_cache.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadtree_insert_query() {
        let bounds = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let qt = Quadtree::new(bounds);

        let node1 = NodeData::new(1, AABB::new(Vec2::new(10.0, 10.0), Vec2::new(20.0, 20.0)), "node1".to_string());
        qt.insert(node1).unwrap();

        let results = qt.query(&AABB::new(Vec2::new(15.0, 15.0), Vec2::new(25.0, 25.0)));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_hit_test() {
        let bounds = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let qt = Quadtree::new(bounds);

        let node = NodeData::new(1, AABB::new(Vec2::new(10.0, 10.0), Vec2::new(20.0, 20.0)), "node".to_string());
        qt.insert(node).unwrap();

        let hit = qt.hit_test(Vec2::new(15.0, 15.0));
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().id, 1);
    }
}
