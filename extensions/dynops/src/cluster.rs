mod coord;
mod mvee;
// mod kdtree;

use coord::Position2d;
use mvee::get_mvee;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

const EPSILON: f64 = 75.0;
const MIN_POINTS: usize = 4;

impl fmt::Display for Position2d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl std::ops::Index<usize> for Position2d {
    type Output = f64;

    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => &0.,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Area {
    pub center: Position2d,
    pub a: f64,
    pub b: f64,
    pub angle: f64,
}

impl fmt::Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "[{}, {}, {}, {}]",
            self.center, self.a, self.b, self.angle
        )
    }
}

trait Distance {
    fn get_distance(&self, other: &Self) -> f64;
}

impl Distance for Position2d {
    fn get_distance(&self, neighbor: &Position2d) -> f64 {
        ((self.x - neighbor.x).powi(2) + (self.y - neighbor.y).powi(2)).sqrt()
    }
}

fn parse_points(data: &[String]) -> Vec<Position2d> {
    data.iter()
        .map(|line| -> Position2d {
            let coord: [f64; 2] = serde_json::from_str(line).unwrap();
            Position2d {
                x: coord[0],
                y: coord[1],
            }
        })
        .collect()
}

fn format_area(area: &Area) -> String {
    format!(
        "[[{:.2},{:.2}],{:.2},{:.2},{:.2}]",
        area.center.x, area.center.y, area.a, area.b, area.angle
    )
}

pub fn entrypoint(data: &[String]) -> String {
    let points: Vec<Position2d> = parse_points(data);
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
            _ => {}
        }
    }
    let centers: Vec<String> = clusters
        .iter()
        .map(|(_, cluster_points)| get_mvee(cluster_points, 1.0))
        .map(|area| format_area(&area))
        .collect();
    format!("[\n{}\n]", centers.join(",\n"))
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

    /// Run the DBSCAN algorithm on a given population of datapoints.
    ///
    /// A vector of [`Classification`] enums is returned, where each element
    /// corresponds to a row in the input matrix.
    ///
    /// # Arguments
    /// * `population` - a matrix of datapoints, organized by rows
    ///
    /// # Example
    ///
    /// ```rust
    /// use dbscan::Classification::*;
    /// use dbscan::Model;
    ///
    /// let model = Model::new(1.0, 3);
    /// let inputs = vec![
    ///     vec![1.5, 2.2],
    ///     vec![1.0, 1.1],
    ///     vec![1.2, 1.4],
    ///     vec![0.8, 1.0],
    ///     vec![3.7, 4.0],
    ///     vec![3.9, 3.9],
    ///     vec![3.6, 4.1],
    ///     vec![10.0, 10.0],
    /// ];
    /// let output = model.run(&inputs);
    /// assert_eq!(
    ///     output,
    ///     vec![
    ///         Edge(0),
    ///         Core(0),
    ///         Core(0),
    ///         Core(0),
    ///         Core(1),
    ///         Core(1),
    ///         Core(1),
    ///         Noise
    ///     ]
    /// );
    /// ```
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

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_point_center() {
        let args: Vec<String> = vec![
            "[0,0]".to_owned(),
            "[1,0]".to_owned(),
            "[2,0]".to_owned(),
            "[0,1]".to_owned(),
            "[1,1]".to_owned(),
            "[2,1]".to_owned(),
            "[0,2]".to_owned(),
            "[1,2]".to_owned(),
            "[2,2]".to_owned(),
        ];
        assert_eq!("[1, 1]", entrypoint(&args));
    }

    #[test]
    fn cluster_nine_points() {
        let args: Vec<Position2d> = vec![
            Position2d { x: 0., y: 0. },
            Position2d { x: 1., y: 0. },
            Position2d { x: 2., y: 0. },
            Position2d { x: 0., y: 1. },
            Position2d { x: 1., y: 1. },
            Position2d { x: 2., y: 1. },
            Position2d { x: 0., y: 2. },
            Position2d { x: 1., y: 2. },
            Position2d { x: 2., y: 2. },
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
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Record {
        Type: String,
        X: f64,
        Y: f64,
    }

    fn test_cluster_csv(data: &str, exp_clusters: usize) {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(data.as_bytes());

        let mut coords: Vec<Position2d> = Vec::new();
        coords.reserve(1000);
        for result in rdr.deserialize() {
            let record: Record = result.unwrap();
            coords.push(Position2d {
                x: record.X,
                y: record.Y,
            });
        }

        let cluster_indexes = cluster(EPSILON, MIN_POINTS, &coords);

        let mut max: usize = 0;
        for index in cluster_indexes {
            match index {
                Core(i) => max = std::cmp::max(max, i),
                Edge(i) => max = std::cmp::max(max, i),
                Noise => {}
            };
        }
        assert_eq!(exp_clusters, max + 1);
    }

    #[test]
    fn test_cluster_altis() {
        test_cluster_csv(include_str!("data/buildings.Altis.csv"), 354);
    }

    #[test]
    fn test_cluster_stratis() {
        test_cluster_csv(include_str!("data/buildings.Stratis.csv"), 20);
    }

    #[test]
    fn test_cluster_livonia() {
        test_cluster_csv(include_str!("data/buildings.Enoch.csv"), 110);
    }

    #[test]
    fn test_cluster_malden() {
        test_cluster_csv(include_str!("data/buildings.Malden.csv"), 121);
    }

    #[test]
    fn test_cluster_tanoa() {
        test_cluster_csv(include_str!("data/buildings.Tanoa.csv"), 228);
    }
}
*/