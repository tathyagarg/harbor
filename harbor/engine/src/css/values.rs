/// https://www.w3.org/TR/css-values-4/

pub mod angles {
    pub const ANGLE_UNITS: &[&str] = &["deg", "grad", "rad", "turn"];

    pub fn is_angle_unit(unit: &str) -> bool {
        ANGLE_UNITS.contains(&unit)
    }

    pub fn deg_to_rad(deg: f64) -> f64 {
        deg * std::f64::consts::PI / 180.0
    }

    pub fn rad_to_deg(rad: f64) -> f64 {
        rad * 180.0 / std::f64::consts::PI
    }

    pub fn grad_to_rad(grad: f64) -> f64 {
        grad * std::f64::consts::PI / 200.0
    }

    pub fn rad_to_grad(rad: f64) -> f64 {
        rad * 200.0 / std::f64::consts::PI
    }

    pub fn turn_to_rad(turn: f64) -> f64 {
        turn * 2.0 * std::f64::consts::PI
    }

    pub fn rad_to_turn(rad: f64) -> f64 {
        rad / (2.0 * std::f64::consts::PI)
    }

    pub fn turn_to_deg(turn: f64) -> f64 {
        turn * 360.0
    }

    pub fn deg_to_turn(deg: f64) -> f64 {
        deg / 360.0
    }

    pub fn grad_to_deg(grad: f64) -> f64 {
        grad * 0.9
    }

    pub fn deg_to_grad(deg: f64) -> f64 {
        deg / 0.9
    }

    pub fn to_canonical_angle(value: f64, unit: &str) -> Option<f64> {
        match unit {
            "deg" => Some(value),
            "rad" => Some(rad_to_deg(value)),
            "grad" => Some(grad_to_deg(value)),
            "turn" => Some(turn_to_deg(value)),
            _ => None,
        }
    }
}
