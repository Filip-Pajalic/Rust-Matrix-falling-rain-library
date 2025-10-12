use std::time::Duration;

#[derive(Debug, Clone, Copy, Default)]
pub struct AlphanumericMatrix;

impl AlphanumericMatrix {
    const CHARSET: &'static [u8] = b"EIOPQRTUWY012345789";
    
    pub fn random_char() -> char {
        let index = fastrand::usize(0..Self::CHARSET.len());
        Self::CHARSET[index] as char
    }
}

pub fn random_duration(min: Duration, max: Duration) -> Duration {
    let range = max.as_millis() - min.as_millis();
    let random_millis = fastrand::u128(0..=range) as u64;
    let duration = min + Duration::from_millis(random_millis);

    duration
}