use std::{
    f64::{INFINITY, NEG_INFINITY},
    ops::Range,
};

use crate::{ray::Ray, vec3::Vec3, range::Expandable};

#[derive(Default, Debug)]
pub struct AABB {
    pub x: Range<f64>,
    pub y: Range<f64>,
    pub z: Range<f64>,
}

impl AABB {
    pub fn new() -> Self {
        Self {
            x: 0.0..0.0,
            y: 0.0..0.0,
            z: 0.0..0.0,
        }
    }
    pub fn from_vecs(start: Vec3, end: Vec3) -> Self {
        Self {
            x: start.x.min(end.x)..start.x.max(end.x),
            y: start.y.min(end.y)..start.y.max(end.y),
            z: start.z.min(end.z)..start.z.max(end.z),
        }

    }
    pub fn from_boxes(a: &AABB, b: &AABB) -> Self {
        Self {
            x: a.x.union(&b.x),
            y: a.y.union(&b.y),
            z: a.z.union(&b.z),
        }

    }
    pub fn axis(&self, n: usize) -> &Range<f64> {
        if n == 1 {
            &self.y
        } else if n == 2 {
            &self.z
        } else {
            &self.x
        }
    }
    pub fn hit(&self, ray: &Ray) -> Option<Range<f64>> {
        let mut raymin = NEG_INFINITY;
        let mut raymax = INFINITY;
        for a in 0..3 {
            let inverse_direction = 1. / ray.direction[a];
            let orig = ray.origin[a];
            let ax = self.axis(a);

            let mut t0 = (ax.start - orig) * inverse_direction;
            let mut t1 = (ax.end - orig) * inverse_direction;

            if inverse_direction < 0. {
                std::mem::swap(&mut t0, &mut t1)
            }

            raymin = t0.max(raymin);
            raymax = t1.min(raymax);

            if raymax <= raymin {
                return None;
            }
        }
        return Some(raymin..raymax);
    }
}
