use std::f64::consts::PI;

pub fn radius_to_mass(radius: f64) -> f64 {
    PI * radius * radius
}

pub fn mass_to_radius(mass: f64) -> f64 {
    (mass / PI).sqrt()
}

pub fn check_distance_is_close(
    x1: f64,
    y1: f64,
    radius1: f64,
    x2: f64,
    y2: f64,
    radius2: f64,
) -> bool {
    let distance_sq = (x1 - x2).powi(2) + (y1 - y2).powi(2);

    let threshold = radius1 + radius2 + 10.0;
    let threshold_sq = threshold.powi(2);

    distance_sq < threshold_sq
}
