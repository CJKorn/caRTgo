// PCG32 random number generator, from https://github.com/LiosK/rust-pcg32

use crate::math::Real;

const MUL: u64 = 6364136223846793005;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    pub const fn new(initstate: u64, initseq: u64) -> Self {
        let inc = (initseq << 1) | 1;
        Self {
            state: inc
                .wrapping_add(initstate)
                .wrapping_mul(MUL)
                .wrapping_add(inc),
            inc,
        }
    }

    #[inline]
    pub const fn generate(&mut self) -> u32 {
        let s = self.state;
        self.state = s.wrapping_mul(MUL).wrapping_add(self.inc);
        let xorshifted = (((s >> 18) ^ s) >> 27) as u32;
        xorshifted.rotate_right((s >> 59) as u32)
    }

    // If switching to f64, will need to use 53 bits instead of 24, and divide by 2^53 instead of 2^24
    // Could just do it here but im lazy
    pub fn next_real(&mut self) -> Real {
        (self.generate() >> 8) as Real * (1.0 / (1u32 << 24) as Real)
    }

    pub fn range(&mut self, min: Real, max: Real) -> Real {
        min + (max - min) * self.next_real()
    }
}

impl Default for Pcg32 {
    fn default() -> Self {
        Self {
            state: 0x853c49e6748fea9b,
            inc: 0xda3e39cb94b95bdb,
        }
    }
}
