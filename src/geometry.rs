//! Exact, deterministic bounding geometry for building clusters.
//!
//! `bounding::bounding_ellipse` is an iterative MVEE fit: it is outlier-sensitive
//! and its inner matrix inversion is singular for fewer than 3 points, which is
//! exactly the 2-3 house farm case this module exists to handle. A convex-hull +
//! rotating-calipers minimum-area oriented bounding box is O(n log n), exact, and
//! well-defined for n = 0, 1, 2 points and for collinear/duplicate input.

use nalgebra::Vector2;

/// A minimum-area oriented bounding box.
///
/// `a` and `b` are half-extents (matching `setMarkerSize`'s half-size convention),
/// `angle` is measured counter-clockwise from +X in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Obb {
    pub center: Vector2<f64>,
    pub a: f64,
    pub b: f64,
    pub angle: f64,
}

impl Obb {
    /// Whether `point` lies inside the box, allowing `tolerance` extra absolute
    /// distance past each edge.
    pub fn contains(&self, point: Vector2<f64>, tolerance: f64) -> bool {
        let d = point - self.center;
        let (s, c) = self.angle.sin_cos();
        let u = d.x * c + d.y * s;
        let v = -d.x * s + d.y * c;
        u.abs() <= self.a + tolerance && v.abs() <= self.b + tolerance
    }
}

fn cross(o: Vector2<f64>, a: Vector2<f64>, b: Vector2<f64>) -> f64 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

/// Convex hull via Andrew's monotone chain, returned counter-clockwise with no
/// repeated start/end point. Collinear boundary points are dropped, so a fully
/// collinear input collapses to its two extreme points.
pub fn convex_hull(points: &[Vector2<f64>]) -> Vec<Vector2<f64>> {
    let mut pts: Vec<Vector2<f64>> = points.to_vec();
    pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    pts.dedup_by(|a, b| a.x == b.x && a.y == b.y);

    if pts.len() <= 2 {
        return pts;
    }

    let mut lower: Vec<Vector2<f64>> = Vec::with_capacity(pts.len());
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }

    let mut upper: Vec<Vector2<f64>> = Vec::with_capacity(pts.len());
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }

    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Minimum-area oriented bounding box via rotating calipers over the convex hull.
///
/// `None` only for empty input, so the no-points case is unrepresentable downstream.
/// For a single point, `a == b == 0.0`. For two points (or a collinear input, which
/// the hull collapses to two points), the box degenerates to a segment: `a` is the
/// half-length along the segment and `b == 0.0`.
pub fn min_area_obb(points: &[Vector2<f64>]) -> Option<Obb> {
    let hull = convex_hull(points);
    match hull.len() {
        0 => None,
        1 => Some(Obb {
            center: hull[0],
            a: 0.0,
            b: 0.0,
            angle: 0.0,
        }),
        2 => {
            let d = hull[1] - hull[0];
            let center = (hull[0] + hull[1]) * 0.5;
            Some(Obb {
                center,
                a: d.norm() / 2.0,
                b: 0.0,
                angle: d.y.atan2(d.x),
            })
        }
        n => {
            // O(h^2): for each hull edge direction, re-scan every hull point for the
            // bounding extent along that axis. h is the hull size of a single
            // settlement's buildings (tens, occasionally low hundreds), so the
            // quadratic term is negligible next to the O(n log n) hull construction.
            let mut best: Option<Obb> = None;
            let mut best_area = f64::INFINITY;

            for i in 0..n {
                let edge = hull[(i + 1) % n] - hull[i];
                let edge_len = edge.norm();
                if edge_len == 0.0 {
                    continue;
                }
                let ux = edge.x / edge_len;
                let uy = edge.y / edge_len;

                let mut min_u = f64::INFINITY;
                let mut max_u = f64::NEG_INFINITY;
                let mut min_v = f64::INFINITY;
                let mut max_v = f64::NEG_INFINITY;
                for &p in &hull {
                    let u = p.x * ux + p.y * uy;
                    let v = -p.x * uy + p.y * ux;
                    min_u = min_u.min(u);
                    max_u = max_u.max(u);
                    min_v = min_v.min(v);
                    max_v = max_v.max(v);
                }

                let area = (max_u - min_u) * (max_v - min_v);
                if area < best_area {
                    best_area = area;
                    let cu = (min_u + max_u) / 2.0;
                    let cv = (min_v + max_v) / 2.0;
                    best = Some(Obb {
                        center: Vector2::new(cu * ux - cv * uy, cu * uy + cv * ux),
                        a: (max_u - min_u) / 2.0,
                        b: (max_v - min_v) / 2.0,
                        angle: uy.atan2(ux),
                    });
                }
            }
            best
        }
    }
}

/// Converts an OBB angle (radians, CCW from +X) to an Arma marker/location
/// direction (degrees, clockwise from north/+Y), normalized to `[0, 360)`.
pub fn obb_to_marker_dir(angle: f64) -> f64 {
    let degrees = (90.0 - angle.to_degrees()) % 360.0;
    if degrees < 0.0 {
        degrees + 360.0
    } else {
        degrees
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    fn assert_contains_all(obb: &Obb, points: &[Vector2<f64>]) {
        for &p in points {
            assert!(
                obb.contains(p, 1e-9),
                "point {p:?} not contained in {obb:?}"
            );
        }
    }

    #[test]
    fn hull_empty() {
        assert!(convex_hull(&[]).is_empty());
    }

    #[test]
    fn hull_single_point() {
        let p = Vector2::new(3.0, 4.0);
        assert_eq!(convex_hull(&[p]), vec![p]);
    }

    #[test]
    fn hull_duplicates_collapse() {
        let p = Vector2::new(1.0, 1.0);
        let hull = convex_hull(&[p, p, p, p]);
        assert_eq!(hull, vec![p]);
    }

    #[test]
    fn hull_collinear_collapses_to_endpoints() {
        let points: Vec<Vector2<f64>> = (0..10).map(|i| Vector2::new(i as f64, 0.0)).collect();
        let hull = convex_hull(&points);
        assert_eq!(hull.len(), 2);
        assert!(hull.contains(&Vector2::new(0.0, 0.0)));
        assert!(hull.contains(&Vector2::new(9.0, 0.0)));
    }

    #[test]
    fn hull_interior_points_dropped() {
        let points = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(4.0, 0.0),
            Vector2::new(4.0, 4.0),
            Vector2::new(0.0, 4.0),
            Vector2::new(2.0, 2.0), // interior, must not appear in hull
        ];
        let hull = convex_hull(&points);
        assert_eq!(hull.len(), 4);
        assert!(!hull.contains(&Vector2::new(2.0, 2.0)));
    }

    #[test]
    fn obb_empty_is_none() {
        assert_eq!(min_area_obb(&[]), None);
    }

    #[test]
    fn obb_single_point() {
        let p = Vector2::new(5.0, -3.0);
        let obb = min_area_obb(&[p]).expect("single point must yield an OBB");
        assert_eq!(obb.center, p);
        assert_eq!(obb.a, 0.0);
        assert_eq!(obb.b, 0.0);
    }

    #[test]
    fn obb_two_points() {
        let points = [Vector2::new(0.0, 0.0), Vector2::new(6.0, 8.0)];
        let obb = min_area_obb(&points).expect("two points must yield an OBB");
        assert!(
            (obb.a - 5.0).abs() < 1e-9,
            "half-length should be 5.0 (10/2), got {}",
            obb.a
        );
        assert_eq!(obb.b, 0.0);
        assert!((obb.center - Vector2::new(3.0, 4.0)).norm() < 1e-9);
        assert_contains_all(&obb, &points);
    }

    #[test]
    fn obb_collinear_points() {
        let points: Vec<Vector2<f64>> = (0..5).map(|i| Vector2::new(i as f64 * 2.0, 3.0)).collect();
        let obb = min_area_obb(&points).expect("collinear points must yield an OBB");
        assert_eq!(obb.b, 0.0);
        assert_contains_all(&obb, &points);
    }

    #[test]
    fn obb_duplicate_points() {
        let p = Vector2::new(1.0, 1.0);
        let points = vec![p; 6];
        let obb = min_area_obb(&points).expect("duplicate points must yield an OBB");
        assert_eq!(obb.center, p);
        assert_eq!(obb.a, 0.0);
        assert_eq!(obb.b, 0.0);
    }

    #[test]
    fn obb_axis_aligned_square() {
        let points = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(2.0, 2.0),
            Vector2::new(0.0, 2.0),
        ];
        let obb = min_area_obb(&points).expect("square must yield an OBB");
        assert!((obb.center - Vector2::new(1.0, 1.0)).norm() < 1e-9);
        assert!((obb.a - 1.0).abs() < 1e-9);
        assert!((obb.b - 1.0).abs() < 1e-9);
        // A square's minimal box is achieved at any quarter-turn; only the
        // orientation modulo 90 degrees is meaningful.
        let residual = obb.angle.rem_euclid(FRAC_PI_2);
        assert!(residual < 1e-9 || (FRAC_PI_2 - residual) < 1e-9);
        assert_contains_all(&obb, &points);
    }

    #[test]
    fn obb_rotated_rectangle() {
        // A 10x4 rectangle rotated 30 degrees, centered at (50, -20).
        let theta = 30f64.to_radians();
        let (s, c) = theta.sin_cos();
        let center = Vector2::new(50.0, -20.0);
        let local = [
            Vector2::new(-5.0, -2.0),
            Vector2::new(5.0, -2.0),
            Vector2::new(5.0, 2.0),
            Vector2::new(-5.0, 2.0),
        ];
        let points: Vec<Vector2<f64>> = local
            .iter()
            .map(|p| center + Vector2::new(p.x * c - p.y * s, p.x * s + p.y * c))
            .collect();

        let obb = min_area_obb(&points).expect("rectangle must yield an OBB");
        assert!((obb.center - center).norm() < 1e-6);

        // The true minimal box is 10x4 (area 40); (a, b) may come out as
        // (5, 2) or (2, 5) depending on which hull edge the calipers hit first.
        assert!((obb.a * 2.0 * obb.b * 2.0 - 40.0).abs() < 1e-6);
        let mut half_extents = [obb.a, obb.b];
        half_extents.sort_by(|x, y| x.total_cmp(y));
        assert!((half_extents[0] - 2.0).abs() < 1e-6);
        assert!((half_extents[1] - 5.0).abs() < 1e-6);

        // Angle must match the rectangle's orientation modulo a quarter turn.
        let residual = (obb.angle - theta).rem_euclid(FRAC_PI_2);
        assert!(residual < 1e-6 || (FRAC_PI_2 - residual) < 1e-6);

        assert_contains_all(&obb, &points);
    }

    #[test]
    fn obb_contains_property_across_random_rotated_rectangles() {
        struct Lcg(u64);
        impl Lcg {
            fn next(&mut self) -> f64 {
                self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
                ((self.0 >> 33) as f64) / (u32::MAX as f64)
            }
        }
        let mut rng = Lcg(42);

        for _ in 0..200 {
            let theta = rng.next() * PI;
            let (s, c) = theta.sin_cos();
            let cx = (rng.next() - 0.5) * 2000.0;
            let cy = (rng.next() - 0.5) * 2000.0;
            let hw = 1.0 + rng.next() * 50.0;
            let hh = 1.0 + rng.next() * 50.0;
            let corners = [
                Vector2::new(-hw, -hh),
                Vector2::new(hw, -hh),
                Vector2::new(hw, hh),
                Vector2::new(-hw, hh),
            ];
            let points: Vec<Vector2<f64>> = corners
                .iter()
                .map(|p| Vector2::new(cx, cy) + Vector2::new(p.x * c - p.y * s, p.x * s + p.y * c))
                .collect();

            let obb = min_area_obb(&points).expect("rectangle must yield an OBB");
            assert_contains_all(&obb, &points);
        }
    }

    #[test]
    fn marker_dir_cardinal_directions() {
        assert!((obb_to_marker_dir(0.0) - 90.0).abs() < 1e-9); // +X (east) -> 90
        assert!((obb_to_marker_dir(FRAC_PI_2) - 0.0).abs() < 1e-9); // +Y (north) -> 0
        assert!((obb_to_marker_dir(PI) - 270.0).abs() < 1e-9); // -X (west) -> 270
        assert!((obb_to_marker_dir(-FRAC_PI_2) - 180.0).abs() < 1e-9); // -Y (south) -> 180
    }

    #[test]
    fn marker_dir_stays_in_range() {
        let mut rng_state: u64 = 7;
        for _ in 0..100 {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let t = ((rng_state >> 33) as f64) / (u32::MAX as f64);
            let angle = (t - 0.5) * 4.0 * PI;
            let dir = obb_to_marker_dir(angle);
            assert!(
                (0.0..360.0).contains(&dir),
                "dir {dir} out of range for angle {angle}"
            );
        }
    }

    #[test]
    fn marker_dir_diagonal() {
        // 45 degrees CCW from +X is compass NE = 45 degrees.
        assert!((obb_to_marker_dir(FRAC_PI_4) - 45.0).abs() < 1e-9);
    }
}
