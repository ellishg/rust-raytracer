use cgmath::{InnerSpace, Point3, Vector3};

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

/// Returns the min and the max of each dimension for the collection of points.
pub fn component_wise_range(points: Vec<Point3<f32>>) -> (Point3<f32>, Point3<f32>) {
    assert!(!points.is_empty());
    let inf = (std::f32::INFINITY, std::f32::INFINITY, std::f32::INFINITY).into();
    points.into_iter().fold((inf, -1.0 * inf), |(min, max), v| {
        let min = (min.x.min(v.x), min.y.min(v.y), min.z.min(v.z)).into();
        let max = (max.x.max(v.x), max.y.max(v.y), max.z.max(v.z)).into();
        (min, max)
    })
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
