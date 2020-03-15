use super::object::Object;
use super::ray::Ray;
use super::utils::component_wise_range;
use cgmath::{Point3, EuclideanSpace};
use time;

/// Bounding Volume Hierarchy
pub struct Bvh {
    bvh_tree: BvhTree,
}

impl Bvh {
    pub fn new(objects: Vec<Object>, leaf_size: usize) -> Self {
        let instant = time::Instant::now();
        let num_objects = objects.len();
        let bvh_tree = BvhTree::new(objects, leaf_size);
        assert_eq!(bvh_tree.get_num_objects(), num_objects);
        debug!(
            "Generated a bvh tree of {} objects with depth {} and total_sa {} in {} seconds.",
            bvh_tree.get_num_objects(),
            bvh_tree.get_depth(),
            bvh_tree.total_sa(),
            instant.elapsed().as_seconds_f32()
        );
        Bvh { bvh_tree }
    }

    /// If `ray` instersects some object, returns `Some((object, t))` such that the
    /// intersection point is at `ray.get_point_on_ray(t)` on `object`. Otherwise
    /// returns `None`.
    ///
    /// Both `ray` and `t` are in world space coordinates.
    pub fn get_closest_intersection(&self, ray: &Ray) -> Option<(&Object, f32)> {
        self.bvh_tree.get_closest_intersection(ray)
    }
}

/// Axis-aligned Minimum Bounding Box
#[derive(Debug, Copy, Clone)]
struct AABB {
    min: Point3<f32>,
    max: Point3<f32>,
}

impl AABB {
    fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        assert!(min.x <= max.x);
        assert!(min.y <= max.y);
        assert!(min.z <= max.z);
        AABB { min, max }
    }

    /// Returns `Some(t)` if `ray` intersects this bounding box at a point give by
    /// `ray.get_point_on_ray(t)`. Otherwise returns `None`.
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        enum Interval {
            Infinite,
            Closed(f32, f32),
            Empty,
        };

        impl Interval {
            /// Construct the interval that a ray intersects some axis on an AABB.
            /// `(a, b)` are the bounds of this axis, `x` is the start of the ray,
            /// and `slope` is the direction of this ray.
            fn new(a: f32, b: f32, x: f32, slope: f32) -> Interval {
                if slope == 0.0 {
                    // The ray is parallel to this axis.
                    if a <= x && x <= b {
                        // The ray is inside the box for this axis.
                        Interval::Infinite
                    } else {
                        // The ray is outide the box for this axis.
                        Interval::Empty
                    }
                } else {
                    let a = (a - x) / slope;
                    let b = (b - x) / slope;
                    Interval::Closed(f32::min(a, b), f32::max(a, b))
                }
            }

            /// Return the intersection of the two intervals.
            fn intersect(self, other: Interval) -> Interval {
                match self {
                    Interval::Infinite => other,
                    Interval::Empty => Interval::Empty,
                    Interval::Closed(a, b) => {
                        match other {
                            Interval::Infinite => Interval::Closed(a, b),
                            Interval::Empty => Interval::Empty,
                            Interval::Closed(c, d) => {
                                // Construct a new interval from the greatest lower bound and the least upper bound.
                                let x = f32::max(a, c);
                                let y = f32::min(b, d);
                                if x <= y {
                                    Interval::Closed(x, y)
                                } else {
                                    // The intervals do not overlap, return the empty interval.
                                    Interval::Empty
                                }
                            }
                        }
                    }
                }
            }
        }

        let position: Point3<f32> = ray.get_point_on_ray(0.0).into();
        let direction = ray.get_direction();
        let x_interval = Interval::new(self.min.x, self.max.x, position.x, direction.x);
        let y_interval = Interval::new(self.min.y, self.max.y, position.y, direction.y);
        let z_interval = Interval::new(self.min.z, self.max.z, position.z, direction.z);
        let t_interval = x_interval.intersect(y_interval.intersect(z_interval));
        match t_interval {
            Interval::Infinite => unreachable!(),
            Interval::Closed(t_min, t_max) => {
                if t_min < 0. {
                    Some(t_max)
                } else {
                    Some(t_min)
                }
            }
            Interval::Empty => None,
        }
    }

    /// Return the union of all the bounding boxes.
    fn union(aabbs: Vec<AABB>) -> Self {
        if aabbs.is_empty() {
            let zero = (0.0, 0.0, 0.0).into();
            AABB::new(zero, zero)
        } else {
            let points = aabbs
                .iter()
                .flat_map(|aabb| vec![aabb.min, aabb.max])
                .collect();
            let (min, max) = component_wise_range(points);
            AABB::new(min, max)
        }
    }

    fn surface_area(&self) -> f32 {
        let diff = self.max - self.min;
        2. * diff.x * diff.y +
        2. * diff.x * diff.z +
        2. * diff.z * diff.y
    }
}

/// Splits objects into two halves after sorting by min x coordinate
fn bvh_split_by_x_axis(mut objects: Vec<Object>) -> (Vec<Object>, Vec<Object>) {
    objects.sort_by(|a, b| {
        let (amin, _amax) = a.get_bounding_box();
        let (bmin, _bmax) = b.get_bounding_box();
        amin.x.partial_cmp(&bmin.x).unwrap()
    });
    let mid = objects.len() / 2;
    let mut left = objects;
    let right = left.split_off(mid);
    (left, right)
}

/// Splits objects into two halves by the midpoint along the dimension with
/// largest range in object centroid positions.
/// As outlined in section 4.4.1 of the PBRT book.
fn bvh_split_by_widest_dim(mut objects: Vec<Object>) -> (Vec<Object>, Vec<Object>) {
    let centroids = objects.iter().map(|obj| {
        let (min, max) = obj.get_bounding_box();
        let c = Point3::centroid(&[min, max]);
        c
    }).collect();
    let (min_c, max_c) = component_wise_range(centroids);
    println!("min_c {:?} max_c {:?}", min_c, max_c);

    let diff = max_c - min_c;
    let mut maxdim = 0;
    let mut max = diff.x;
    if diff.y > max {
        max = diff.y;
        maxdim = 1;
    }
    if diff.z > max {
        max = diff.z;
        maxdim = 2;
    }
    let max_axis_midpoint: f32 = (max_c[maxdim] - min_c[maxdim]) / 2.;

    println!("max {} maxdim {} midpoint {}", max, maxdim, max_axis_midpoint);
    let (left, right) = objects.drain(..).partition(|obj| {
        let (min, max) = obj.get_bounding_box();
        let c = Point3::centroid(&[min, max]);
        c[maxdim] < max_axis_midpoint
    });

    (left, right)
}


/// Splits objects into two halves in order to minimize the expected cost
/// of a ray intersection query, using the Surface Area Heuristic (SAH).
/// See section 4.4.2 of the PBRT book.
// fn bvh_split_by_sah(mut objects: Vec<Object>) -> (Vec<Object>, Vec<Object>) {
// }

enum BvhTree {
    Node(AABB, Box<BvhTree>, Box<BvhTree>, usize),
    Leaf(AABB, Vec<Object>, usize),
}

impl BvhTree {
    fn new(objects: Vec<Object>, leaf_size: usize) -> Self {
        let size = objects.len();
        if size <= leaf_size {
            let aabbs = objects
                .iter()
                .map(|object| {
                    let (min, max) = object.get_bounding_box();
                    AABB::new(min, max)
                })
                .collect();
            let aabb = AABB::union(aabbs);
            BvhTree::Leaf(aabb, objects, size)
        } else {
            // let (left_objects, right_objects) = bvh_split_by_x_axis(objects);
            let (left_objects, right_objects) = bvh_split_by_widest_dim(objects);
            let size = left_objects.len() + right_objects.len();
            let left = BvhTree::new(left_objects, leaf_size);
            let right = BvhTree::new(right_objects, leaf_size);

            let aabb = AABB::union(vec![left.get_aabb(), right.get_aabb()]);

            BvhTree::Node(aabb, Box::new(left), Box::new(right), size)
        }
    }

    fn get_closest_intersection(&self, ray: &Ray) -> Option<(&Object, f32)> {
        match self {
            BvhTree::Node(aabb, left, right, _size) => {
                if let Some(_) = aabb.intersect(ray) {
                    [left, right]
                        .iter()
                        .filter_map(|bvh| bvh.get_closest_intersection(ray))
                        // Just a hacky way to find the smallest t value.
                        .min_by(|(_, t_left), (_, t_right)| {
                            t_left
                                .partial_cmp(t_right)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                } else {
                    None
                }
            }
            BvhTree::Leaf(aabb, objects, _size) => {
                if let Some(_) = aabb.intersect(ray) {
                    objects
                        .iter()
                        .filter_map(|object| match object.get_intersection(ray) {
                            Some(t) => Some((object, t)),
                            None => None,
                        })
                        // Just a hacky way to find the smallest t value.
                        .min_by(|(_, t_left), (_, t_right)| {
                            t_left
                                .partial_cmp(t_right)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                } else {
                    None
                }
            }
        }
    }

    fn get_aabb(&self) -> AABB {
        match self {
            BvhTree::Node(aabb, _, _, _) => *aabb,
            BvhTree::Leaf(aabb, _, _) => *aabb,
        }
    }

    fn get_depth(&self) -> usize {
        match self {
            BvhTree::Node(_, left, right, _) => 1 + left.get_depth().max(right.get_depth()),
            BvhTree::Leaf(_, _, _) => 0,
        }
    }

    fn get_num_objects(&self) -> usize {
        match self {
            BvhTree::Node(_, _, _, size) => *size,
            BvhTree::Leaf(_, _, size) => *size,
        }
    }

    /// Total surface area of this bvh (recursively computed)
    fn total_sa(&self) -> f32 {
        match self {
            BvhTree::Leaf(aabb, _objs, _size) => {
                aabb.surface_area()
            }
            BvhTree::Node(aabb, left, right, _size) => {
                aabb.surface_area() + left.total_sa() + right.total_sa()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bvh, AABB, bvh_split_by_widest_dim};
    use crate::material::{Material, MaterialType, TextureType};
    use crate::object::Object;
    use crate::ray::Ray;
    use cgmath::Point3;

    #[test]
    fn test_aabb_surface_area() {
        let min = (0., 0., 0.).into();
        let max = (1., 1., 2.).into();
        let aabb = AABB::new(min, max);
        assert_eq!(aabb.surface_area(), 10.);
    }

    #[test]
    fn test_aabb_intersect() {
        let min = (-1.0, -1.0, 10.0).into();
        let max = (1.0, 1.0, 11.0).into();
        let aabb = AABB::new(min, max);

        let ray = Ray::new((0.0, 0.0, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(aabb.intersect(&ray).is_some());

        let ray = Ray::new((0.5, 0.0, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(aabb.intersect(&ray).is_some());

        let ray = Ray::new((-10.1, -10.0, 0.0).into(), (10.1, 10.2, 10.3).into());
        assert!(aabb.intersect(&ray).is_some());

        let ray = Ray::new((0.0, 0.0, 10.5).into(), (0.0, 0.0, 1.0).into());
        assert!(aabb.intersect(&ray).is_some());

        let ray = Ray::new((0.0, 0.0, 10.5).into(), (1.0, 1.0, 1.0).into());
        assert!(aabb.intersect(&ray).is_some());

        let ray = Ray::new((1.5, 0.0, 0.0).into(), (1.5, 0.0, 1.0).into());
        assert!(aabb.intersect(&ray).is_none());

        let min = (0., 0., 0.).into();
        let max = (1.0, 1.0, 1.0).into();
        let aabb = AABB::new(min, max);

        let ray = Ray::new((-0.5, -0.5, -0.5).into(), (0.5, 0.5, 0.5).into());
        assert!(aabb.intersect(&ray).is_some());

        let ray = Ray::new((-0.5, -0.5, -0.5).into(), (-0.5, 0.5, 0.5).into());
        assert!(aabb.intersect(&ray).is_none());

        let ray = Ray::new((-1.0, 0.5, 0.5).into(), (1.0, 0., 0.).into());
        assert_eq!(aabb.intersect(&ray), Some(1.));

        let ray = Ray::new((-0.5, -0.5, 0.5).into(), (0.5, 0.5, 0.).into());
        assert_eq!(aabb.intersect(&ray), Some(1. / (2. as f32).sqrt()));

        // ray grazes a corner
        let ray = Ray::new((-1.0, -1.0, 0.).into(), (1., 0.5, 0.).into());
        assert_eq!(aabb.intersect(&ray), Some(ray.get_t((1., 0., 0.).into())));

        // ray starts in the middle and shoots out
        let ray = Ray::new((0.5, 0.5, 0.5).into(), (1., 0., 0.).into());
        assert_eq!(aabb.intersect(&ray), Some(0.5));
    }

    #[test]
    fn test_bvh_intersect() {
        let leaf_size = 1;
        let m = Material::new(MaterialType::None, TextureType::None);
        let triangle = Object::new_triangle(
            (0.0, 0.0, 1.0).into(),
            (0.0, 1.0, 1.0).into(),
            (1.0, 0.0, 1.0).into(),
            m.clone(),
        );
        let sphere = Object::new_sphere((0.0, 5.0, 1.0).into(), 0.5, m.clone());
        let quad = Object::new_quad(
            (0.0, -5.0, 0.0).into(),
            (1.0, -5.0, 0.0).into(),
            (1.0, -5.0, -1.0).into(),
            (0.0, -5.0, -1.0).into(),
            m.clone(),
        );
        let objects = vec![triangle, sphere, quad];
        println!("0");
        let bvh = Bvh::new(objects, leaf_size);
        println!("1");

        let ray = Ray::new((-1.0, 0.0, 0.0).into(), (-1.0, 0.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_none());

        // Intersect triangle
        let ray = Ray::new((0.1, 0.1, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_some());

        let ray = Ray::new((-0.1, 0.1, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_none());

        // Intersect sphere
        let ray = Ray::new((0.0, 5.25, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_some());

        let ray = Ray::new((0.0, 5.55, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_none());

        // Intersect quad
        let ray = Ray::new((0.0, 0.0, 0.0).into(), (0.0, -1.0, 0.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_some());

        let ray = Ray::new((0.0, 0.0, 0.0).into(), (0.1, -5.0, -0.1).into());
        assert!(bvh.get_closest_intersection(&ray).is_some());

        let ray = Ray::new((2.0, 0.0, 0.0).into(), (0.0, -1.0, 0.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_none());
    }

    #[test]
    fn test_bvh_split_by_widest_dim() {
        let mock_sphere = |center: Point3<f32>| {
            Object::new_sphere(center, 1.0, Material::new(MaterialType::None, TextureType::None))
        };

        let objects = vec![
            mock_sphere((0., 0., 0.).into()),
            mock_sphere((2., 1., 1.).into()),
            mock_sphere((1.5, 1., 1.).into()),
        ];

        let (left, right) = bvh_split_by_widest_dim(objects);

        // let left: Vec<(Point3<f32>, Point3<f32>)> = left.iter().map(|obj| { obj.get_bounding_box() }).collect();
        // println!("{:?}", left);
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 2);
    }
}
