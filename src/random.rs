use std::ops::BitXor;

pub struct Rng {
    state: [u64; 2],
}

impl Rng {
    pub fn from_seed(seed: [u64; 2]) -> Self {
        Self { state: seed }
    }
    pub fn next_u64(&mut self) -> u64 {
        let a = self.state[0];
        let b = self.state[1];
        let result = (a.wrapping_add(b));

        let c = b.bitxor(a);
        self.state[0] = a.rotate_left(24).bitxor(c).bitxor(c << 16);
        self.state[1] = c.rotate_left(37);

        return result;
    }
    pub fn next_f64(&mut self) -> f64 {
        self.next_u64() as f64 / u64::MAX as f64
    }
    fn jump_impl(&mut self, jumper: [u64; 2]) -> &mut Self {
        let mut s0 = 0;
        let mut s1 = 0;

        for j in &jumper {
            for b in 0..64 {
                if (j & 1 << b) != 0 {
                    s0 ^= self.state[0];
                    s1 ^= self.state[1];
                }
                self.next_u64();
            }
        }
        self.state[0] = s0;
        self.state[1] = s1;
        return self;
    }
    pub fn short_jump(&mut self) -> &mut Self {
        const JUMPER: [u64; 2] = [0xdf900294d8f554a5, 0x170865df4b3201fc];
        return self.jump_impl(JUMPER);
    }
    pub fn long_jump(&mut self) -> &mut Self {
        const JUMPER: [u64; 2] = [0xd2a98b26625eee7b, 0xdddf9b1090aa7ac1];
        return self.jump_impl(JUMPER);
    }
}
