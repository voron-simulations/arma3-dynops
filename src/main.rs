//! Offline tuning tool: renders raw building points plus detected settlement
//! OBBs (colour-coded by class) so detection parameters can be validated
//! against the bundled map dumps without launching Arma.

use anyhow::{Context, Result};
use dynops::locations::{DetectParams, Location, LocationClass, detect};
use image::{ImageFormat, Rgba, RgbaImage};
use nalgebra::Vector2;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut input: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--out" {
            out = Some(PathBuf::from(
                args.next().context("--out requires a path argument")?,
            ));
        } else {
            input = Some(arg);
        }
    }
    let input = input.context("usage: dynops-test <input> [--out <path>]")?;
    let out = out.unwrap_or_else(|| env::temp_dir().join("output.png"));

    let data = fs::read_to_string(&input).with_context(|| format!("failed to read {input}"))?;
    let points = parse_points(&data)?;
    tracing::info!(count = points.len(), "parsed input points");

    let locations = detect(&points, &DetectParams::default());
    tracing::info!(count = locations.len(), "detected locations");

    render(&points, &locations, &out)?;
    tracing::info!(path = %out.display(), "wrote image");
    Ok(())
}

fn parse_points(data: &str) -> Result<Vec<Vector2<f64>>> {
    data.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (x, y) = line
                .split_once(',')
                .with_context(|| format!("expected \"x,y\", got {line:?}"))?;
            Ok(Vector2::new(
                x.trim()
                    .parse::<f64>()
                    .with_context(|| format!("bad x coordinate in {line:?}"))?,
                y.trim()
                    .parse::<f64>()
                    .with_context(|| format!("bad y coordinate in {line:?}"))?,
            ))
        })
        .collect()
}

fn class_color(class: LocationClass) -> Rgba<u8> {
    match class {
        LocationClass::Farm => Rgba([255, 255, 0, 255]), // yellow
        LocationClass::Hamlet => Rgba([255, 165, 0, 255]), // orange
        LocationClass::Village => Rgba([255, 0, 0, 255]), // red
        LocationClass::Town => Rgba([255, 0, 255, 255]), // magenta
        LocationClass::City => Rgba([0, 128, 255, 255]), // blue
    }
}

/// Map-space (x, y with y-up) to image-space (pixels with y-down) conversion.
struct Transform {
    min_x: f64,
    max_y: f64,
    scale: f64,
}

impl Transform {
    fn apply(&self, p: Vector2<f64>) -> (i64, i64) {
        (
            ((p.x - self.min_x) * self.scale).round() as i64,
            ((self.max_y - p.y) * self.scale).round() as i64,
        )
    }
}

fn put_pixel_checked(image: &mut RgbaImage, x: i64, y: i64, color: Rgba<u8>) {
    if x < 0 || y < 0 {
        return;
    }
    let (x, y) = (x as u32, y as u32);
    if x < image.width() && y < image.height() {
        image.put_pixel(x, y, color);
    }
}

fn draw_line(image: &mut RgbaImage, from: (i64, i64), to: (i64, i64), color: Rgba<u8>) {
    // Bresenham's line algorithm; corners may land a pixel or two outside the
    // frame for OBBs near the map edge, which put_pixel_checked just drops.
    let (mut x0, mut y0) = from;
    let (x1, y1) = to;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        put_pixel_checked(image, x0, y0, color);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn draw_obb(image: &mut RgbaImage, transform: &Transform, location: &Location) {
    let (sin, cos) = location.obb.angle.sin_cos();
    let local = [
        Vector2::new(-location.obb.a, -location.obb.b),
        Vector2::new(location.obb.a, -location.obb.b),
        Vector2::new(location.obb.a, location.obb.b),
        Vector2::new(-location.obb.a, location.obb.b),
    ];
    let corners: Vec<(i64, i64)> = local
        .iter()
        .map(|p| {
            let world =
                location.obb.center + Vector2::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos);
            transform.apply(world)
        })
        .collect();
    let color = class_color(location.class);
    for i in 0..corners.len() {
        draw_line(image, corners[i], corners[(i + 1) % corners.len()], color);
    }
}

fn render(points: &[Vector2<f64>], locations: &[Location], out: &Path) -> Result<()> {
    let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
    anyhow::ensure!(min_x.is_finite(), "no points to render");

    let max_dimension = 1024.0;
    let scale = (max_dimension / (max_x - min_x + 1.0)).min(max_dimension / (max_y - min_y + 1.0));
    let size_x = ((max_x - min_x) * scale) as u32 + 1;
    let size_y = ((max_y - min_y) * scale) as u32 + 1;

    let mut image = RgbaImage::new(size_x, size_y);
    let transform = Transform {
        min_x,
        max_y,
        scale,
    };

    for &p in points {
        let (x, y) = transform.apply(p);
        put_pixel_checked(&mut image, x, y, Rgba([0, 255, 0, 255]));
    }

    for location in locations {
        draw_obb(&mut image, &transform, location);
    }

    image
        .save_with_format(out, ImageFormat::Png)
        .with_context(|| format!("failed to save {}", out.display()))
}
