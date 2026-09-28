//https://github.com/okaneco/rtiow/blob/master/src/bvh.rs

use crate::accel::aabb::Aabb;
use crate::geometry::hittable::HitRecord;
use crate::math::{Real, interval::Interval, ray::Ray, vec3::Vec3};

// Binned SAH
const BINS: usize = 12;
const TRAVERSAL_COST: Real = 0.125;
const MAX_LEAF_SIZE: usize = 4;
// Also sizes the traversal stack
const MAX_DEPTH: usize = 60;

#[derive(Debug, Clone, Copy)]
struct Node {
    bounds: Aabb,
    start: u32,
    count: u32,
    axis: u8,
}

#[derive(Debug, Clone, Default)]
pub struct Bvh {
    nodes: Vec<Node>,
    indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct BvhStats {
    pub nodes: usize,
    pub leaves: usize,
    pub max_depth: usize,
    pub max_leaf_size: usize,
}

#[derive(Clone, Copy)]
struct Bin {
    bounds: Aabb,
    count: usize,
}

impl Bvh {
    pub fn build(boxes: &[Aabb]) -> Self {
        let mut bvh = Self {
            nodes: Vec::with_capacity(2 * boxes.len()),
            indices: (0..boxes.len() as u32).collect(),
        };
        if boxes.is_empty() {
            return bvh;
        }

        let centroids: Vec<Vec3> = boxes.iter().map(Aabb::centroid).collect();
        bvh.nodes.push(Node {
            bounds: Aabb::EMPTY,
            start: 0,
            count: 0,
            axis: 0,
        });
        bvh.subdivide(0, 0, boxes.len(), 0, boxes, &centroids);
        bvh
    }

    fn subdivide(
        &mut self,
        node: usize,
        start: usize,
        end: usize,
        depth: usize,
        boxes: &[Aabb],
        centroids: &[Vec3],
    ) {
        let prims = &self.indices[start..end];
        let bounds = prims.iter().fold(Aabb::EMPTY, |b, &i| Aabb::union(b, boxes[i as usize]));
        let count = end - start;

        let leaf = |bvh: &mut Self| {
            bvh.nodes[node] = Node {
                bounds,
                start: start as u32,
                count: count as u32,
                axis: 0,
            };
        };
        if count <= 1 || depth >= MAX_DEPTH {
            return leaf(self);
        }

        let mut centroid_bounds = [Interval::EMPTY; 3];
        for &i in prims {
            let c = centroids[i as usize];
            for (axis, range) in centroid_bounds.iter_mut().enumerate() {
                *range = Interval::enclosing(*range, Interval::new(c[axis], c[axis]));
            }
        }

        let Some((axis, split_bin, split_cost)) = self.best_split(prims, &bounds, &centroid_bounds, boxes, centroids)
        else {
            return leaf(self);
        };
        let leaf_cost = count as Real;
        if count <= MAX_LEAF_SIZE && split_cost >= leaf_cost {
            return leaf(self);
        }

        let axis_range = centroid_bounds[axis];
        let mut mid = start;
        for i in start..end {
            let c = centroids[self.indices[i] as usize][axis];
            if bin_index(c, axis_range) < split_bin {
                self.indices.swap(i, mid);
                mid += 1;
            }
        }
        if mid == start || mid == end {
            mid = start + count / 2;
            self.indices[start..end].select_nth_unstable_by(count / 2, |&a, &b| {
                centroids[a as usize][axis].total_cmp(&centroids[b as usize][axis])
            });
        }

        let left = self.nodes.len();
        let placeholder = Node {
            bounds: Aabb::EMPTY,
            start: 0,
            count: 0,
            axis: 0,
        };
        self.nodes.push(placeholder);
        self.nodes.push(placeholder);
        self.nodes[node] = Node {
            bounds,
            start: left as u32,
            count: 0,
            axis: axis as u8,
        };

        self.subdivide(left, start, mid, depth + 1, boxes, centroids);
        self.subdivide(left + 1, mid, end, depth + 1, boxes, centroids);
    }

    // SAH cost = traversal + (area_left * count_left + area_right * count_right) / area_parent
    fn best_split(
        &self,
        prims: &[u32],
        bounds: &Aabb,
        centroid_bounds: &[Interval; 3],
        boxes: &[Aabb],
        centroids: &[Vec3],
    ) -> Option<(usize, usize, Real)> {
        let parent_area = bounds.surface_area();
        let mut best: Option<(usize, usize, Real)> = None;

        for axis in 0..3 {
            let axis_range = centroid_bounds[axis];
            if axis_range.size() <= 0.0 {
                continue;
            }

            let mut bins = [Bin {
                bounds: Aabb::EMPTY,
                count: 0,
            }; BINS];
            for &i in prims {
                let bin = &mut bins[bin_index(centroids[i as usize][axis], axis_range)];
                bin.bounds = Aabb::union(bin.bounds, boxes[i as usize]);
                bin.count += 1;
            }

            let mut right_cost = [0.0; BINS];
            let mut right = Bin {
                bounds: Aabb::EMPTY,
                count: 0,
            };
            for s in (1..BINS).rev() {
                right.bounds = Aabb::union(right.bounds, bins[s].bounds);
                right.count += bins[s].count;
                right_cost[s] = right.bounds.surface_area() * right.count as Real;
            }

            let mut left = Bin {
                bounds: Aabb::EMPTY,
                count: 0,
            };
            for s in 1..BINS {
                left.bounds = Aabb::union(left.bounds, bins[s - 1].bounds);
                left.count += bins[s - 1].count;
                let left_cost = left.bounds.surface_area() * left.count as Real;
                let cost = TRAVERSAL_COST + (left_cost + right_cost[s]) / parent_area;
                if best.is_none_or(|(_, _, best_cost)| cost < best_cost) {
                    best = Some((axis, s, cost));
                }
            }
        }
        best
    }

    pub fn hit(
        &self,
        ray: &Ray,
        t_range: Interval,
        hit_primitive: impl Fn(usize, Interval) -> Option<HitRecord>,
    ) -> Option<HitRecord> {
        if self.nodes.is_empty() {
            return None;
        }

        let mut range = t_range;
        let mut closest = None;
        let direction = ray.direction();

        let mut stack = [0u32; MAX_DEPTH + 4];
        let mut stack_len = 1;
        while stack_len > 0 {
            stack_len -= 1;
            let node = &self.nodes[stack[stack_len] as usize];
            if !node.bounds.hit(ray, range) {
                continue;
            }

            if node.count > 0 {
                for &i in &self.indices[node.start as usize..(node.start + node.count) as usize] {
                    if let Some(hit) = hit_primitive(i as usize, range) {
                        range.max = hit.t;
                        closest = Some(hit);
                    }
                }
                continue;
            }

            let (near, far) = if direction[node.axis as usize] < 0.0 {
                (node.start + 1, node.start)
            }
            else {
                (node.start, node.start + 1)
            };
            stack[stack_len] = far;
            stack[stack_len + 1] = near;
            stack_len += 2;
        }
        closest
    }

    pub fn stats(&self) -> BvhStats {
        let mut stats = BvhStats {
            nodes: self.nodes.len(),
            leaves: 0,
            max_depth: 0,
            max_leaf_size: 0,
        };
        let mut stack = vec![(0usize, 0usize)];
        while let Some((index, depth)) = stack.pop() {
            let Some(node) = self.nodes.get(index) else {
                break;
            };
            stats.max_depth = stats.max_depth.max(depth);
            if node.count > 0 {
                stats.leaves += 1;
                stats.max_leaf_size = stats.max_leaf_size.max(node.count as usize);
            }
            else {
                stack.push((node.start as usize, depth + 1));
                stack.push((node.start as usize + 1, depth + 1));
            }
        }
        stats
    }
}

fn bin_index(c: Real, axis_range: Interval) -> usize {
    let t = (c - axis_range.min) / axis_range.size();
    ((t * BINS as Real) as usize).min(BINS - 1)
}
