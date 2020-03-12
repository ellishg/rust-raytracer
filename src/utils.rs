use cgmath::{InnerSpace, Point3, Vector3, Matrix4, Transform};

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

/// Returns the (min, max) of each dimension for the collection of points.
pub fn component_wise_range(mut points: Vec<Point3<f32>>) -> (Point3<f32>, Point3<f32>) {
    let v = points.pop().expect("points cannot be empty!");
    let min = points.iter().fold(v, |a, b| {
        (f32::min(a.x, b.x), f32::min(a.y, b.y), f32::min(a.z, b.z)).into()
    });
    let max = points.iter().fold(v, |a, b| {
        (f32::max(a.x, b.x), f32::max(a.y, b.y), f32::max(a.z, b.z)).into()
    });
    (min, max)
}

/// Get the scaling factor that a matrix `mat` has on the unit vectors.
/// Returns a triple representing how much x, y, and z are scaled.
pub fn get_axis_scaling(mat: &Matrix4<f32>) -> Vector3<f32> {
    let get_scaling = |vec: Vector3<f32>| {
        let new_vec = mat.transform_vector(vec);
        new_vec.magnitude()
    };
    (
        get_scaling(Vector3::unit_x()),
        get_scaling(Vector3::unit_y()),
        get_scaling(Vector3::unit_z())
    ).into()
}

#[cfg(test)]
mod tests {
    use super::{clamp, reflect, component_wise_range, get_axis_scaling};
    use cgmath::{MetricSpace, Vector3, Matrix4, Deg, assert_abs_diff_eq, Transform};

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
    fn test_component_wise_range() {
        let points = vec![
            (-1., 0., 1.).into(),
            (0., -1., 0.).into(),
            (1., 0., 0.).into(),
        ];
        let range = component_wise_range(points);
        assert_eq!(range, ((-1., -1., 0.).into(), (1., 0., 1.).into()));
    }

    #[test]
    fn test_get_scaling() {
        let rotate =
            Matrix4::from_angle_x(Deg(120.0)) *
            Matrix4::from_angle_y(Deg(90.0)) *
            Matrix4::from_angle_z(Deg(45.0));

        let translate = Matrix4::from_translation((1.0, 0.5, -2.0).into());
        let transform = rotate * translate;
        assert_abs_diff_eq!(get_axis_scaling(&transform), (1., 1., 1.).into());

        let translate = Matrix4::from_translation((1.0, 0.5, -2.0).into());
        let scale = Matrix4::from_nonuniform_scale(1., 2., 3.);
        let transform = translate * rotate * scale * translate.inverse_transform().unwrap();
        assert_abs_diff_eq!(get_axis_scaling(&transform), (1., 2., 3.).into());

        let scale = Matrix4::from_nonuniform_scale(10., -0.5, 1.);
        assert_abs_diff_eq!(get_axis_scaling(&scale), (10., 0.5, 1.).into());
    }
}
