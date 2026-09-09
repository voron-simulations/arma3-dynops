//! Settlement detection: recursive span-limited single-linkage clustering.
//!
//! A single fixed `EPSILON` (as `cluster::entrypoint` used) chains settlements
//! together on dense maps -- on Altis the largest single-linkage component at
//! eps=100 spans 2468x3688m, a tenth of the map. Cutting the dendrogram at a
//! settlement-sized span instead fixes that without losing 2-3 house farms the
//! way a large `min_points` floor would. See adr/0001-location-detection.md.

use crate::cluster::{Classification, cluster};
use crate::geometry::{Obb, min_area_obb};
use nalgebra::Vector2;
use std::collections::HashMap;

/// Tuning knobs for [`detect`]. Defaults are the values validated against the
/// bundled map dumps (see the ADR's measurement table).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetectParams {
    /// Initial single-linkage radius.
    pub eps: f64,
    /// A component wider than this (in either axis) gets re-cut at a smaller eps.
    pub max_span: f64,
    /// eps multiplier applied at each recursion level.
    pub split_factor: f64,
    /// Recursion stops here even if a component is still oversized.
    pub min_eps: f64,
    /// Components smaller than this are dropped.
    pub min_buildings: usize,
}

impl Default for DetectParams {
    fn default() -> Self {
        DetectParams {
            eps: 100.0,
            max_span: 700.0,
            split_factor: 0.7,
            min_eps: 35.0,
            min_buildings: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationClass {
    Farm,    // 2-4 buildings
    Hamlet,  // 5-12 buildings
    Village, // 13-40 buildings
    Town,    // 41-120 buildings
    City,    // 120+ buildings
}

impl LocationClass {
    fn classify(buildings: usize) -> Self {
        match buildings {
            0..=4 => LocationClass::Farm,
            5..=12 => LocationClass::Hamlet,
            13..=40 => LocationClass::Village,
            41..=120 => LocationClass::Town,
            _ => LocationClass::City,
        }
    }

    /// Enumeration index used on the wire protocol (`locations:page`'s
    /// `classIndex`), in declaration order.
    pub fn index(self) -> u8 {
        match self {
            LocationClass::Farm => 0,
            LocationClass::Hamlet => 1,
            LocationClass::Village => 2,
            LocationClass::Town => 3,
            LocationClass::City => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub id: usize,
    pub obb: Obb,
    pub buildings: usize,
    pub class: LocationClass,
}

fn span(points: &[Vector2<f64>]) -> f64 {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in points {
        min_x = min_x.min(p.x);
        max_x = max_x.max(p.x);
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
    }
    (max_x - min_x).max(max_y - min_y)
}

/// `cluster(eps, 2, points)` *is* connected-components at radius eps: `KdTree::within`
/// includes the query point itself, so a point with one neighbor is `Core` and an
/// isolated point is `Noise` -- singletons drop out for free.
fn group_by_cluster(points: &[Vector2<f64>], eps: f64) -> Vec<Vec<Vector2<f64>>> {
    let classes = cluster(eps, 2, points);
    let mut groups: HashMap<usize, Vec<Vector2<f64>>> = HashMap::new();
    for (class, &point) in classes.iter().zip(points) {
        let id = match class {
            Classification::Core(id) | Classification::Edge(id) => *id,
            Classification::Noise => continue,
        };
        groups.entry(id).or_default().push(point);
    }
    groups.into_values().collect()
}

fn detect_into(
    points: &[Vector2<f64>],
    eps: f64,
    params: &DetectParams,
    out: &mut Vec<Vec<Vector2<f64>>>,
) {
    for group in group_by_cluster(points, eps) {
        if span(&group) > params.max_span && eps > params.min_eps {
            detect_into(&group, eps * params.split_factor, params, out);
        } else if group.len() >= params.min_buildings {
            out.push(group);
        }
    }
}

/// Detects settlements in `points` (building positions).
///
/// Runs DBSCAN-as-connected-components at `params.eps`; any resulting component
/// wider than `params.max_span` is re-cut at `eps * params.split_factor` and so
/// on down to `params.min_eps`. Each surviving component becomes a [`Location`]
/// with a minimum-area bounding box and a class derived from building count.
pub fn detect(points: &[Vector2<f64>], params: &DetectParams) -> Vec<Location> {
    let mut groups = Vec::new();
    detect_into(points, params.eps, params, &mut groups);

    groups
        .into_iter()
        .enumerate()
        .filter_map(|(id, group)| {
            let obb = min_area_obb(&group)?;
            let buildings = group.len();
            Some(Location {
                id,
                obb,
                buildings,
                class: LocationClass::classify(buildings),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(origin: Vector2<f64>, n: usize, spacing: f64) -> Vec<Vector2<f64>> {
        (0..n)
            .flat_map(|i| (0..n).map(move |j| (i, j)))
            .map(|(i, j)| origin + Vector2::new(i as f64 * spacing, j as f64 * spacing))
            .collect()
    }

    #[test]
    fn two_well_separated_blobs_stay_separate() {
        let mut points = grid(Vector2::new(0.0, 0.0), 3, 10.0);
        points.extend(grid(Vector2::new(10_000.0, 10_000.0), 3, 10.0));

        let locations = detect(&points, &DetectParams::default());
        assert_eq!(locations.len(), 2);
        for loc in &locations {
            assert_eq!(loc.buildings, 9);
        }
    }

    #[test]
    fn isolated_point_is_dropped() {
        // Spaced far enough apart (1000 units) that nothing is within eps (100) of
        // anything else.
        let points: Vec<Vector2<f64>> = (0..5)
            .map(|i| Vector2::new(i as f64 * 1000.0, 0.0))
            .collect();
        let locations = detect(&points, &DetectParams::default());
        assert!(locations.is_empty());
    }

    #[test]
    fn two_point_cluster_survives_min_buildings_two() {
        let points = vec![Vector2::new(0.0, 0.0), Vector2::new(10.0, 0.0)];
        let locations = detect(&points, &DetectParams::default());
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].buildings, 2);
        assert_eq!(locations[0].class, LocationClass::Farm);
    }

    #[test]
    fn single_point_is_dropped_even_alone() {
        let points = vec![Vector2::new(0.0, 0.0)];
        let locations = detect(&points, &DetectParams::default());
        assert!(locations.is_empty());
    }

    #[test]
    fn empty_input_yields_no_locations() {
        assert!(detect(&[], &DetectParams::default()).is_empty());
    }

    #[test]
    fn blob_spanning_more_than_max_span_gets_split() {
        // Two tight 3x3 grids ~820 units apart (over max_span=700), joined by a
        // sparse "bridge" of points 80 units apart. At the initial eps=100 the
        // bridge links everything into one component; once eps shrinks to 70
        // (100 * split_factor 0.7) the 80-unit bridge gaps exceed eps and break,
        // leaving the two dense grids as separate, span-compliant locations.
        let mut points = grid(Vector2::new(0.0, 0.0), 3, 10.0);
        points.extend(grid(Vector2::new(800.0, 0.0), 3, 10.0));
        for k in 1..9 {
            points.push(Vector2::new(k as f64 * 80.0, 0.0));
        }

        let params = DetectParams::default();
        let locations = detect(&points, &params);
        assert_eq!(
            locations.len(),
            2,
            "expected the bridge to be cut, got {locations:?}"
        );
        for loc in &locations {
            assert!(loc.obb.a * 2.0 <= params.max_span + 1e-6);
            assert!(loc.obb.b * 2.0 <= params.max_span + 1e-6);
        }
    }

    #[test]
    fn span_floor_stops_recursion_at_min_eps() {
        // A uniform chain spaced 5 units apart, 200 points long (span ~995,
        // over max_span=700). The spacing is tight enough that the chain
        // stays fully connected through every recursion level (100 -> 70 ->
        // 49 -> 34.3), so it can never be *split*; once eps drops to/below
        // min_eps (35) recursion stops and the still-oversized chain is
        // emitted as a single location rather than being dropped.
        let points: Vec<Vector2<f64>> = (0..200)
            .map(|i| Vector2::new(i as f64 * 5.0, 0.0))
            .collect();
        let params = DetectParams {
            eps: 100.0,
            max_span: 700.0,
            split_factor: 0.7,
            min_eps: 35.0,
            min_buildings: 2,
        };
        let locations = detect(&points, &params);
        assert_eq!(
            locations.len(),
            1,
            "expected one oversized location, got {locations:?}"
        );
        assert!(locations[0].obb.a * 2.0 > params.max_span);
    }

    #[test]
    fn classification_boundaries() {
        assert_eq!(LocationClass::classify(4), LocationClass::Farm);
        assert_eq!(LocationClass::classify(5), LocationClass::Hamlet);
        assert_eq!(LocationClass::classify(12), LocationClass::Hamlet);
        assert_eq!(LocationClass::classify(13), LocationClass::Village);
        assert_eq!(LocationClass::classify(40), LocationClass::Village);
        assert_eq!(LocationClass::classify(41), LocationClass::Town);
        assert_eq!(LocationClass::classify(120), LocationClass::Town);
        assert_eq!(LocationClass::classify(121), LocationClass::City);
    }

    #[test]
    fn every_building_lies_inside_exactly_one_location_obb() {
        let mut points = grid(Vector2::new(0.0, 0.0), 4, 15.0);
        points.extend(grid(Vector2::new(5_000.0, 0.0), 6, 12.0));
        let locations = detect(&points, &DetectParams::default());
        assert_eq!(locations.len(), 2);
        for &p in &points {
            let containing = locations
                .iter()
                .filter(|loc| loc.obb.contains(p, 1e-6))
                .count();
            assert_eq!(
                containing, 1,
                "point {p:?} should be inside exactly one location OBB"
            );
        }
    }
}
