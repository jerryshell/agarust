use crate::*;

const INIT_RADIUS: f64 = 20.0;
const INIT_DIRECTION_ANGLE: f64 = 0.0;
const INIT_SPEED: f64 = 150.0;

#[derive(Debug, Clone)]
pub struct Player {
    pub connection_id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub direction_angle: f64,
    pub speed: f64,
    pub color: i32,
}

impl Player {
    pub fn random(connection_id: String, name: String, color: i32) -> Self {
        Self {
            connection_id,
            name,
            x: 0.0,
            y: 0.0,
            radius: INIT_RADIUS,
            direction_angle: INIT_DIRECTION_ANGLE,
            speed: INIT_SPEED,
            color,
        }
    }

    pub fn respawn(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.radius = INIT_RADIUS;
        self.speed = INIT_SPEED;
    }

    pub fn add_mass(&mut self, mass: f64) {
        let mut player_mass = radius_to_mass(self.radius);
        player_mass += mass;
        self.radius = mass_to_radius(player_mass);
    }
}
