use super::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position2d {
    pub x: f64,
    pub y: f64,
}

pub trait HasPosition2d {
    fn get_position_2d(&self) -> Position2d;
}

pub trait Distance {
    fn get_distance(&self, other: &Self) -> f64;
}

impl Distance for Position2d {
    fn get_distance(&self, other: &Position2d) -> f64 {
        let dx2 = (self.x - other.x).powi(2);
        let dy2 = (self.y - other.y).powi(2);
        (dx2 + dy2).sqrt()
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

impl Display for Position2d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}