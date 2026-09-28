use crate::math::{Real, vec3::Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Onb {
    u: Vec3,
    v: Vec3,
    w: Vec3,
}

impl Onb {
    // Duff et al. 2017, "Building an Orthonormal Basis, Revisited"
    pub fn from_w(w: Vec3) -> Self {
        let sign = (1.0 as Real).copysign(w.z());
        let a = -1.0 / (sign + w.z());
        let b = w.x() * w.y() * a;
        Self {
            u: Vec3::new(1.0 + sign * w.x() * w.x() * a, sign * b, -sign * w.x()),
            v: Vec3::new(b, sign + w.y() * w.y() * a, -w.y()),
            w,
        }
    }

    pub fn to_world(&self, local: Vec3) -> Vec3 {
        local.x() * self.u + local.y() * self.v + local.z() * self.w
    }
}
