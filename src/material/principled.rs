use crate::color::Color;
use crate::geometry::hittable::HitRecord;
use crate::math::{Real, ray::Ray, sampling::uniform_sphere, vec3::Vec3};
use crate::rng::Pcg32;

// Instead of doing the RTIOW way of having a separate material for each type of material
// I thought it was smarter to emulate the Blender Principled BSDF, which is a single shader with a lot of parameters.
#[derive(Debug, Clone, Copy)]
pub struct Principled {
    pub base_color: Color,
    pub metallic: Real,
    pub roughness: Real,
    pub ior: Real,
    pub transmission: Real,
    pub emission: Color,
}

impl Default for Principled {
    // Blender default
    fn default() -> Self {
        Self {
            base_color: Color::new(0.8, 0.8, 0.8),
            metallic: 0.0,
            roughness: 0.5,
            ior: 1.5,
            transmission: 0.0,
            emission: Color::BLACK,
        }
    }
}

pub struct Scatter {
    pub ray: Ray,
    pub attenuation: Color,
}

impl Principled {
    pub fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut Pcg32) -> Option<Scatter> {
        let dir = ray.direction().normalize();
        let fuzz = self.roughness * self.roughness;

        if rng.next_real() < self.metallic {
            let reflected = fuzzy(reflect(dir, hit.normal), fuzz, rng);
            return self.bounce(ray, hit, reflected, self.base_color);
        }

        if rng.next_real() < self.transmission {
            let eta = if hit.front_face {
                1.0 / self.ior
            }
            else {
                self.ior
            };
            let cos = (-dir).dot(hit.normal).min(1.0);
            let sin = (1.0 - cos * cos).sqrt();
            let out = if eta * sin > 1.0 || schlick(cos, eta) > rng.next_real() {
                reflect(dir, hit.normal)
            }
            else {
                refract(dir, hit.normal, eta, cos)
            };
            return Some(Scatter {
                ray: Ray::new(hit.point, fuzzy(out, fuzz, rng), ray.time()),
                attenuation: self.base_color,
            });
        }

        let cos = (-dir).dot(hit.normal).min(1.0);
        if schlick(cos, self.ior) > rng.next_real() {
            let reflected = fuzzy(reflect(dir, hit.normal), fuzz, rng);
            return self.bounce(ray, hit, reflected, Color::WHITE);
        }

        let mut diffuse = hit.normal + uniform_sphere(rng.next_real(), rng.next_real());
        if diffuse.length_squared() < 1e-8 {
            diffuse = hit.normal;
        }
        self.bounce(ray, hit, diffuse, self.base_color)
    }

    fn bounce(&self, ray: &Ray, hit: &HitRecord, dir: Vec3, attenuation: Color) -> Option<Scatter> {
        (dir.dot(hit.normal) > 0.0).then(|| Scatter {
            ray: Ray::new(hit.point, dir, ray.time()),
            attenuation,
        })
    }
}

fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

fn refract(v: Vec3, n: Vec3, eta: Real, cos: Real) -> Vec3 {
    let perp = eta * (v + cos * n);
    let parallel = -(1.0 - perp.length_squared()).abs().sqrt() * n;
    perp + parallel
}

fn schlick(cos: Real, eta: Real) -> Real {
    let r0 = ((1.0 - eta) / (1.0 + eta)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cos).powi(5)
}

fn fuzzy(dir: Vec3, fuzz: Real, rng: &mut Pcg32) -> Vec3 {
    if fuzz <= 0.0 {
        return dir;
    }
    dir.normalize() + fuzz * uniform_sphere(rng.next_real(), rng.next_real())
}
