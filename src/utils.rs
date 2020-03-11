use cgmath::{InnerSpace, Vector3};

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
    (v - 2.0 * v.dot(normal) * normal).normalize()
}

pub fn refract(v: Vector3<f32>, normal: Vector3<f32>, refraction_index: f32) -> Vector3<f32> {
    // The refraction index for air is about 1.0.
    let n = if v.dot(normal) <= 0.0 {
        // Ray is entering surface.
        1.0 / refraction_index
    } else {
        // Ray is exiting surface.
        refraction_index / 1.0
    };
    // Snell's Law.
    let cos_theta_in = v.dot(normal).abs();
    let cos_theta_out = (1.0 - n.powf(2.0) * (1.0 - cos_theta_in.powf(2.0))).sqrt();
    (v * n + (n * cos_theta_in - cos_theta_out) * normal).normalize()
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
