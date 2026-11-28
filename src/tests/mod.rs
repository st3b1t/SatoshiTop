//! Simulated data sources for developing without a Bitcoin node (`FAKERPC`)
//! or on a machine without real metrics (`FAKESYS`, `FAKEMODE` in the original), ported from
//! `test/`.

pub mod rpc_fake;
pub mod sys_fake;

use std::time::{SystemTime, UNIX_EPOCH};

/// Seed for the simulations: `FAKESEED` if it is a valid number (for
/// reproducible sessions), otherwise the clock.
pub fn seed_from_env() -> u64 {
    std::env::var("FAKESEED")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0)
        })
}

/// Small deterministic PRNG (SplitMix64): good enough for fake data, no
/// external crate.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`, like `Math.random()`.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `[0, n)`; `n` must be greater than 0.
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    /// Shuffles `items` in place (Fisher-Yates).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod rng_tests {
    use super::*;

    #[test]
    fn rng_is_deterministic_and_in_range() {
        let (mut a, mut b) = (Rng::new(42), Rng::new(42));
        for _ in 0..1000 {
            let x = a.next_f64();
            assert_eq!(x, b.next_f64());
            assert!((0.0..1.0).contains(&x));
            assert!(a.below(7) < 7);
            b.below(7);
        }
    }

    #[test]
    fn shuffle_keeps_the_elements() {
        let mut items: Vec<u32> = (0..20).collect();
        Rng::new(1).shuffle(&mut items);
        assert_ne!(items, (0..20).collect::<Vec<_>>());
        items.sort();
        assert_eq!(items, (0..20).collect::<Vec<_>>());
    }
}
