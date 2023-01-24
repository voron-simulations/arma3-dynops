use image::{ImageFormat, Rgba, RgbaImage};
use std::{fs, env};
use std::path::Path;

fn main() {
    let input = std::env::args().nth(1).expect("No input file given");

    let data = fs::read_to_string(input).expect("Something went wrong reading the file");

    let lines: Vec<&str> = data.lines().collect();
    let mut points: Vec<(f64, f64)> = Vec::new();
    points.reserve_exact(lines.len());
    for line in lines {
        let parts = line.split_once(',').unwrap();
        let x: f64 = parts.0.parse::<f64>().unwrap();
        let y: f64 = parts.1.parse::<f64>().unwrap();
        points.push((x, y));
    }

    let min_x = points.iter().map(|p| -> i32 { p.0 as i32 }).min().unwrap();
    let max_x = points.iter().map(|p| -> i32 { p.0 as i32 }).max().unwrap();
    let min_y = points.iter().map(|p| -> i32 { p.1 as i32 }).min().unwrap();
    let max_y = points.iter().map(|p| -> i32 { p.1 as i32 }).max().unwrap();
    let delta_x = max_x - min_x;
    let delta_y = max_y - min_y;

    let max_dimension = 1024;

    let scale_x = max_dimension as f64 / (max_x - min_x + 1) as f64;
    let scale_y = max_dimension as f64 / (max_y - min_y + 1) as f64;
    let scale = scale_x.min(scale_y);

    let size_x = (delta_x as f64 * scale) as u32 + 1;
    let size_y = (delta_y as f64 * scale) as u32 + 1;

    let mut image = RgbaImage::new(size_x, size_y);

    for p in points.iter() {
        let x = ((p.0 - min_x as f64) * scale) as u32;
        let y = ((max_y as f64 - p.1) * scale) as u32;

        assert!(x < size_x);
        assert!(y < size_y);

        image.put_pixel(x, y, Rgba([0, 255, 0, 255]));
    }
    let path = Path::new(&env::temp_dir()).join("output.png");
    image.save_with_format(&path, ImageFormat::Png).unwrap();
}
