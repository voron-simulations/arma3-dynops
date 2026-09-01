use crate::bounding::bounding_ellipse;
use crate::kdtree::KdTree;
use nalgebra::Vector2;
use std::collections::{HashMap, VecDeque};

pub const EPSILON: f64 = 100.0;
pub const MIN_POINTS: usize = 6;

pub fn entrypoint(data: &str) -> Result<String, String> {
    let mut points: Vec<Vector2<f64>> = Vec::with_capacity(1000);

    for line in data.lines() {
        if line.is_empty() {
            continue;
        }
        let parts = line.split_once(',').ok_or(format!(
            "Expected two comma-delimited coordinates, got {line}"
        ))?;
        let x: f64 = parts
            .0
            .parse::<f64>()
            .map_err(|_| format!("Failed to parse value {}", parts.0))?;
        let y: f64 = parts
            .1
            .parse::<f64>()
            .map_err(|_| format!("Failed to parse value {}", parts.1))?;
        points.push(Vector2::new(x, y));
    }

    let classifications = cluster(EPSILON, MIN_POINTS, &points);

    let mut clusters: HashMap<usize, Vec<Vector2<f64>>> = HashMap::new();
    for (class, coord) in classifications.iter().zip(points) {
        match class {
            Core(i) => {
                if clusters.contains_key(i) {
                    clusters.get_mut(i).unwrap().push(coord);
                } else {
                    clusters.insert(*i, vec![coord]);
                }
            }
            Edge(i) => {
                if clusters.contains_key(i) {
                    clusters.get_mut(i).unwrap().push(coord);
                } else {
                    clusters.insert(*i, vec![coord]);
                }
            }
            _ => {}
        }
    }
    let centers: Vec<String> = clusters
        .values()
        .map(|cluster_points| bounding_ellipse(cluster_points, 0.1))
        .map(|area| format_area(&area))
        .collect();
    Ok(format!("[\n{}\n]", centers.join(",\n")))
}

fn format_area(_area: &crate::shape::Ellipse) -> String {
    format!(
        "[{},{},{},{},{}]",
        _area.x, _area.y, _area.a, _area.b, _area.r
    )
}

// https://github.com/lazear/dbscan/blob/master/src/lib.rs
use Classification::{Core, Edge, Noise};

/// Classification according to the DBSCAN algorithm
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Classification {
    /// A point with at least `min_points` neighbors within `eps` diameter
    Core(usize),
    /// A point within `eps` of a core point, but has less than `min_points` neighbors
    Edge(usize),
    /// A point with no connections
    Noise,
}

/// Cluster datapoints using the DBSCAN algorithm
///
/// # Arguments
/// * `eps` - maximum distance between datapoints within a cluster
/// * `min_points` - minimum number of datapoints to make a cluster
/// * `input` - datapoints to cluster
pub fn cluster(eps: f64, min_points: usize, input: &[Vector2<f64>]) -> Vec<Classification> {
    Model::new(eps, min_points, input).run()
}

/// DBSCAN parameters and working state
struct Model<'a> {
    /// Epsilon value - maximum distance between points in a cluster
    eps: f64,
    /// Minimum number of points in a cluster
    mpt: usize,

    points: &'a [Vector2<f64>],
    tree: KdTree,

    c: Vec<Classification>,
    /// Has the radius query for this point already been run?
    v: Vec<bool>,
    /// Is this point already sitting in the expansion worklist?
    queued: Vec<bool>,
    /// Scratch buffer for `KdTree::within`, reused across every query in the run.
    buf: Vec<usize>,
}

impl<'a> Model<'a> {
    fn new(eps: f64, min_points: usize, points: &'a [Vector2<f64>]) -> Model<'a> {
        let n = points.len();
        Model {
            eps,
            mpt: min_points,
            tree: KdTree::build(points),
            points,
            c: vec![Noise; n],
            v: vec![false; n],
            queued: vec![false; n],
            buf: Vec::new(),
        }
    }

    /// Expand the cluster reachable from a freshly-confirmed core point.
    ///
    /// Iterative breadth-first worklist, replacing the depth-first recursion this
    /// used to be (a densely-connected blob could recurse thousands deep). This can
    /// change *which* cluster a border point reachable from two clusters ends up
    /// claimed by on a tie (whichever expansion reaches it first wins, and BFS vs
    /// DFS reach points in different orders), but never changes the core/edge/noise
    /// partition itself.
    ///
    /// Expects `self.buf` to already hold `index`'s neighbors on entry.
    fn expand_cluster(&mut self, index: usize, cluster: usize) {
        self.c[index] = Core(cluster);
        self.queued[index] = true;

        let mut queue: VecDeque<usize> = VecDeque::new();
        for i in 0..self.buf.len() {
            let n_idx = self.buf[i];
            if !self.queued[n_idx] {
                self.queued[n_idx] = true;
                queue.push_back(n_idx);
            }
        }

        while let Some(n_idx) = queue.pop_front() {
            // n_idx is at least an edge point of this cluster. This must happen
            // before the visited check: a point already visited and left as Noise
            // by `run` (too few neighbors to start its own cluster) must still
            // become a border point here.
            if self.c[n_idx] == Noise {
                self.c[n_idx] = Edge(cluster);
            }

            if !self.v[n_idx] {
                self.v[n_idx] = true;
                self.buf.clear();
                self.tree
                    .within(self.points[n_idx], self.eps, &mut self.buf);
                if self.buf.len() >= self.mpt {
                    // n_idx is a core point, we can reach at least min_points neighbors
                    self.c[n_idx] = Core(cluster);
                    for i in 0..self.buf.len() {
                        let nn_idx = self.buf[i];
                        if !self.queued[nn_idx] {
                            self.queued[nn_idx] = true;
                            queue.push_back(nn_idx);
                        }
                    }
                }
            }
        }
    }

    fn run(mut self) -> Vec<Classification> {
        let mut cluster = 0;
        for idx in 0..self.points.len() {
            if self.v[idx] {
                continue;
            }
            self.v[idx] = true;
            self.buf.clear();
            self.tree.within(self.points[idx], self.eps, &mut self.buf);
            if self.buf.len() >= self.mpt {
                self.expand_cluster(idx, cluster);
                cluster += 1;
            }
        }
        self.c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_point_center() {
        let data = "0,0\n1,0\n2,0\n0,1\n1,1\n2,1\n0,2\n1,2\n2,2".to_owned();
        entrypoint(&data).unwrap();
    }

    #[test]
    fn cluster_nine_points() {
        let args: Vec<Vector2<f64>> = vec![
            Vector2::new(0., 0.),
            Vector2::new(1., 0.),
            Vector2::new(2., 0.),
            Vector2::new(0., 1.),
            Vector2::new(1., 1.),
            Vector2::new(2., 1.),
            Vector2::new(0., 2.),
            Vector2::new(1., 2.),
            Vector2::new(2., 2.),
        ];
        assert_eq!(
            vec![
                Core(0),
                Core(0),
                Core(0),
                Core(0),
                Core(0),
                Core(0),
                Core(0),
                Core(0),
                Core(0),
            ],
            cluster(5., 3, &args)
        );
    }

    #[test]
    fn empty_input() {
        let points: Vec<Vector2<f64>> = vec![];
        assert_eq!(
            Vec::<Classification>::new(),
            cluster(EPSILON, MIN_POINTS, &points)
        );
    }

    #[test]
    fn all_noise_when_points_further_apart_than_eps() {
        // Grid spaced far enough apart (200 units) that no point has any neighbor
        // within eps (100 units) other than itself.
        let points: Vec<Vector2<f64>> = (0..5)
            .flat_map(|x| (0..5).map(move |y| Vector2::new(x as f64 * 200., y as f64 * 200.)))
            .collect();
        let result = cluster(EPSILON, MIN_POINTS, &points);
        assert!(result.iter().all(|c| *c == Noise));
    }

    #[test]
    fn two_well_separated_clusters_get_distinct_ids() {
        // Two dense 3x3 grids far apart; each point has enough neighbors to be Core.
        let mut points: Vec<Vector2<f64>> = Vec::new();
        for x in 0..3 {
            for y in 0..3 {
                points.push(Vector2::new(x as f64, y as f64));
            }
        }
        for x in 0..3 {
            for y in 0..3 {
                points.push(Vector2::new(10_000. + x as f64, 10_000. + y as f64));
            }
        }
        let result = cluster(5., 3, &points);
        let first_cluster = result[0];
        let second_cluster = result[9];
        assert!(matches!(first_cluster, Core(_)));
        assert!(matches!(second_cluster, Core(_)));
        assert_ne!(first_cluster, second_cluster);
        for &c in &result[0..9] {
            assert_eq!(c, first_cluster);
        }
        for &c in &result[9..18] {
            assert_eq!(c, second_cluster);
        }
    }

    #[test]
    fn long_chain_does_not_stack_overflow() {
        // A chain long enough that the old recursive expand_cluster would have
        // nested deeply enough to risk a stack overflow. The iterative worklist
        // handles it fine; this only checks it completes and forms a single
        // connected cluster, not the old recursive helper (which we deliberately
        // do not run on data this deep).
        let n = 50_000;
        let points: Vec<Vector2<f64>> = (0..n).map(|i| Vector2::new(i as f64 * 10., 0.)).collect();
        let result = cluster(EPSILON, MIN_POINTS, &points);
        assert!(result.iter().all(|c| !matches!(c, Noise)));
        let Core(cluster_id) = result[0] else {
            panic!(
                "expected first point to be a core point, got {:?}",
                result[0]
            );
        };
        for c in &result {
            match c {
                Core(id) | Edge(id) => assert_eq!(*id, cluster_id),
                Noise => panic!("unexpected noise point in a dense chain"),
            }
        }
    }

    /// The pre-rewrite O(n^2) recursive DBSCAN, kept only to cross-check the
    /// k-d-tree-backed iterative version against real map data.
    fn brute_cluster(eps: f64, min_points: usize, points: &[Vector2<f64>]) -> Vec<Classification> {
        struct BruteModel {
            eps: f64,
            mpt: usize,
            c: Vec<Classification>,
            v: Vec<bool>,
        }

        impl BruteModel {
            fn range_query(&self, sample: Vector2<f64>, points: &[Vector2<f64>]) -> Vec<usize> {
                points
                    .iter()
                    .enumerate()
                    .filter(|(_, &pt)| (sample - pt).norm() < self.eps)
                    .map(|(idx, _)| idx)
                    .collect()
            }

            fn expand_cluster(
                &mut self,
                points: &[Vector2<f64>],
                index: usize,
                neighbors: &[usize],
                cluster: usize,
            ) {
                self.c[index] = Core(cluster);
                for &n_idx in neighbors {
                    let visited = self.v[n_idx];
                    if self.c[n_idx] == Noise {
                        self.c[n_idx] = Edge(cluster);
                    }
                    if !visited {
                        self.v[n_idx] = true;
                        let nn = self.range_query(points[n_idx], points);
                        if nn.len() >= self.mpt {
                            self.expand_cluster(points, n_idx, &nn, cluster);
                        }
                    }
                }
            }

            fn run(mut self, points: &[Vector2<f64>]) -> Vec<Classification> {
                self.c = vec![Noise; points.len()];
                self.v = vec![false; points.len()];
                let mut cluster = 0;
                for idx in 0..points.len() {
                    if !self.v[idx] {
                        self.v[idx] = true;
                        let n = self.range_query(points[idx], points);
                        if n.len() >= self.mpt {
                            self.expand_cluster(points, idx, &n, cluster);
                            cluster += 1;
                        }
                    }
                }
                self.c
            }
        }

        BruteModel {
            eps,
            mpt: min_points,
            c: Vec::new(),
            v: Vec::new(),
        }
        .run(points)
    }

    fn parse(data: &str) -> Vec<Vector2<f64>> {
        data.lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| line.split_once(','))
            .filter_map(|(x, y)| Some(Vector2::new(x.parse().ok()?, y.parse().ok()?)))
            .collect()
    }

    #[test]
    fn matches_brute_force_on_stratis() {
        let points = parse(include_str!("../data/objects.Stratis.txt"));
        let expected = brute_cluster(EPSILON, MIN_POINTS, &points);
        let actual = cluster(EPSILON, MIN_POINTS, &points);
        assert_eq!(expected, actual);
    }
}
