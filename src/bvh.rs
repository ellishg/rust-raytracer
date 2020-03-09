use super::object::Object;
use super::ray::Ray;
use cgmath::ElementWise;
use cgmath::{Point3, Vector3};

/// Bounding Volume Hierarchy
pub struct Bvh<'a> {
    bvh_tree: BvhTree<'a>,
}

impl<'a> Bvh<'a> {
    pub fn new(
        objects: &'a Vec<Object>,
        min: Point3<f32>,
        max: Point3<f32>,
        leaf_size: usize,
    ) -> Self {
        let objects: Vec<(&'a Object, AABB)> = objects
            .iter()
            .map(|object| {
                let (min, max) = object.get_bounding_box();
                (object, AABB::new(min, max))
            })
            .collect();
        let bvh_tree = BvhTree::new(objects, AABB::new(min, max), leaf_size);
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

    /// Returns the list of objects that can be found inside this bounding box.
    fn filter_objects<'a>(&self, objects: Vec<(&'a Object, AABB)>) -> Vec<(&'a Object, AABB)> {
        objects
            .into_iter()
            .filter(|(_, object_bounding_box)| {
                self.contains_point(object_bounding_box.min)
                    || self.contains_point(object_bounding_box.max)
            })
            .collect()
    }

    /// Returns `true` if `ray` intersects this bounding box.
    fn intersects(&self, ray: &Ray) -> bool {
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

    /// Returns true if this bounding box contains `point`.
    fn contains_point(&self, point: Point3<f32>) -> bool {
        self.min.x <= point.x
            && point.x <= self.max.x
            && self.min.y <= point.y
            && point.y <= self.max.y
            && self.min.z <= point.z
            && point.z <= self.max.z
    }

    /// Return the union of all the bounding boxes.
    fn union(aabbs: Vec<&AABB>) -> Self {
        if aabbs.is_empty() {
            let zero = (0.0, 0.0, 0.0).into();
            AABB::new(zero, zero)
        } else {
            let (mins, maxes): (Vec<Point3<f32>>, Vec<Point3<f32>>) =
                aabbs.into_iter().map(|aabb| (aabb.min, aabb.max)).unzip();
            let cmp = |a: &f32, b: &f32| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal);
            let min_x = mins.iter().map(|p| p.x).min_by(|a, b| cmp(a, b)).unwrap();
            let min_y = mins.iter().map(|p| p.y).min_by(|a, b| cmp(a, b)).unwrap();
            let min_z = mins.iter().map(|p| p.z).min_by(|a, b| cmp(a, b)).unwrap();
            let max_x = maxes.iter().map(|p| p.x).max_by(|a, b| cmp(a, b)).unwrap();
            let max_y = maxes.iter().map(|p| p.y).max_by(|a, b| cmp(a, b)).unwrap();
            let max_z = maxes.iter().map(|p| p.z).max_by(|a, b| cmp(a, b)).unwrap();
            let min = (min_x, min_y, min_z).into();
            let max = (max_x, max_y, max_z).into();
            AABB::new(min, max)
        }
    }
}

enum BvhTree<'a> {
    Node(AABB, Box<BvhTree<'a>>, Box<BvhTree<'a>>),
    Leaf(AABB, Vec<&'a Object>),
    Empty,
}

impl<'a> BvhTree<'a> {
    fn new(objects: Vec<(&'a Object, AABB)>, aabb: AABB, leaf_size: usize) -> Self {
        let objects = aabb.filter_objects(objects);
        if objects.is_empty() {
            BvhTree::Empty
        } else if objects.len() <= leaf_size {
            let objects = objects.into_iter().map(|(object, _)| object).collect();
            BvhTree::Leaf(aabb, objects)
        } else {
            let aabb = AABB::union(objects.iter().map(|(_, aabb)| aabb).collect());

            // Split the bounding box into two parts along its largest dimension.
            // TODO: Find a better way to separate objects into regions.
            let diagonal = aabb.max - aabb.min;
            let offset = if diagonal.x > diagonal.y && diagonal.x > diagonal.z {
                diagonal.mul_element_wise(Vector3::new(0.5, 0.0, 0.0))
            } else if diagonal.y > diagonal.x && diagonal.y > diagonal.z {
                diagonal.mul_element_wise(Vector3::new(0.0, 0.5, 0.0))
            } else {
                diagonal.mul_element_wise(Vector3::new(0.0, 0.0, 0.5))
            };

            let left = AABB {
                min: aabb.min,
                max: aabb.min + offset,
            };
            let right = AABB {
                min: aabb.max - offset,
                max: aabb.max,
            };

            BvhTree::Node(
                aabb,
                Box::new(BvhTree::new(objects.clone(), left, leaf_size)),
                Box::new(BvhTree::new(objects, right, leaf_size)),
            )
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
            BvhTree::Empty => None,
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

        let ray = Ray::new((1.5, 0.0, 0.0).into(), (1.5, 0.0, 1.0).into());
        assert!(!aabb.intersects(&ray));
    }

    #[test]
    fn test_aabb_contains() {
        let min = (-1.0, -1.0, 10.0).into();
        let max = (1.0, 1.0, 11.0).into();
        let aabb0 = AABB::new(min, max);

        let min = (-1.5, -1.5, 9.0).into();
        let max = (1.0, 1.0, 12.0).into();
        let aabb1 = AABB::new(min, max);

        let aabb = AABB::union(vec![&aabb0, &aabb1]);

        assert!(aabb.contains_point((0.0, 0.0, 10.0).into()));
        assert!(aabb.contains_point((1.0, 1.0, 12.0).into()));
        assert!(aabb.contains_point((-1.5, -1.5, 12.0).into()));
        assert!(!aabb.contains_point((1.5, 1.0, 12.0).into()));
    }

    #[test]
    fn test() {
        let min = (-100.0, -100.0, -100.0).into();
        let max = (100.0, 100.0, 100.0).into();
        let leaf_size = 10;
        let m = Material::new(MaterialType::None, TextureType::None);
        let triangle = Object::new_triangle(
            (0.0, 0.0, 1.0).into(),
            (0.0, 1.0, 1.0).into(),
            (1.0, 0.0, 1.0).into(),
            m.clone(),
        );
        let sphere = Object::new_sphere((0.0, 5.0, 1.0).into(), 0.5, m.clone());
        let plane = Object::new_plane((0.0, -5.0, 0.0).into(), (0.0, 1.0, 0.0).into(), m.clone());
        let objects = vec![triangle, sphere, plane];
        let bvh = Bvh::new(&objects, min, max, leaf_size);

        let ray = Ray::new((-1.0, 0.0, 0.0).into(), (-1.0, 0.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_none());

        // Intersect triangle
        let ray = Ray::new((0.1, 0.1, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_some());

        // Intersect sphere
        let ray = Ray::new((0.0, 5.25, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_some());

        // Intersect plane
        let ray = Ray::new((0.0, 0.0, 0.0).into(), (4.0, -1.0, 1.0).into());
        assert!(bvh.get_closest_intersection(&ray).is_some());
    }
}
