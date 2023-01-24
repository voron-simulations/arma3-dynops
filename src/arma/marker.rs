use nalgebra::{Rotation2, Vector2};
use std::fmt;

pub type Vector2<f64> = Vector2<f64>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MarkerShape {
    Icon,
    Rectangle,
    Ellipse,
    Hexagon,
    Polyline,
}

pub struct Marker {
    pub pos: Vector2<f64>,
    pub Size: Vector2<f64>,
    pub Dir: f32,
    pub Alpha: f32,

    pub Name: String,
    pub Color: String,
    pub shape: MarkerShape,
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _shape = match self.shape {
            MarkerShape::Rectangle => "RECTANGLE",
            MarkerShape::Ellipse => "ELLIPSE",
            MarkerShape::Hexagon => "ELLIPSE",
            MarkerShape::Icon => "ICON",
            MarkerShape::Polyline => "PolyLine",
        };

        let pos = if self.shape == MarkerShape::Hexagon {
            -self.pos
        } else {
            self.pos
        };

        write!(f, "|{}|[{},{}]||", self.Name, pos[0], pos[1],)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapObject {
    pub name: String,
    pub position: Vector2<f64>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AreaKind {
    Rectangle,
    Ellipse,
    Hexagon,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Area {
    // Center position
    pub center: Vector2<f64>,
    // half-size along x/y axes
    pub xsize: f64,
    pub ysize: f64,
    // Counterclockwise rotation in radians
    pub angle: f64,
    pub kind: AreaKind,
}

impl Area {
    pub fn contains(&self, point: &Vector2<f64>) -> bool {
        self.contains_tolerance(point, 0.)
    }

    pub fn contains_tolerance(&self, point: &Vector2<f64>, tolerance: f64) -> bool {
        let axis_aligned_relative_pos = point - self.center;
        let rotation = Rotation2::new(-self.angle);
        let relative_position = rotation * axis_aligned_relative_pos;
        let tolerance_quotient = 1. + tolerance.abs();
        match self.kind {
            AreaKind::Rectangle => {
                relative_position.x.abs() <= self.xsize * tolerance_quotient
                    && relative_position.y.abs() <= self.ysize * tolerance_quotient
            }
            AreaKind::Ellipse => {
                sqr(relative_position.x / self.xsize) + sqr(relative_position.y / self.ysize)
                    <= tolerance_quotient
            }
            AreaKind::Hexagon => panic!("Not implemented"),
        }
    }
}

pub trait HasPosition {
    fn get_position(&self) -> Vector2<f64>;
}

impl HasPosition for MapObject {
    fn get_position(&self) -> Vector2<f64> {
        self.position
    }
}

impl HasPosition for Area {
    fn get_position(&self) -> Vector2<f64> {
        self.center
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
            self.center.x, self.center.y, self.xsize, self.ysize, self.angle
        )
    }
}

pub fn distance(a1: Area, a2: Area) -> f64 {
    let d = sqr(a1.center.x - a2.center.x)
        + sqr(a1.center.y - a2.center.y)
        + sqr(a1.xsize - a2.xsize)
        + sqr(a1.ysize - a2.ysize)
        + sqr(a1.angle - a2.angle);
    d.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_4;

    #[test]
    fn test_ellipse_area_contains() {
        let area = Area {
            center: Vector2::new(2., 0.),
            xsize: 2.,
            ysize: 1.,
            kind: AreaKind::Ellipse,
            angle: 0.,
        };
        assert!(area.contains(&Vector2::new(0., 0.)));
        assert!(area.contains(&Vector2::new(2., 0.)));
        assert!(area.contains(&Vector2::new(4., 0.)));
        assert!(area.contains(&Vector2::new(2., 1.)));
        assert!(area.contains(&Vector2::new(2., -1.)));

        assert!(!area.contains(&Vector2::new(5., 1.)));
        assert!(!area.contains(&Vector2::new(1., -1.)));
        assert!(!area.contains(&Vector2::new(5., 0.)));
        assert!(!area.contains(&Vector2::new(2., 2.)));
    }

    #[test]
    fn test_rotated_ellipse_area_contains() {
        let area = Area {
            center: Vector2::new(5., 5.),
            xsize: 4.0,
            ysize: 0.1,
            kind: AreaKind::Ellipse,
            angle: FRAC_PI_4,
        };
        assert!(area.contains(&Vector2::new(6., 6.)));
        assert!(area.contains(&Vector2::new(4., 4.)));

        assert!(!area.contains(&Vector2::new(6., 4.)));
        assert!(!area.contains(&Vector2::new(4., 6.)));
    }

    #[test]
    fn test_rectangle_area_contains() {
        let area = Area {
            center: Vector2::new(2., 0.),
            xsize: 2.,
            ysize: 1.,
            kind: AreaKind::Rectangle,
            angle: 0.,
        };
        assert!(area.contains(&Vector2::new(0., 0.)));
        assert!(area.contains(&Vector2::new(2., 0.)));
        assert!(area.contains(&Vector2::new(4., 0.)));
        assert!(area.contains(&Vector2::new(2., 1.)));
        assert!(area.contains(&Vector2::new(2., -1.)));
        assert!(area.contains(&Vector2::new(1., 1.)));
        assert!(area.contains(&Vector2::new(1., -1.)));

        assert!(!area.contains(&Vector2::new(-3., 0.)));
        assert!(!area.contains(&Vector2::new(-5., 0.)));
    }

    #[test]
    fn test_rotated_rectangle_area_contains() {
        let area = Area {
            center: Vector2::new(5., 5.),
            xsize: 4.0,
            ysize: 0.1,
            kind: AreaKind::Rectangle,
            angle: FRAC_PI_4,
        };
        assert!(area.contains(&Vector2::new(6., 6.)));
        assert!(area.contains(&Vector2::new(4., 4.)));

        assert!(!area.contains(&Vector2::new(6., 4.)));
        assert!(!area.contains(&Vector2::new(4., 6.)));
    }
}
