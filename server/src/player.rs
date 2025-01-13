use crate::*;

const INIT_RADIUS: f64 = 20.0;
const INIT_DIRECTION_ANGLE: f64 = 0.0;
const INIT_SPEED: f64 = 150.0;

#[derive(Debug, Clone)]
pub struct Player {
    pub db_id: i64,
    pub connection_id: String,
    pub nickname: String,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub direction_angle: f64,
    pub speed: f64,
    pub color: i32,
}

impl Player {
    pub fn random(db_id: i64, connection_id: String, nickname: String, color: i32) -> Self {
        Self {
            db_id,
            connection_id,
            nickname,
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

    pub fn increase_mass(&mut self, mass: f64) {
        let mut player_mass = util::radius_to_mass(self.radius);
        player_mass += mass;

        self.radius = util::mass_to_radius(player_mass);
    }

    pub fn try_decrease_mass(&mut self, mass: f64) -> bool {
        if self.radius <= 10.0 {
            return false;
        }

        let mut player_mass = util::radius_to_mass(self.radius);
        player_mass -= mass;
        if player_mass <= 0.0 {
            return false;
        }

        self.radius = util::mass_to_radius(player_mass);

        true
    }

    pub fn try_drop_mass(&mut self) -> Option<f64> {
        let target_radius = (5.0 + self.radius / 50.0).min(15.0);
        let target_mass = util::radius_to_mass(target_radius);
        if self.try_decrease_mass(target_mass) {
            return Some(target_mass);
        }
        None
    }
}
