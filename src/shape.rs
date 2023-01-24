use std::fmt;

use nalgebra::{Rotation2, Vector2};

pub trait Shape {
    fn contains_xy(&self, x: f64, y: f64) -> String;
    fn contains_vec(&self, pos: &Vector2<f64>) -> String;
}

pub struct Ellipse {
    pub x: f64,
    pub y: f64,
    pub a: f64,
    pub b: f64,
    pub r: f64,
}

impl fmt::Display for Ellipse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}±{}, {}±{}, α={}]", self.x, self.a, self.y, self.b, self.r)
    }
}

impl Ellipse {
    pub fn new(x: f64, y: f64, a: f64, b: f64, r: f64) -> Ellipse {
        Ellipse { x, y, a, b, r }
    }

    pub fn contains_xy(&self, x: f64, y: f64) -> bool {
        Self::contains_vec(self, &Vector2::new(x, y))
    }

    pub fn contains_vec(&self, pos: &Vector2<f64>) -> bool {
        let rot = Rotation2::new(-self.r);
        let rel_pos = rot.transform_vector(&(pos - Vector2::new(self.x, self.y)));

        f64::powf(rel_pos.x / self.a, 2.) + f64::powf(rel_pos.y / self.b, 2.) <= 1.
    }
}
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub a: f64,
    pub b: f64,
    pub r: f64,
}

impl Rectangle {
    pub fn new(x: f64, y: f64, a: f64, b: f64, r: f64) -> Rectangle {
        Rectangle { x, y, a, b, r }
    }

    pub fn contains_xy(&self, x: f64, y: f64) -> bool {
        Self::contains_vec(self, &Vector2::new(x, y))
    }

    pub fn contains_vec(&self, pos: &Vector2<f64>) -> bool {
        let rot = Rotation2::new(-self.r);
        let rel_pos = rot.transform_vector(&(pos - Vector2::new(self.x, self.y)));

        rel_pos.x.abs() <= self.a && rel_pos.y.abs() <= self.b
    }
}

#[cfg(test)]
mod tests {
    use na::Rotation2;
    use na::Vector2;
    use nalgebra as na;

    use super::Ellipse;
    use super::Rectangle;

    #[test]
    fn nalgebra_sanity() {
        let vec = Vector2::new(1., 0.);
        let rot = Rotation2::new(std::f64::consts::FRAC_PI_2);
        let result = rot.transform_vector(&vec);
        assert_eq!(result, Vector2::new(0., 1.));
    }

    #[test]
    fn ellipse_contains() {
        let e1 = Ellipse::new(10., 5., 5., 1., 0.);
        assert!(e1.contains_xy(10., 5.));
        assert!(e1.contains_xy(5., 5.));
        assert!(e1.contains_xy(15., 5.));
        assert!(e1.contains_xy(10., 4.));
        assert!(e1.contains_xy(10., 6.));

        assert!(!e1.contains_xy(11., 6.));
        assert!(!e1.contains_xy(9., 4.));

        let e2 = Ellipse::new(10., 5., 5., 1., std::f64::consts::FRAC_PI_4);
        assert!(e2.contains_xy(10., 5.));
        assert!(!e2.contains_xy(8., 5.));
        assert!(!e2.contains_xy(12., 5.));
        assert!(!e2.contains_xy(10., 3.));
        assert!(!e2.contains_xy(10., 7.));

        assert!(e2.contains_xy(11., 6.));
        assert!(e2.contains_xy(9., 4.));
    }

    #[test]
    fn rectangle_contains() {
        let e1 = Rectangle::new(10., 5., 5., 1., 0.);
        assert!(e1.contains_xy(10., 5.));
        assert!(e1.contains_xy(5., 5.));
        assert!(e1.contains_xy(15., 5.));
        assert!(e1.contains_xy(10., 4.));
        assert!(e1.contains_xy(10., 6.));

        assert!(!e1.contains_xy(11., 6.));
        assert!(!e1.contains_xy(9., 4.));

        let e2 = Rectangle::new(10., 5., 5., 1., std::f64::consts::FRAC_PI_4);
        assert!(e2.contains_xy(10., 5.));
        assert!(!e2.contains_xy(8., 5.));
        assert!(!e2.contains_xy(12., 5.));
        assert!(!e2.contains_xy(10., 3.));
        assert!(!e2.contains_xy(10., 7.));

        assert!(e2.contains_xy(11., 6.));
        assert!(e2.contains_xy(9., 4.));
    }
}
