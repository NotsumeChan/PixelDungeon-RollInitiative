//! `game/rng.py` del prototipo: `random.Random` de CPython (MT19937, mismo seeding con entero, `random()` de
//! 53 bits) con los nombres de `com.watabou.utils.Random`. Misma seed = misma secuencia que en Python, así los
//! niveles salen idénticos y se pueden comparar con el prototipo.

const N: usize = 624;
const M: usize = 397;

#[derive(Clone)]
pub struct Rng {
    mt: [u32; N],
    i: usize,
}

impl Rng {
    /// `random.Random(seed)`: init_by_array con los dígitos de 32 bits de |seed| (little endian; 0 -> [0]).
    pub fn new(seed: i64) -> Self {
        let mut n = seed.unsigned_abs();
        let mut key = Vec::new();
        while n > 0 {
            key.push(n as u32);
            n >>= 32;
        }
        if key.is_empty() {
            key.push(0);
        }
        let mut r = Self { mt: [0; N], i: N };
        r.init_by_array(&key);
        r
    }

    fn init_genrand(&mut self, s: u32) {
        self.mt[0] = s;
        for i in 1..N {
            let p = self.mt[i - 1];
            self.mt[i] = 1812433253u32.wrapping_mul(p ^ (p >> 30)).wrapping_add(i as u32);
        }
        self.i = N;
    }

    fn init_by_array(&mut self, key: &[u32]) {
        self.init_genrand(19650218);
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..N.max(key.len()) {
            let p = self.mt[i - 1];
            self.mt[i] = (self.mt[i] ^ (p ^ (p >> 30)).wrapping_mul(1664525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                self.mt[0] = self.mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..N - 1 {
            let p = self.mt[i - 1];
            self.mt[i] = (self.mt[i] ^ (p ^ (p >> 30)).wrapping_mul(1566083941)).wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                self.mt[0] = self.mt[N - 1];
                i = 1;
            }
        }
        self.mt[0] = 0x8000_0000;
    }

    fn next_u32(&mut self) -> u32 {
        if self.i >= N {
            for k in 0..N {
                let y = (self.mt[k] & 0x8000_0000) | (self.mt[(k + 1) % N] & 0x7fff_ffff);
                self.mt[k] = self.mt[(k + M) % N] ^ (y >> 1) ^ if y & 1 != 0 { 0x9908_b0df } else { 0 };
            }
            self.i = 0;
        }
        let mut y = self.mt[self.i];
        self.i += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }

    /// `random()` de CPython: [0, 1) con 53 bits.
    pub fn random(&mut self) -> f64 {
        let a = (self.next_u32() >> 5) as f64;
        let b = (self.next_u32() >> 6) as f64;
        (a * 67108864.0 + b) * (1.0 / 9007199254740992.0)
    }

    /// Float() -> [0, 1).
    pub fn float(&mut self) -> f64 {
        self.random()
    }

    /// Float(max) -> [0, max).
    pub fn float_max(&mut self, a: f64) -> f64 {
        self.random() * a
    }

    /// Float(min, max) -> [min, max).
    pub fn float_range(&mut self, a: f64, b: f64) -> f64 {
        a + self.random() * (b - a)
    }

    /// Int(max) -> [0, max). Con max <= 0 da 0 **sin gastar** rng (como el prototipo).
    pub fn int(&mut self, a: i32) -> i32 {
        if a > 0 { (self.random() * a as f64) as i32 } else { 0 }
    }

    /// Int(min, max) -> [min, max). `as i32` trunca hacia 0, igual que `int()` de Python.
    pub fn int_range(&mut self, a: i32, b: i32) -> i32 {
        a + (self.random() * (b - a) as f64) as i32
    }

    /// IntRange(min, max) -> [min, max] inclusivo.
    pub fn int_range_incl(&mut self, a: i32, b: i32) -> i32 {
        a + (self.random() * (b - a + 1) as f64) as i32
    }

    /// NormalIntRange(min, max): promedio de dos tiradas (campana), inclusivo.
    pub fn normal_int_range(&mut self, a: i32, b: i32) -> i32 {
        let s = self.random() + self.random();
        a + (s * (b - a + 1) as f64 / 2.0) as i32
    }

    /// chances(pesos): índice elegido con probabilidad proporcional al peso.
    pub fn chances(&mut self, weights: &[f64]) -> usize {
        let value = self.float_max(weights.iter().sum());
        let mut acc = 0.0;
        for (i, w) in weights.iter().enumerate() {
            acc += w;
            if value < acc {
                return i;
            }
        }
        0
    }

    /// `_randbelow` de CPython (getrandbits con rechazo): [0, n).
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let k = 32 - n.leading_zeros();
        loop {
            let r = self.next_u32() >> (32 - k);
            if r < n {
                return r;
            }
        }
    }

    /// `random.shuffle` de CPython: misma seed = misma permutación que el prototipo.
    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i as u32 + 1) as usize;
            v.swap(i, j);
        }
    }

    /// element(seq) como índice: None (sin gastar rng) si está vacía.
    pub fn pick(&mut self, len: usize) -> Option<usize> {
        if len == 0 { None } else { Some(self.int(len as i32) as usize) }
    }
}
