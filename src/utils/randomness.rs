//! Stable, domain-separated fixture and scheduling randomness.
//! Seed derivation is FNV-1a over length-prefixed UTF-8 keys followed by SplitMix64.
//! InputRng is SplitMix64; neither algorithm depends on rand's version or HashMap order.

pub const VALIDATION_SEED: u64 = 0;

pub fn fresh_seed() -> u64 {
    use rand::Rng;
    rand::rng().random()
}

fn mix(mut n: u64) -> u64 {
    n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
    n ^ (n >> 31)
}

pub fn derive_seed(root: u64, keys: &[&str]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for bytes in std::iter::once(root.to_le_bytes().to_vec()).chain(keys.iter().map(|key| {
        let mut bytes = (key.len() as u64).to_le_bytes().to_vec();
        bytes.extend_from_slice(key.as_bytes());
        bytes
    })) {
        for byte in bytes {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    mix(hash)
}

pub struct InputRng {
    state: u64,
}

impl InputRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        mix(self.state)
    }
    pub fn next_f32_range(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0
    }
    pub fn next_u32_range(&mut self, max: u32) -> u32 {
        assert!(max > 0);
        let bound = u64::from(max);
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let value = self.next_u64();
            if value >= threshold {
                return (value % bound) as u32;
            }
        }
    }
    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        for i in (1..values.len()).rev() {
            let bound = (i + 1) as u64;
            let threshold = bound.wrapping_neg() % bound;
            let j = loop {
                let value = self.next_u64();
                if value >= threshold {
                    break (value % bound) as usize;
                }
            };
            values.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streams_are_stable_and_separate() {
        assert_eq!(InputRng::new(0).next_u64(), 0xe220a8397b1dcdaf);
        let seed = derive_seed(42, &["input", "family", "scenario", "64", "0"]);
        assert_ne!(seed, derive_seed(42, &["schedule"]));
        assert_ne!(derive_seed(42, &["ab", "c"]), derive_seed(42, &["a", "bc"]));
        let mut a = InputRng::new(seed);
        let mut b = InputRng::new(seed);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
