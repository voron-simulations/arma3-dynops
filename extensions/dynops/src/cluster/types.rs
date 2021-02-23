use nalgebra::{Rotation2, Vector2};
use std::fmt;

pub type Position2d = Vector2<f64>;

#[derive(Debug, Clone, PartialEq)]
pub struct MapObject {
    pub name: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Area {
    // Center position
    pub x: f64,
    pub y: f64,
    // half-size along x/y axes
    pub a: f64,
    pub b: f64,
    // Rotation angle in radians
    pub angle: f64,
    // Is ellipse or rectangle
    pub is_ellipse: bool,
}

pub fn area_contains(area: &Area, point: Position2d) -> bool {
    // Axis-aligned relative position in global frame
    let aa_rel_pos = point - area.get_position();
    let rotation = Rotation2::new(area.angle);
    // Relative position in area's frame of reference
    let rel_pos = rotation * aa_rel_pos;
    if area.is_ellipse {
        sqr(rel_pos.x / area.a) + sqr(rel_pos.y / area.b) <= 1.
    } else {
        rel_pos.y.abs() <= area.a && rel_pos.y.abs() <= area.b
    }
}

pub trait HasPosition {
    fn get_position(&self) -> Position2d;
}

impl HasPosition for MapObject {
    fn get_position(&self) -> Position2d {
        Vector2::new(self.x, self.y)
    }
}

impl HasPosition for Area {
    fn get_position(&self) -> Position2d {
        Vector2::new(self.x, self.y)
    }
}

pub trait Distance {
    fn get_distance(&self, other: &Self) -> f64;
}

impl Distance for Position2d {
    fn get_distance(&self, neighbor: &Position2d) -> f64 {
        (sqr(self.x - neighbor.x) + sqr(self.y - neighbor.y)).sqrt()
    }
}

fn sqr(f: f64) -> f64 {
    f * f
}

impl fmt::Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "[[{}, {}], {}, {}, {}]",
            self.x,
            self.y,
            self.a,
            self.b,
            self.angle * 180. / std::f64::consts::PI
        )
    }
}

pub fn distance(a1: Area, a2: Area) -> f64 {
    let d = sqr(a1.x - a2.x)
        + sqr(a1.y - a2.y)
        + sqr(a1.a - a2.a)
        + sqr(a1.b - a2.b)
        + sqr(a1.angle - a2.angle);
    d.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipse_area_contains() {
        let area = Area {
            x: 2.,
            y: 0.,
            a: 2.,
            b: 1.,
            is_ellipse: true,
            angle: 0.,
        };
        assert!(area_contains(&area, Position2d::new(0., 0.)));
        assert!(area_contains(&area, Position2d::new(2., 0.)));
        assert!(area_contains(&area, Position2d::new(4., 0.)));
        assert!(area_contains(&area, Position2d::new(2., 1.)));
        assert!(area_contains(&area, Position2d::new(2., -1.)));

        assert!(!area_contains(&area, Position2d::new(1., 1.)));
        assert!(!area_contains(&area, Position2d::new(1., -1.)));
        assert!(!area_contains(&area, Position2d::new(5., 0.)));
        assert!(!area_contains(&area, Position2d::new(2., 2.)));
    }

    #[test]
    fn test_square_area_contains() {
        let area = Area {
            x: 2.,
            y: 0.,
            a: 2.,
            b: 1.,
            is_ellipse: false,
            angle: 0.,
        };
        assert!(area_contains(&area, Position2d::new(0., 0.)));
        assert!(area_contains(&area, Position2d::new(2., 0.)));
        assert!(area_contains(&area, Position2d::new(4., 0.)));
        assert!(area_contains(&area, Position2d::new(2., 1.)));
        assert!(area_contains(&area, Position2d::new(2., -1.)));
        assert!(area_contains(&area, Position2d::new(1., 1.)));
        assert!(area_contains(&area, Position2d::new(1., -1.)));
    }

    #[test]
    fn test_rotated_ellipse_area_contains() {
        let area = Area {
            x: 10.,
            y: 10.,
            a: 5.,
            b: 1.,
            is_ellipse: true,
            angle: -std::f64::consts::FRAC_PI_4,
        };
        assert!(area_contains(&area, Position2d::new(12., 12.)));
        assert!(area_contains(&area, Position2d::new(8., 8.)));

        assert!(!area_contains(&area, Position2d::new(12., 8.)));
        assert!(!area_contains(&area, Position2d::new(8., 12.)));
    }
}
