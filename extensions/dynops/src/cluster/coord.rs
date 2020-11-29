#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position2d {
    pub x: f64,
    pub y: f64,
}

pub trait HasPosition2d {
    fn get_position_2d(&self, other: &Self) -> Position2d;
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
