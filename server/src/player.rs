use crate::*;

const PLAYER_BOUND: f64 = 3000.0;
const INIT_RADIUS: f64 = 20.0;
const INIT_DIRECTION_ANGLE: f64 = 0.0;
const INIT_SPEED: f64 = 150.0;

fn random_xy() -> f64 {
    (rand::random::<f64>() * 2.0 - 1.0) * PLAYER_BOUND
}

#[derive(Debug, Clone)]
pub struct Player {
    pub db_id: i64,
    pub connection_id: Arc<str>,
    pub nickname: String,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub direction_angle: f64,
    pub speed: f64,
    pub color: i64,
}

impl Player {
    pub fn random(db_id: i64, connection_id: Arc<str>, nickname: String, color: i64) -> Self {
        Self {
            db_id,
            connection_id,
            nickname,
            x: random_xy(),
            y: random_xy(),
            radius: INIT_RADIUS,
            direction_angle: INIT_DIRECTION_ANGLE,
            speed: INIT_SPEED,
            color,
        }
    }

    pub fn respawn(&mut self) {
        self.x = random_xy();
        self.y = random_xy();
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
