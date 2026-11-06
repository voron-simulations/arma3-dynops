use criterion::{Criterion, criterion_group, criterion_main};
use dynops::geometry::{convex_hull, min_area_obb};
use dynops::locations::{DetectParams, detect};
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

fn bench_detect(c: &mut Criterion) {
    let mut group = c.benchmark_group("detect");
    let params = DetectParams::default();

    let stratis = parse(STRATIS);
    group.bench_function("stratis_332", |b| {
        b.iter(|| detect(black_box(&stratis), &params))
    });

    let malden = parse(MALDEN);
    group.bench_function("malden_2478", |b| {
        b.iter(|| detect(black_box(&malden), &params))
    });

    let altis = parse(ALTIS);
    group.sample_size(10);
    group.bench_function("altis_11771", |b| {
        b.iter(|| detect(black_box(&altis), &params))
    });

    group.finish();
}

/// A synthetic "settlement": a compact, slightly irregular ring of points, the
/// shape `min_area_obb` is actually invoked on inside `detect`.
fn ring(n: usize, radius: f64) -> Vec<Vector2<f64>> {
    (0..n)
        .map(|i| {
            let theta = i as f64 / n as f64 * std::f64::consts::TAU;
            let r = radius * (1.0 + 0.1 * (i as f64 * 2.7).sin());
            Vector2::new(r * theta.cos(), r * theta.sin())
        })
        .collect()
}

fn bench_geometry(c: &mut Criterion) {
    let mut group = c.benchmark_group("geometry");

    for &n in &[10usize, 50, 200] {
        let points = ring(n, 50.0);
        group.bench_function(format!("convex_hull_{n}"), |b| {
            b.iter(|| convex_hull(black_box(&points)))
        });
        group.bench_function(format!("min_area_obb_{n}"), |b| {
            b.iter(|| min_area_obb(black_box(&points)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_detect, bench_geometry);
criterion_main!(benches);
