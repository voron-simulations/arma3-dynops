mod mvee;

use crate::types::{Area, AreaKind, Distance, Position2d};
use mvee::get_mvee;
use nalgebra::Vector2;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::marker::PhantomData;

const EPSILON: f64 = 100.0;
const MIN_POINTS: usize = 6;

fn format_area(area: &Area) -> String {
    format!(
        "[[{:.2},{:.2}],{:.2},{:.2},{:.2}]",
        area.center.x,
        area.center.y,
        area.xsize,
        area.ysize,
        -area.angle * 180. / PI
    )
}

fn bounding_rec(coords: &[Vector2<f64>]) -> Area {
    let mut xmin: f64 = f64::MAX;
    let mut xmax: f64 = f64::MIN;
    let mut ymin: f64 = f64::MAX;
    let mut ymax: f64 = f64::MIN;

    for point in coords {
        xmin = xmin.min(point.x);
        xmax = xmax.max(point.x);
        ymin = ymin.min(point.y);
        ymax = ymax.max(point.y);
    }
    let x = (xmax + xmin) / 2.0;
    let y = (ymax + ymin) / 2.0;

    Area {
        center: Position2d::new(x, y),
        xsize: (xmax - xmin) / 2.0,
        ysize: (ymax - ymin) / 2.0,
        angle: 0.,
        kind: AreaKind::Rectangle,
    }
}

pub fn entrypoint(data: &String) -> Result<String, String> {
    let mut points: Vec<Position2d> = Vec::new();
    points.reserve(1000);

    for line in data.lines() {
        let parts = line
            .split_once(',')
            .ok_or("Expected two comma-delimited values")?;
        let x: f64 = parts
            .0
            .parse::<f64>()
            .or_else(|_| Err(format!("Failed to parse value {}", parts.0)))?;
        let y: f64 = parts
            .1
            .parse::<f64>()
            .or_else(|_| Err(format!("Failed to parse value {}", parts.1)))?;
        points.push(Position2d::new(x, y));
    }

    let classifications = cluster(EPSILON, MIN_POINTS, &points);

    let mut clusters: HashMap<usize, Vec<Position2d>> = HashMap::new();
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
        .iter()
        .map(|(_, cluster_points)| get_mvee(cluster_points, 0.1))
        .map(|area| format_area(&area))
        .collect();
    Ok(format!("[\n{}\n]", centers.join(",\n")))
}

// https://github.com/lazear/dbscan/blob/master/src/lib.rs
use Classification::{Core, Edge, Noise};

/// Classification according to the DBSCAN algorithm
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
enum Classification {
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
/// * `input` - a Vec<Vec<f64>> of datapoints, organized by row
fn cluster<T>(eps: f64, min_points: usize, input: &Vec<T>) -> Vec<Classification>
where
    T: Distance,
{
    Model::new(eps, min_points).run(input)
}

/// DBSCAN parameters
struct Model<T>
where
    T: Distance,
{
    /// Epsilon value - maximum distance between points in a cluster
    pub eps: f64,
    /// Minimum number of points in a cluster
    pub mpt: usize,

    c: Vec<Classification>,
    v: Vec<bool>,
    __phantom: PhantomData<T>,
}

impl<T> Model<T>
where
    T: Distance,
{
    /// Create a new `Model` with a set of parameters
    ///
    /// # Arguments
    /// * `eps` - maximum distance between datapoints within a cluster
    /// * `min_points` - minimum number of datapoints to make a cluster
    pub fn new(eps: f64, min_points: usize) -> Model<T> {
        Model {
            eps,
            mpt: min_points,
            c: Vec::new(),
            v: Vec::new(),
            __phantom: PhantomData,
        }
    }

    fn expand_cluster(
        &mut self,
        population: &Vec<T>,
        index: usize,
        neighbors: &[usize],
        cluster: usize,
    ) {
        self.c[index] = Core(cluster);
        for &n_idx in neighbors {
            // Have we previously visited this point?
            let v = self.v[n_idx];
            // n_idx is at least an edge point
            if self.c[n_idx] == Noise {
                self.c[n_idx] = Edge(cluster);
            }

            if !v {
                self.v[n_idx] = true;
                // What about neighbors of this neighbor? Are they close enough to add into
                // the current cluster? If so, recurse and add them.
                let nn = self.range_query(&population[n_idx], population);
                if nn.len() >= self.mpt {
                    // n_idx is a core point, we can reach at least min_points neighbors
                    self.expand_cluster(population, n_idx, &nn, cluster);
                }
            }
        }
    }

    #[inline]
    fn range_query(&self, sample: &T, population: &Vec<T>) -> Vec<usize> {
        population
            .iter()
            .enumerate()
            .filter(|(_, pt)| sample.get_distance(pt) < self.eps)
            .map(|(idx, _)| idx)
            .collect()
    }

    pub fn run(mut self, population: &Vec<T>) -> Vec<Classification> {
        self.c = (0..population.len()).map(|_| Noise).collect();
        self.v = (0..population.len()).map(|_| false).collect();

        let mut cluster = 0;
        for (idx, sample) in population.iter().enumerate() {
            let v = self.v[idx];
            if !v {
                self.v[idx] = true;
                let n = self.range_query(sample, population);
                if n.len() >= self.mpt {
                    self.expand_cluster(population, idx, &n, cluster);
                    cluster += 1;
                }
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
        let data = "[\"\",0,0]\n[\"\",1,0]\n[\"\",2,0]\n[\"\",0,1]\n[\"\",1,1]\n[\"\",2,1]\n[\"\",0,2]\n[\"\",1,2]\n[\"\",2,2]".to_owned();
        entrypoint(&data);
    }

    #[test]
    fn cluster_nine_points() {
        let args: Vec<Position2d> = vec![
            Position2d::new(0., 0.),
            Position2d::new(1., 0.),
            Position2d::new(2., 0.),
            Position2d::new(0., 1.),
            Position2d::new(1., 1.),
            Position2d::new(2., 1.),
            Position2d::new(0., 2.),
            Position2d::new(1., 2.),
            Position2d::new(2., 2.),
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
}
