pub struct MagicPRNG {
    state: u32,
}

impl MagicPRNG {
    pub fn new() -> Self {
        Self { state: 1804289383 }
    }

    fn rand_32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        self.state
    }

    fn rand_64(&mut self) -> u64 {
        let (n1, n2, n3, n4) = (
            (self.rand_32() & 0xFFFF) as u64,
            (self.rand_32() & 0xFFFF) as u64,
            (self.rand_32() & 0xFFFF) as u64,
            (self.rand_32() & 0xFFFF) as u64,
        );

        n1 | (n2 << 16) | (n3 << 32) | (n4 << 48)
    }

    pub fn rand_magic(&mut self) -> u64 {
        self.rand_64() & self.rand_64() & self.rand_64()
    }
}
