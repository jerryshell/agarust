use crate::*;

use nanoid::nanoid;

const SPORE_BOUND: f64 = 3000.0;

fn random_xy() -> f64 {
    (rand::random::<f64>() * 2.0 - 1.0) * SPORE_BOUND
}

#[derive(Debug, Clone)]
pub struct Spore {
    pub id: Arc<str>,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

impl Spore {
    pub fn random() -> Self {
        let radius = (rand::random::<f64>() * 3.0 + 10.0).max(5.0);
        Self {
            id: nanoid!().into(),
            x: random_xy(),
            y: random_xy(),
            radius,
        }
    }
}
