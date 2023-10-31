use super::aabb::AABB;
use super::HitRecord;
use std::cmp::Ordering;
use std::f64::INFINITY;
use std::f64::NEG_INFINITY;
use std::ops::Range;

use crate::range::RangeExtensions;
use crate::ray::Ray;
use super::Hittable;

#[derive(Default, Debug)]
pub struct HittableList {
    pub(crate) objects: Vec<Box<dyn Hittable>>,
    pub(crate) bounding_box: AABB,
}

impl HittableList {
    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.bounding_box = AABB::from_boxes(&self.bounding_box, object.bounding_box());
        self.objects.push(object);
    }
    pub fn into_bvh(mut self) -> Box<dyn Hittable> {
        return BVHNode::from_vec(&mut self.objects);
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, ray_trange: &Range<f64>) -> Option<HitRecord> {
        let mut closest_so_far = ray_trange.end;
        let mut result = None;

        for object in self.objects.iter() {
            if let Some(hit_record) = object.hit(ray, &(ray_trange.start..closest_so_far)) {
                closest_so_far = hit_record.t;
                result = Some(hit_record);
            }
        }
        return result;
    }

    fn bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}

#[derive(Debug)]
pub struct BVHNode {
    pub(crate) left: Box<dyn Hittable>,
    pub(crate) right: Box<dyn Hittable>,
    pub(crate) bounding_box: AABB,
}

impl BVHNode {
    pub(crate) fn from_vec(mut objects: &mut Vec<Box<dyn Hittable>>) -> Box<dyn Hittable> {
        return BVHNode::inner_from_vec(objects, 0, 0);
    }
    pub(crate) fn inner_from_vec(
        mut objects: &mut Vec<Box<dyn Hittable>>,
        start: usize,
        depth: usize,
    ) -> Box<dyn Hittable> {
        let length = objects.len() - start;
        let axis = depth % 3;

        return if length == 1 {
            //println!("{}node", " ".repeat(depth));
            objects.pop().unwrap()
        } else if length == 2 {
            let left = objects.pop().unwrap();
            let right = objects.pop().unwrap();
            let bounding_box = AABB::from_boxes(left.bounding_box(), right.bounding_box());
            //println!("{}node", " ".repeat(depth + 1));
            //println!("{}node", " ".repeat(depth + 1));
            Box::new(BVHNode {
                left,
                right,
                bounding_box,
            })
        } else {
            // let axis = {
            //     let mut result = 0;
            //     let mut max_variance = NEG_INFINITY;
            //     for i in 0..3 {
            //         let variance = (objects
            //             .split_at(start)
            //             .1
            //             .iter()
            //             .map(|x| x.bounding_box().axis(i).middle().powi(2))
            //             .sum::<f64>()
            //             - (objects
            //                 .split_at(start)
            //                 .1
            //                 .iter()
            //                 .map(|x| x.bounding_box().axis(i).middle())
            //                 .sum::<f64>()
            //                 .powi(2)
            //                 / length as f64))
            //             / length as f64;
            //         if variance > max_variance {
            //             result = i;
            //             max_variance = variance;
            //         }
            //     }
            //     result
            // };
            let (axis, _) = (0..3).fold((0, NEG_INFINITY), |(prev_axid, highest_diff), axid| {
                let (min, max) = objects
                    .split_at(start)
                    .1
                    .iter()
                    .map(|o| o.bounding_box().axis(axid).middle())
                    .fold((INFINITY, NEG_INFINITY), |(min, max), next| {
                        (min.min(next), max.max(next))
                    });
                ((max - min) > highest_diff)
                    .then(|| (axid, max - min))
                    .unwrap_or_else(|| (prev_axid, highest_diff))
            });
            let comparator = |a: &_, b: &_| BVHNode::box_compare(a, b, axis);
            let mean = objects
                .split_at(start)
                .1
                .iter()
                .map(|o| o.bounding_box().axis(axis).middle())
                .sum::<f64>()
                / length as f64;

            // sort the end of the vec from `start` to the end
            objects.split_at_mut(start).1.sort_by(comparator);

            let split = objects
                .split_at(start)
                .1
                .iter()
                .map(|o| o.bounding_box().axis(axis).middle())
                .position(|x| x >= mean)
                .unwrap_or(length / 2)
                .max(1);

            // take the part after the split and recurse. All elements in the part will be popped.
            let right = BVHNode::inner_from_vec(objects, start + split, depth + 1);
            // take the whole part which only includes the part before the split as the rest was popped.
            let left = BVHNode::inner_from_vec(objects, start, depth + 1);
            let bounding_box = AABB::from_boxes(left.bounding_box(), right.bounding_box());
            Box::new(BVHNode {
                left,
                right,
                bounding_box,
            })
        };
    }

    pub(crate) fn box_compare(
        a: &Box<dyn Hittable>,
        b: &Box<dyn Hittable>,
        axis_index: usize,
    ) -> Ordering {
        let a_bound = a.bounding_box().axis(axis_index);
        let b_bound = b.bounding_box().axis(axis_index);
        //a_bound.end.total_cmp(&b_bound.end)
        a_bound.middle().total_cmp(&b_bound.middle())
        // a_bound.start.total_cmp(&b_bound.start)
    }
    pub(crate) fn box_x_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 0)
    }
    pub(crate) fn box_y_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 1)
    }
    pub(crate) fn box_z_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 2)
    }
}

impl Hittable for BVHNode {
    fn hit(&self, ray: &Ray, ray_trange: &Range<f64>) -> Option<HitRecord> {
        if self.bounding_box.hit(ray).is_none() {
            return None;
        }

        if let Some(record) = self.left.hit(ray, ray_trange) {
            if let Some(record) = self.right.hit(ray, &(ray_trange.start..record.t)) {
                Some(record)
            } else {
                Some(record)
            }
        } else {
            self.right.hit(ray, ray_trange)
        }
    }

    fn bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}
