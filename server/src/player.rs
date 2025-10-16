use crate::*;
use std::time::Duration;
use tokio::time::Instant;

const PLAYER_BOUND: f64 = 3000.0;
const INIT_RADIUS: f64 = 20.0;
const INIT_DIRECTION_ANGLE: f64 = 0.0;
const INIT_SPEED: f64 = 150.0;
const RUSH_SPEED: f64 = 300.0;
const RUSH_DURATION: Duration = Duration::from_secs(2);

fn random_xy() -> f64 {
    (rand::random::<f64>() * 2.0 - 1.0) * PLAYER_BOUND
}

#[derive(Debug, Clone)]
pub struct Player {
    pub db_id: i64,
    pub connection_id: Arc<str>,
    pub nickname: Arc<str>,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub direction_angle: f64,
    pub speed: f64,
    pub color: i64,
    pub rush_instant: Option<Instant>,
}

impl Player {
    pub fn random(db_id: i64, connection_id: Arc<str>, nickname: Arc<str>, color: i64) -> Self {
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
            rush_instant: None,
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        let delta_secs = delta.as_secs_f64();

        let new_x = self.x + self.speed * self.direction_angle.cos() * delta_secs;
        let new_y = self.y + self.speed * self.direction_angle.sin() * delta_secs;

        self.x = new_x;
        self.y = new_y;

        if let Some(rush_instant) = self.rush_instant
            && rush_instant.elapsed() > RUSH_DURATION
        {
            self.speed = INIT_SPEED;
            self.rush_instant = None;
        }
    }

    pub fn rush(&mut self) {
        self.speed = RUSH_SPEED;
        self.rush_instant = Some(Instant::now());
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

    pub fn try_drop_mass(&mut self, mass: f64) -> Option<f64> {
        if self.try_decrease_mass(mass) {
            return Some(mass);
        }
        None
    }
}
