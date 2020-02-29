use cgmath::InnerSpace;
use cgmath::Vector3;

/// Clamps a value x to be in the range (low, high)
// `f32.clamp` is nightly-only :(
pub fn clamp(x: f32, low: f32, high: f32) -> f32 {
    if x < low {
        low
    } else if x > high {
        high
    } else {
        x
    }
}

pub fn reflect(v: Vector3<f32>, normal: Vector3<f32>) -> Vector3<f32> {
    (v - 2.0 * InnerSpace::dot(v, normal) * normal).normalize()
}

#[cfg(test)]
mod tests {
    use super::{clamp, reflect};
    use cgmath::MetricSpace;
    use cgmath::Vector3;

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(100., 0., 1.), 1.0);
        assert_eq!(clamp(-100., 0., 1.), 0.);
        assert_eq!(clamp(0.5, 0., 1.), 0.5);
    }

    #[test]
    fn test_reflect() {
        let v = (1.0, 0.0, 0.0).into();
        let n = 2_f32.sqrt() / 2.0 * Vector3::new(-1.0, 1.0, 0.0);
        assert!(reflect(v, n).distance((0.0, 1.0, 0.0).into()) < 1e-5);
    }
}
