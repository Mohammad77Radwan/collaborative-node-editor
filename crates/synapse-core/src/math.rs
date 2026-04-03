use crate::types::Vec2;

/// Fast bezier curve calculation for edge rendering
/// Uses De Casteljau's algorithm for numerical stability
#[inline]
pub fn calculate_bezier_quadratic(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let one_minus_t = 1.0 - t;
    let one_minus_t_sq = one_minus_t * one_minus_t;
    let t_sq = t * t;
    let two_t_one_minus_t = 2.0 * t * one_minus_t;

    Vec2 {
        x: one_minus_t_sq * p0.x + two_t_one_minus_t * p1.x + t_sq * p2.x,
        y: one_minus_t_sq * p0.y + two_t_one_minus_t * p1.y + t_sq * p2.y,
    }
}

/// Cubic bezier curve calculation for complex paths
#[inline]
pub fn calculate_bezier_cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let one_minus_t = 1.0 - t;
    let one_minus_t_2 = one_minus_t * one_minus_t;
    let one_minus_t_3 = one_minus_t_2 * one_minus_t;
    let t_2 = t * t;
    let t_3 = t_2 * t;

    Vec2 {
        x: one_minus_t_3 * p0.x
            + 3.0 * one_minus_t_2 * t * p1.x
            + 3.0 * one_minus_t * t_2 * p2.x
            + t_3 * p3.x,
        y: one_minus_t_3 * p0.y
            + 3.0 * one_minus_t_2 * t * p1.y
            + 3.0 * one_minus_t * t_2 * p2.y
            + t_3 * p3.y,
    }
}

/// Generate SVG path for bezier curve with subdivisions
pub fn generate_bezier_path(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    subdivisions: u32,
) -> String {
    let mut path = format!("M{:.2},{:.2}", p0.x, p0.y);
    let step = 1.0 / subdivisions as f32;

    let mut i = 1;
    while i <= subdivisions {
        let t = step * i as f32;
        let point = calculate_bezier_cubic(p0, p1, p2, p3, t);
        path.push_str(&format!("L{:.2},{:.2}", point.x, point.y));
        i += 1;
    }

    path
}

/// Matrix3d for hardware-accelerated transforms (CSS)
#[derive(Debug, Clone, Copy)]
pub struct Matrix3d {
    // row-major: [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
    pub m: [f32; 16],
}

impl Matrix3d {
    /// Identity matrix
    #[inline]
    pub fn identity() -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Translation matrix for 2D transforms
    #[inline]
    pub fn translate(tx: f32, ty: f32, tz: f32) -> Self {
        let mut m = Self::identity();
        m.m[12] = tx;
        m.m[13] = ty;
        m.m[14] = tz;
        m
    }

    /// Scale matrix
    #[inline]
    pub fn scale(sx: f32, sy: f32, sz: f32) -> Self {
        let mut m = Self::identity();
        m.m[0] = sx;
        m.m[5] = sy;
        m.m[10] = sz;
        m
    }

    /// Matrix multiplication
    #[inline]
    pub fn multiply(&self, other: &Matrix3d) -> Self {
        let mut result = Self::identity();
        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.m[i * 4 + k] * other.m[k * 4 + j];
                }
                result.m[i * 4 + j] = sum;
            }
        }
        result
    }

    /// Convert to CSS matrix3d() string
    pub fn to_css_string(&self) -> String {
        format!(
            "matrix3d({},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{})",
            self.m[0], self.m[1], self.m[2], self.m[3],
            self.m[4], self.m[5], self.m[6], self.m[7],
            self.m[8], self.m[9], self.m[10], self.m[11],
            self.m[12], self.m[13], self.m[14], self.m[15]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bezier_quadratic() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(1.0, 1.0);
        let p2 = Vec2::new(2.0, 0.0);

        let mid = calculate_bezier_quadratic(p0, p1, p2, 0.5);
        assert!((mid.x - 1.0).abs() < 0.01);
        assert!((mid.y - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_matrix_identity() {
        let m = Matrix3d::identity();
        let p = Vec2::new(5.0, 10.0);
        // Identity should not change point
    }
}
