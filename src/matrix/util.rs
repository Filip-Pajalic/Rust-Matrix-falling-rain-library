use ::rand::distr::Distribution;
use ::rand::{Rng, rng};
use std::time::Duration;

#[derive(Debug, Clone, Copy, Default)]
pub struct AlphanumericMatrix;

impl Distribution<u8> for AlphanumericMatrix {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u8 {
        const RANGE: u32 = 10 + 9;

        const GEN_ASCII_STR_CHARSET: &[u8] = b"EIOPQRTUWY\
                012345789";
        // We can pick from 62 characters. This is so close to a power of 2, 64,
        // that we can do better than `Uniform`. Use a simple bitshift and
        // rejection sampling. We do not use a bitmask, because for small RNGs
        // the most significant bits are usually of higher quality.
        loop {
            let var = rng.next_u32() >> (32 - 6);
            if var < RANGE {
                return GEN_ASCII_STR_CHARSET[var as usize];
            }
        }
    }
}

pub fn random_duration(min: Duration, max: Duration) -> Duration {
    let mut rng = rng();
    let range = max.as_millis() - min.as_millis();
    let random_millis = rng.random_range(0..=range) as u64;
    min + Duration::from_millis(random_millis)
}
