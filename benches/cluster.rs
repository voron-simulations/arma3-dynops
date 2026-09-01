use criterion::{criterion_group, criterion_main, Criterion};
use dynops::cluster::{cluster, entrypoint, EPSILON, MIN_POINTS};
use dynops::kdtree::KdTree;
use nalgebra::Vector2;
use std::hint::black_box;

const STRATIS: &str = include_str!("../data/objects.Stratis.txt");
const MALDEN: &str = include_str!("../data/objects.Malden.txt");
const ALTIS: &str = include_str!("../data/objects.Altis.txt");

fn parse(data: &str) -> Vec<Vector2<f64>> {
    data.lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.split_once(','))
        .filter_map(|(x, y)| Some(Vector2::new(x.parse().ok()?, y.parse().ok()?)))
        .collect()
}

fn bench_dbscan(c: &mut Criterion) {
    let mut group = c.benchmark_group("dbscan");

    let stratis = parse(STRATIS);
    group.bench_function("stratis_332", |b| {
        b.iter(|| cluster(EPSILON, MIN_POINTS, black_box(&stratis)))
    });

    let malden = parse(MALDEN);
    group.bench_function("malden_2478", |b| {
        b.iter(|| cluster(EPSILON, MIN_POINTS, black_box(&malden)))
    });

    let altis = parse(ALTIS);
    group.sample_size(10);
    group.bench_function("altis_11771", |b| {
        b.iter(|| cluster(EPSILON, MIN_POINTS, black_box(&altis)))
    });

    group.finish();
}

fn bench_entrypoint(c: &mut Criterion) {
    let mut group = c.benchmark_group("entrypoint");

    group.bench_function("stratis_332", |b| b.iter(|| entrypoint(black_box(STRATIS))));
    group.bench_function("malden_2478", |b| b.iter(|| entrypoint(black_box(MALDEN))));

    group.sample_size(10);
    group.bench_function("altis_11771", |b| b.iter(|| entrypoint(black_box(ALTIS))));

    group.finish();
}

fn bench_kdtree(c: &mut Criterion) {
    let mut group = c.benchmark_group("kdtree");

    let altis = parse(ALTIS);

    group.bench_function("build_altis_11771", |b| {
        b.iter(|| KdTree::build(black_box(&altis)))
    });

    let tree = KdTree::build(&altis);
    // A fixed batch of queries against every point in the map, eps-sized radius.
    group.bench_function("within_altis_11771", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            for &point in black_box(&altis) {
                out.clear();
                tree.within(point, EPSILON, &mut out);
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_dbscan, bench_entrypoint, bench_kdtree);
criterion_main!(benches);
