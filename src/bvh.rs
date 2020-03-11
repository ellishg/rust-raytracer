use super::object::Object;
use super::ray::Ray;
use super::utils::component_wise_range;
use cgmath::Point3;

/// Bounding Volume Hierarchy
pub struct Bvh<'a> {
    bvh_tree: BvhTree<'a>,
}

impl<'a> Bvh<'a> {
    pub fn new(objects: Vec<&'a Object>, leaf_size: usize) -> Self {
        let num_objects = objects.len();
        let bvh_tree = BvhTree::new(objects, leaf_size);
        assert_eq!(bvh_tree.get_num_objects(), num_objects);
        debug!(
            "Generated a bvh tree of {} objects with depth {}",
            bvh_tree.get_num_objects(),
            bvh_tree.get_depth()
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

    /// Returns `true` if `ray` intersects this bounding box.
    fn intersects(&self, ray: &Ray) -> bool {
        // FIXME: This function needs to be fixed.
        // Was following
        // https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-box-intersection
        // but it looks like I have a bug.
        // TODO: Return Some(t) instead of true.
        let aux = |min: f32, max: f32, x: f32, slope: f32| {
            if slope == 0.0 {
                (std::f32::NEG_INFINITY, std::f32::INFINITY)
            } else {
                let a = (min - x) / slope;
                let b = (max - x) / slope;
                if a < b {
                    return (a, b);
                } else {
                    (b, a)
                }
            }
        };
        let position: Point3<f32> = ray.get_point_on_ray(0.0).into();
        let direction = ray.get_direction();
        let range = aux(self.min.x, self.max.x, position.x, direction.x);
        let range_y = aux(self.min.y, self.max.y, position.y, direction.y);
        if range.0 > range_y.1 || range_y.0 > range.1 {
            false
        } else {
            let range = (range.0.max(range_y.0), range.1.min(range_y.1));
            let range_z = aux(self.min.z, self.max.z, position.z, direction.z);
            if range.0 > range_z.1 || range_z.0 > range.1 {
                false
            } else {
                true
            }
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
}

enum BvhTree<'a> {
    Node(AABB, Box<BvhTree<'a>>, Box<BvhTree<'a>>),
    Leaf(AABB, Vec<&'a Object>),
}

impl<'a> BvhTree<'a> {
    fn new(objects: Vec<&'a Object>, leaf_size: usize) -> Self {
        if objects.len() <= leaf_size {
            let aabbs = objects
                .iter()
                .map(|object| {
                    let (min, max) = object.get_bounding_box();
                    AABB::new(min, max)
                })
                .collect();
            let aabb = AABB::union(aabbs);
            BvhTree::Leaf(aabb, objects)
        } else {
            // TODO: Partition objects in a smarter way.
            let mid = objects.len() / 2;
            let (left_objects, right_objects) = objects.split_at(mid);

            let left = BvhTree::new(left_objects.to_vec(), leaf_size);
            let right = BvhTree::new(right_objects.to_vec(), leaf_size);

            let aabb = AABB::union(vec![left.get_aabb(), right.get_aabb()]);

            BvhTree::Node(aabb, Box::new(left), Box::new(right))
        }
    }

    fn get_closest_intersection(&self, ray: &Ray) -> Option<(&Object, f32)> {
        match self {
            BvhTree::Node(aabb, left, right) => {
                if aabb.intersects(ray) {
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
            BvhTree::Leaf(aabb, objects) => {
                if aabb.intersects(ray) {
                    objects
                        .iter()
                        .filter_map(|object| match object.get_intersection(ray) {
                            Some(t) => Some((*object, t)),
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
            BvhTree::Node(aabb, _, _) => *aabb,
            BvhTree::Leaf(aabb, _) => *aabb,
        }
    }

    fn get_depth(&self) -> usize {
        match self {
            BvhTree::Node(_, left, right) => 1 + left.get_depth().max(right.get_depth()),
            BvhTree::Leaf(_, _) => 0,
        }
    }

    fn get_num_objects(&self) -> usize {
        match self {
            BvhTree::Node(_, left, right) => left.get_num_objects() + right.get_num_objects(),
            BvhTree::Leaf(_, objects) => objects.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bvh, AABB};
    use crate::material::{Material, MaterialType, TextureType};
    use crate::object::Object;
    use crate::ray::Ray;

    #[test]
    fn test_aabb_intersect() {
        let min = (-1.0, -1.0, 10.0).into();
        let max = (1.0, 1.0, 11.0).into();
        let aabb = AABB::new(min, max);

        let ray = Ray::new((0.0, 0.0, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(aabb.intersects(&ray));

        let ray = Ray::new((0.5, 0.0, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(aabb.intersects(&ray));

        let ray = Ray::new((0.0, 0.0, 10.5).into(), (0.0, 0.0, 1.0).into());
        assert!(aabb.intersects(&ray));

        let ray = Ray::new((1.5, 0.0, 0.0).into(), (1.5, 0.0, 1.0).into());
        assert!(!aabb.intersects(&ray));
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
        let objects = vec![&triangle, &sphere, &quad];
        let bvh = Bvh::new(objects, leaf_size);

        let ray = Ray::new((-1.0, 0.0, 0.0).into(), (-1.0, 0.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_none());

        // Intersect triangle
        let ray = Ray::new((0.1, 0.1, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_some());

        // Intersect sphere
        let ray = Ray::new((0.0, 5.25, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_some());

        // Intersect quad
        let ray = Ray::new((0.0, 0.0, 0.0).into(), (0.0, -1.0, 0.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_some());

        let ray = Ray::new((0.0, 0.0, 0.0).into(), (0.1, -5.0, -0.1).into());
        assert!(bvh.get_closest_intersection(&ray).is_some());
    }
}
