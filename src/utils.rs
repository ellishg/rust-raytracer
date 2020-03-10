use cgmath::{InnerSpace, Matrix4, SquareMatrix, Transform, Vector3};

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

pub fn safe_inverse(mat: Matrix4<f32>) -> Matrix4<f32> {
    mat.inverse_transform().unwrap_or(Matrix4::identity())
}

#[cfg(test)]
mod tests {
    use super::{clamp, reflect, safe_inverse};
    use cgmath::num_traits::identities::Zero;
    use cgmath::Angle;
    use cgmath::Matrix4;
    use cgmath::MetricSpace;
    use cgmath::Rad;
    use cgmath::SquareMatrix;
    use cgmath::Transform;
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

    #[test]
    fn test_safe_inverse() {
        assert_eq!(safe_inverse(Matrix4::zero()), Matrix4::identity());
        let x: Matrix4<f32> = Matrix4::from_angle_x(Rad(1.0));
        assert_eq!(safe_inverse(x), x.inverse_transform().unwrap());
    }
}
