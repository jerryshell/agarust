use std::f64::consts::PI;

pub fn radius_to_mass(radius: f64) -> f64 {
    PI * radius * radius
}

pub fn mass_to_radius(mass: f64) -> f64 {
    (mass / PI).sqrt()
}
