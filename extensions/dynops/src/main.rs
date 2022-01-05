use image::{ImageFormat, Rgba, RgbaImage};
use std::fs;
use std::path::Path;

fn main() {
    let input = std::env::args().nth(1).expect("No input file given");

    let data = fs::read_to_string(input).expect("Something went wrong reading the file");

    let lines: Vec<&str> = data.lines().collect();
    let points: Vec<(String, f64, f64)> = lines
        .iter()
        .map(|line| -> (String, f64, f64) {
            let coord: (String, f64, f64) = serde_json::from_str(line).unwrap();
            coord
        })
        .collect();

    let min_x = points.iter().map(|p| -> i32 { p.1 as i32 }).min().unwrap();
    let max_x = points.iter().map(|p| -> i32 { p.1 as i32 }).max().unwrap();
    let min_y = points.iter().map(|p| -> i32 { p.2 as i32 }).min().unwrap();
    let max_y = points.iter().map(|p| -> i32 { p.2 as i32 }).max().unwrap();
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
        let x = ((p.1 - min_x as f64) * scale) as u32;
        let y = ((max_y as f64 - p.2) * scale) as u32;

        assert!(x < size_x);
        assert!(y < size_y);

        image.put_pixel(x, y, Rgba([0, 255, 0, 255]));
    }
    let path = Path::new("C:\\Users\\Oleg\\AppData\\Local\\Temp\\output.png");
    image.save_with_format(&path, ImageFormat::Png).unwrap();
}
