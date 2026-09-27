use crate::math::Real;

#[derive(Debug, Clone, Copy)]
pub struct ImageSpec {
    width: u32,
    height: u32,
    inv_width: Real,
    inv_height: Real,
}

impl ImageSpec {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            inv_width: 1.0 / width as Real,
            inv_height: 1.0 / height as Real,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn inv_width(&self) -> Real {
        self.inv_width
    }

    pub fn inv_height(&self) -> Real {
        self.inv_height
    }

    pub fn aspect(&self) -> Real {
        self.width as Real * self.inv_height
    }

    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }
}
