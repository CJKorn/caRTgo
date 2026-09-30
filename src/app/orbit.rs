use cartgo::math::{Real, vec3::Vec3};
use cartgo::render::camera::Camera;

const MAX_PITCH: Real = 89.0 * std::f32::consts::PI / 180.0;

pub struct Orbit {
    pub target: Vec3,
    pub yaw: Real,
    pub pitch: Real,
    pub distance: Real,
}

impl Orbit {
    pub fn from_position(position: Vec3, target: Vec3) -> Self {
        let offset = position - target;
        let distance = offset.length();
        Self {
            target,
            yaw: offset.x().atan2(-offset.y()),
            pitch: (offset.z() / distance).asin(),
            distance,
        }
    }

    pub fn rotate(&mut self, d_yaw: Real, d_pitch: Real) {
        self.yaw += d_yaw;
        self.pitch = (self.pitch + d_pitch).clamp(-MAX_PITCH, MAX_PITCH);
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin(), self.yaw.cos(), 0.0)
    }

    pub fn right(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), self.yaw.sin(), 0.0)
    }

    pub fn up(&self) -> Vec3 {
        self.offset().cross(self.right())
    }

    pub fn pan(&mut self, dx: Real, dy: Real) {
        self.target += dy * self.up() - dx * self.right();
    }

    pub fn apply(&self, camera: &mut Camera) {
        camera.set_position(self.target + self.distance * self.offset());
        camera.look_at(self.target);
    }

    fn offset(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        Vec3::new(cos_pitch * self.yaw.sin(), -cos_pitch * self.yaw.cos(), sin_pitch)
    }
}
