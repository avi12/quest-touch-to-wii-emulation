use super::Vec3;

/// `SteamVR`'s world up axis.
pub const WORLD_UP: Vec3 = [0.0, 1.0, 0.0];

#[must_use]
pub fn dot(first: Vec3, second: Vec3) -> f64 {
    first[0] * second[0] + first[1] * second[1] + first[2] * second[2]
}

#[must_use]
pub fn norm(vector: Vec3) -> f64 {
    dot(vector, vector).sqrt()
}

#[must_use]
pub fn scale(vector: Vec3, factor: f64) -> Vec3 {
    vector.map(|component| component * factor)
}

#[must_use]
pub fn normalize(vector: Vec3) -> Vec3 {
    scale(vector, 1.0 / norm(vector))
}

#[must_use]
pub fn add(first: Vec3, second: Vec3) -> Vec3 {
    [
        first[0] + second[0],
        first[1] + second[1],
        first[2] + second[2],
    ]
}

#[must_use]
pub fn subtract(first: Vec3, second: Vec3) -> Vec3 {
    [
        first[0] - second[0],
        first[1] - second[1],
        first[2] - second[2],
    ]
}

#[must_use]
pub fn cross(first: Vec3, second: Vec3) -> Vec3 {
    [
        first[1] * second[2] - first[2] * second[1],
        first[2] * second[0] - first[0] * second[2],
        first[0] * second[1] - first[1] * second[0],
    ]
}

/// `vector` without its part along the unit `axis`.
#[must_use]
pub fn remove_component(vector: Vec3, axis: Vec3) -> Vec3 {
    subtract(vector, scale(axis, dot(vector, axis)))
}

/// Angle in radians between two non-zero vectors of any length.
#[must_use]
pub fn angle_between(first: Vec3, second: Vec3) -> f64 {
    (dot(first, second) / (norm(first) * norm(second)))
        .clamp(-1.0, 1.0)
        .acos()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn basic_operations() {
        assert_eq!(dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]), 32.0);
        assert_eq!(norm([3.0, 4.0, 12.0]), 13.0);
        assert_eq!(add([1.0, 2.0, 3.0], [1.0, 1.0, 1.0]), [2.0, 3.0, 4.0]);
        assert_eq!(subtract([1.0, 2.0, 3.0], [1.0, 1.0, 1.0]), [0.0, 1.0, 2.0]);
        assert_eq!(scale([1.0, -2.0, 3.0], 2.0), [2.0, -4.0, 6.0]);
        assert_eq!(cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn normalize_gives_unit_length() {
        assert!((norm(normalize([2.0, -7.0, 0.5])) - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn remove_component_leaves_perpendicular_part() {
        assert_eq!(remove_component([1.0, 2.0, 3.0], WORLD_UP), [1.0, 0.0, 3.0]);
    }

    #[test]
    fn angle_between_ignores_length() {
        let angle = angle_between([2.0, 0.0, 0.0], [0.0, 0.0, 5.0]);
        assert!((angle - std::f64::consts::FRAC_PI_2).abs() < TOLERANCE);
    }
}
