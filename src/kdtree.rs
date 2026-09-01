use nalgebra::Vector2;

/// A point together with its index in the slice the tree was built from.
#[derive(Clone, Copy)]
struct Node {
    point: Vector2<f64>,
    id: usize,
}

/// Immutable 2-d tree, bulk-built by recursive median split.
///
/// Nodes live in one flat `Vec` in implicit-subtree order: within the range
/// `lo..hi` the node at `mid = (lo + hi) / 2` is the splitter, `lo..mid` is its
/// left subtree and `mid + 1..hi` its right. Balance comes from the median
/// split, so no child pointers are stored.
pub struct KdTree {
    nodes: Vec<Node>,
}

impl KdTree {
    /// Bulk-load a tree from `points` via recursive median split.
    pub fn build(points: &[Vector2<f64>]) -> Self {
        let mut nodes: Vec<Node> = points
            .iter()
            .enumerate()
            .map(|(id, &point)| Node { point, id })
            .collect();
        let len = nodes.len();
        Self::build_range(&mut nodes, 0, len, 0);
        Self { nodes }
    }

    fn build_range(nodes: &mut [Node], lo: usize, hi: usize, axis: usize) {
        if hi - lo < 2 {
            return;
        }
        let mid = lo + (hi - lo) / 2;
        nodes[lo..hi]
            .select_nth_unstable_by(mid - lo, |a, b| a.point[axis].total_cmp(&b.point[axis]));
        let next_axis = 1 - axis;
        Self::build_range(nodes, lo, mid, next_axis);
        Self::build_range(nodes, mid + 1, hi, next_axis);
    }

    /// Appends the indices of every point within `radius` of `center` to `out`.
    ///
    /// Uses squared-distance comparison against `radius * radius`, strictly
    /// less-than to match the linear-scan predicate it replaces. This can
    /// differ from a direct `sqrt`-based comparison by an ulp for a point
    /// sitting exactly on the boundary, which does not matter at map scale.
    pub fn within(&self, center: Vector2<f64>, radius: f64, out: &mut Vec<usize>) {
        if self.nodes.is_empty() {
            return;
        }
        self.within_range(0, self.nodes.len(), 0, center, radius, out);
    }

    fn within_range(
        &self,
        lo: usize,
        hi: usize,
        axis: usize,
        center: Vector2<f64>,
        radius: f64,
        out: &mut Vec<usize>,
    ) {
        if lo >= hi {
            return;
        }
        let mid = lo + (hi - lo) / 2;
        let node = &self.nodes[mid];

        if (node.point - center).norm_squared() < radius * radius {
            out.push(node.id);
        }

        let delta = center[axis] - node.point[axis];
        let next_axis = 1 - axis;
        if delta <= radius {
            self.within_range(lo, mid, next_axis, center, radius, out);
        }
        if delta >= -radius {
            self.within_range(mid + 1, hi, next_axis, center, radius, out);
        }
    }

    /// Returns the index of the point nearest to `target`, or `None` if the tree is empty.
    pub fn nearest(&self, target: Vector2<f64>) -> Option<usize> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut best: Option<(usize, f64)> = None;
        self.nearest_range(0, self.nodes.len(), 0, target, &mut best);
        best.map(|(id, _)| id)
    }

    fn nearest_range(
        &self,
        lo: usize,
        hi: usize,
        axis: usize,
        target: Vector2<f64>,
        best: &mut Option<(usize, f64)>,
    ) {
        if lo >= hi {
            return;
        }
        let mid = lo + (hi - lo) / 2;
        let node = &self.nodes[mid];

        let d2 = (node.point - target).norm_squared();
        if best.is_none_or(|(_, best_d2)| d2 < best_d2) {
            *best = Some((node.id, d2));
        }

        let delta = target[axis] - node.point[axis];
        let next_axis = 1 - axis;
        let (near_lo, near_hi, far_lo, far_hi) = if delta <= 0. {
            (lo, mid, mid + 1, hi)
        } else {
            (mid + 1, hi, lo, mid)
        };
        self.nearest_range(near_lo, near_hi, next_axis, target, best);

        let prune = best.is_some_and(|(_, best_d2)| delta * delta >= best_d2);
        if !prune {
            self.nearest_range(far_lo, far_hi, next_axis, target, best);
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brute_within(points: &[Vector2<f64>], center: Vector2<f64>, radius: f64) -> Vec<usize> {
        points
            .iter()
            .enumerate()
            .filter(|(_, &p)| (p - center).norm_squared() < radius * radius)
            .map(|(i, _)| i)
            .collect()
    }

    fn brute_nearest(points: &[Vector2<f64>], target: Vector2<f64>) -> Option<usize> {
        points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (**a - target)
                    .norm_squared()
                    .total_cmp(&(**b - target).norm_squared())
            })
            .map(|(i, _)| i)
    }

    fn sorted(mut v: Vec<usize>) -> Vec<usize> {
        v.sort_unstable();
        v
    }

    // Small inline LCG for reproducible test point clouds; no `rand` dependency.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> f64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((self.0 >> 33) as f64) / (u32::MAX as f64)
        }
    }

    #[test]
    fn empty_tree() {
        let tree = KdTree::build(&[]);
        let mut out = Vec::new();
        tree.within(Vector2::new(0., 0.), 10., &mut out);
        assert!(out.is_empty());
        assert_eq!(tree.nearest(Vector2::new(0., 0.)), None);
        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());
    }

    #[test]
    fn single_point() {
        let points = vec![Vector2::new(1., 1.)];
        let tree = KdTree::build(&points);
        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());

        let mut out = Vec::new();
        tree.within(Vector2::new(1., 1.), 1., &mut out);
        assert_eq!(out, vec![0]);

        out.clear();
        tree.within(Vector2::new(100., 100.), 1., &mut out);
        assert!(out.is_empty());

        assert_eq!(tree.nearest(Vector2::new(5., 5.)), Some(0));
    }

    #[test]
    fn all_duplicate_points() {
        let points = vec![Vector2::new(2., 2.); 5];
        let tree = KdTree::build(&points);
        let mut out = Vec::new();
        tree.within(Vector2::new(2., 2.), 1., &mut out);
        assert_eq!(sorted(out), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn grid_hand_computed() {
        // 3x3 unit grid centered at origin.
        let points = vec![
            Vector2::new(-1., -1.),
            Vector2::new(0., -1.),
            Vector2::new(1., -1.),
            Vector2::new(-1., 0.),
            Vector2::new(0., 0.),
            Vector2::new(1., 0.),
            Vector2::new(-1., 1.),
            Vector2::new(0., 1.),
            Vector2::new(1., 1.),
        ];
        let tree = KdTree::build(&points);

        // Radius covering only the center point.
        let mut out = Vec::new();
        tree.within(Vector2::new(0., 0.), 0.5, &mut out);
        assert_eq!(sorted(out), vec![4]);

        // Radius covering the center plus its 4 orthogonal neighbors (distance 1).
        let mut out = Vec::new();
        tree.within(Vector2::new(0., 0.), 1.0 + 1e-9, &mut out);
        assert_eq!(sorted(out), vec![1, 3, 4, 5, 7]);

        // Radius covering everything (max distance sqrt(2) from center).
        let mut out = Vec::new();
        tree.within(Vector2::new(0., 0.), 2.0, &mut out);
        assert_eq!(sorted(out), (0..9).collect::<Vec<_>>());

        assert_eq!(tree.nearest(Vector2::new(0.1, 0.1)), Some(4));
        assert_eq!(tree.nearest(Vector2::new(-0.9, -0.9)), Some(0));
    }

    #[test]
    fn brute_force_cross_check() {
        let mut rng = Lcg(0x1234_5678_9abc_def0);
        let points: Vec<Vector2<f64>> = (0..300)
            .map(|_| Vector2::new(rng.next() * 200. - 100., rng.next() * 200. - 100.))
            .collect();
        let tree = KdTree::build(&points);

        for _ in 0..50 {
            let center = Vector2::new(rng.next() * 200. - 100., rng.next() * 200. - 100.);
            let radius = rng.next() * 50.;

            let mut expected = brute_within(&points, center, radius);
            let mut actual = Vec::new();
            tree.within(center, radius, &mut actual);
            expected.sort_unstable();
            actual.sort_unstable();
            assert_eq!(
                expected, actual,
                "within mismatch at center={center:?} r={radius}"
            );

            let expected_nn = brute_nearest(&points, center);
            let actual_nn = tree.nearest(center);
            // Ties (equidistant points) are broken arbitrarily; compare distances instead of ids.
            let expected_d = expected_nn.map(|i| (points[i] - center).norm_squared());
            let actual_d = actual_nn.map(|i| (points[i] - center).norm_squared());
            assert_eq!(
                expected_d, actual_d,
                "nearest mismatch at center={center:?}"
            );
        }
    }

    #[test]
    fn sorted_by_x_input() {
        // The input ordering that degenerated the old incremental tree into a linked list.
        let points: Vec<Vector2<f64>> = (0..100).map(|i| Vector2::new(i as f64, 0.)).collect();
        let tree = KdTree::build(&points);

        let mut out = Vec::new();
        tree.within(Vector2::new(50., 0.), 5.5, &mut out);
        assert_eq!(sorted(out), (45..=55).collect::<Vec<_>>());

        assert_eq!(tree.nearest(Vector2::new(50.4, 0.)), Some(50));
    }
}
