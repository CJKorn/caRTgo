// https://github.com/okaneco/rtiow/blob/master/src/aabb.rs

use crate::math::{Real, interval::Interval, ray::Ray, vec3::Vec3};

const MIN_THICKNESS: Real = 1e-4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    axes: [Interval; 3],
}

impl Aabb {
    pub const EMPTY: Self = Self {
        axes: [Interval::EMPTY; 3],
    };

    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        let pad = |i: Interval| {
            if i.size() < MIN_THICKNESS {
                i.expand(MIN_THICKNESS)
            }
            else {
                i
            }
        };
        Self {
            axes: [pad(x), pad(y), pad(z)],
        }
    }

    pub fn from_points(points: &[Vec3]) -> Self {
        let mut axes = [Interval::EMPTY; 3];
        for p in points {
            for (axis, interval) in axes.iter_mut().enumerate() {
                *interval = Interval::enclosing(*interval, Interval::new(p[axis], p[axis]));
            }
        }
        Self::new(axes[0], axes[1], axes[2])
    }

    pub fn union(a: Self, b: Self) -> Self {
        Self {
            axes: [0, 1, 2].map(|axis| Interval::enclosing(a.axes[axis], b.axes[axis])),
        }
    }

    pub fn axis(&self, axis: usize) -> Interval {
        self.axes[axis]
    }

    pub fn centroid(&self) -> Vec3 {
        let mid = |i: Interval| 0.5 * (i.min + i.max);
        Vec3::new(mid(self.axes[0]), mid(self.axes[1]), mid(self.axes[2]))
    }

    pub fn surface_area(&self) -> Real {
        let [x, y, z] = self.axes.map(|i| i.size());
        if x < 0.0 || y < 0.0 || z < 0.0 {
            return 0.0;
        }
        2.0 * (x * y + y * z + z * x)
    }

    pub fn longest_axis(&self) -> usize {
        let [x, y, z] = self.axes.map(|i| i.size());
        if x > y && x > z {
            0
        }
        else if y > z {
            1
        }
        else {
            2
        }
    }

    // Slab test
    pub fn hit(&self, ray: &Ray, mut t_range: Interval) -> bool {
        let origin = ray.origin();
        let inv_dir = ray.inverse_direction();
        for axis in 0..3 {
            let t0 = (self.axes[axis].min - origin[axis]) * inv_dir[axis];
            let t1 = (self.axes[axis].max - origin[axis]) * inv_dir[axis];
            let (near, far) = if t0 < t1 {
                (t0, t1)
            }
            else {
                (t1, t0)
            };

            t_range.min = t_range.min.max(near);
            t_range.max = t_range.max.min(far);
            if t_range.max <= t_range.min {
                return false;
            }
        }
        true
    }
}
