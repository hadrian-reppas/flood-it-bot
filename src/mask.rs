use core::arch::aarch64::{
    uint8x16_t, uint16x8_t, vaddlvq_u16, vaddq_u16, vaddvq_u32, vandq_u16, vbicq_u16, vcntq_u8,
    vcombine_u8, vdotq_u32, vdupq_n_u16, vdupq_n_u32, veorq_u16, vextq_u16, vmaxvq_u16, vminvq_u16,
    vmovn_u16, vmvnq_u16, vorrq_u16, vpaddlq_u8, vrbitq_u8, vreinterpretq_u8_u16,
    vreinterpretq_u16_u8, vrev16q_u8, vrev64q_u16, vshlq_n_u16, vshrq_n_u16,
};

use rand::Rng;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Mask {
    low: uint16x8_t,
    high: uint16x8_t,
}

impl Mask {
    pub const fn empty() -> Self {
        Self::from_u16([0u16; 16])
    }

    pub const fn full() -> Self {
        Self::from_u16([u16::MAX; 16])
    }

    pub fn one_hot(row: usize, col: usize) -> Self {
        debug_assert!(row < 16, "row out of bounds");
        debug_assert!(col < 16, "col out of bounds");

        let mut array = [0u16; 16];
        array[row] = 1 << (15 - col);
        array.into()
    }

    pub fn ne(self, rhs: Self) -> bool {
        self.xor(rhs).any()
    }

    pub fn eq(self, rhs: Self) -> bool {
        !self.ne(rhs)
    }

    pub fn and(self, rhs: Self) -> Self {
        unsafe {
            Self {
                low: vandq_u16(self.low, rhs.low),
                high: vandq_u16(self.high, rhs.high),
            }
        }
    }

    pub fn and_not(self, rhs: Self) -> Self {
        unsafe {
            Self {
                low: vbicq_u16(self.low, rhs.low),
                high: vbicq_u16(self.high, rhs.high),
            }
        }
    }

    pub fn or(self, rhs: Self) -> Self {
        unsafe {
            Self {
                low: vorrq_u16(self.low, rhs.low),
                high: vorrq_u16(self.high, rhs.high),
            }
        }
    }

    pub fn xor(self, rhs: Self) -> Self {
        unsafe {
            Self {
                low: veorq_u16(self.low, rhs.low),
                high: veorq_u16(self.high, rhs.high),
            }
        }
    }

    pub fn not(self) -> Self {
        unsafe {
            Self {
                low: vmvnq_u16(self.low),
                high: vmvnq_u16(self.high),
            }
        }
    }

    pub fn shift_left(self) -> Self {
        unsafe {
            Self {
                low: vshlq_n_u16(self.low, 1),
                high: vshlq_n_u16(self.high, 1),
            }
        }
    }

    pub fn shift_right(self) -> Self {
        unsafe {
            Self {
                low: vshrq_n_u16(self.low, 1),
                high: vshrq_n_u16(self.high, 1),
            }
        }
    }

    pub fn shift_up(self) -> Self {
        unsafe {
            Self {
                low: vextq_u16(self.low, self.high, 1),
                high: vextq_u16(self.high, vdupq_n_u16(0), 1),
            }
        }
    }

    pub fn shift_down(self) -> Self {
        unsafe {
            Self {
                low: vextq_u16(vdupq_n_u16(0), self.low, 7),
                high: vextq_u16(self.low, self.high, 7),
            }
        }
    }

    pub fn neighbors(self) -> Self {
        self.shift_left()
            .or(self.shift_right())
            .or(self.shift_up())
            .or(self.shift_down())
            .and_not(self)
    }

    pub fn neighbors2(self) -> Self {
        let neighbors = self
            .shift_left()
            .or(self.shift_right())
            .or(self.shift_up())
            .or(self.shift_down());
        neighbors
            .or(neighbors.shift_left())
            .or(neighbors.shift_right())
            .or(neighbors.shift_up())
            .or(neighbors.shift_down())
            .and_not(self)
    }

    pub fn count(self) -> u32 {
        unsafe {
            let pc_low = vcntq_u8(vreinterpretq_u8_u16(self.low));
            let pc_high = vcntq_u8(vreinterpretq_u8_u16(self.high));
            let h_low = vpaddlq_u8(pc_low);
            let h_high = vpaddlq_u8(pc_high);
            vaddlvq_u16(vaddq_u16(h_low, h_high))
        }
    }

    pub fn any(self) -> bool {
        !self.is_empty()
    }

    pub fn is_empty(self) -> bool {
        unsafe { vmaxvq_u16(vorrq_u16(self.low, self.high)) == 0 }
    }

    pub fn is_full(self) -> bool {
        unsafe { vminvq_u16(vandq_u16(self.low, self.high)) == u16::MAX }
    }

    pub fn score(self, scores: &Scores) -> u32 {
        unsafe {
            let one = vdupq_n_u16(1);
            let mut total = vdupq_n_u32(0);

            macro_rules! iteration {
                (0) => {
                    let low_bits = vandq_u16(self.low, one);
                    let high_bits = vandq_u16(self.high, one);
                    let bits = vcombine_u8(vmovn_u16(low_bits), vmovn_u16(high_bits));
                    total = vdotq_u32(total, bits, scores.0[0]);
                };

                ($i:expr) => {
                    let low_bits = vandq_u16(vshrq_n_u16(self.low, $i), one);
                    let high_bits = vandq_u16(vshrq_n_u16(self.high, $i), one);
                    let bits = vcombine_u8(vmovn_u16(low_bits), vmovn_u16(high_bits));
                    total = vdotq_u32(total, bits, scores.0[$i as usize]);
                };
            }

            iteration!(0);
            iteration!(1);
            iteration!(2);
            iteration!(3);
            iteration!(4);
            iteration!(5);
            iteration!(6);
            iteration!(7);
            iteration!(8);
            iteration!(9);
            iteration!(10);
            iteration!(11);
            iteration!(12);
            iteration!(13);
            iteration!(14);
            iteration!(15);

            vaddvq_u32(total) as u32
        }
    }

    pub fn get(self, row: usize, col: usize) -> bool {
        debug_assert!(row < 16, "row out of bounds");
        debug_assert!(col < 16, "col out of bounds");

        let array: [u16; 16] = self.into();
        (array[row] >> (15 - col)) & 1 == 1
    }

    pub fn flip_horizontal(self) -> Self {
        fn fliph(v: uint16x8_t) -> uint16x8_t {
            unsafe {
                let v_u8 = vreinterpretq_u8_u16(v);
                let v_rev_u8 = vrbitq_u8(v_u8);
                let v_rev_u16 = vrev16q_u8(v_rev_u8);
                vreinterpretq_u16_u8(v_rev_u16)
            }
        }

        Self {
            low: fliph(self.low),
            high: fliph(self.high),
        }
    }

    pub fn flip_vertical(self) -> Self {
        fn flipv(v: uint16x8_t) -> uint16x8_t {
            unsafe {
                let rev = vrev64q_u16(v);
                vextq_u16(rev, rev, 4)
            }
        }

        Self {
            low: flipv(self.high),
            high: flipv(self.low),
        }
    }

    pub fn flip(self) -> Self {
        self.flip_horizontal().flip_vertical()
    }

    pub fn sample(self, rng: &mut impl Rng) -> Self {
        let array = self.into_u64();
        let [x0, x1, x2, x3] = array;

        let count0 = x0.count_ones();
        let count1 = x1.count_ones();
        let count2 = x2.count_ones();
        let count3 = x3.count_ones();

        let total = count0 + count1 + count2 + count3;
        let mut k = rng.random_range(0..total);

        let in_x2_or_x3 = k >= count0 + count1;
        k -= (count0 + count1) * u32::from(in_x2_or_x3);
        let count = if in_x2_or_x3 { count2 } else { count0 };
        let in_x1_or_x3 = k >= count;
        k -= count * u32::from(in_x1_or_x3);

        let i = 2 * usize::from(in_x2_or_x3) + usize::from(in_x1_or_x3);
        let mut out = [0u64; 4];
        out[i] = get_kth_one(array[i], k);
        Self::from_u64(out)
    }

    pub fn bfs(mut self, accessible: Self) -> Self {
        loop {
            let captured = self.neighbors().and(accessible);
            if captured.any() {
                self = self.or(captured);
            } else {
                return self;
            }
        }
    }

    pub const fn into_u16(self) -> [u16; 16] {
        unsafe { core::mem::transmute(self) }
    }

    pub const fn from_u16(a: [u16; 16]) -> Self {
        unsafe { core::mem::transmute(a) }
    }

    pub const fn into_u64(self) -> [u64; 4] {
        unsafe { core::mem::transmute(self) }
    }

    pub const fn from_u64(a: [u64; 4]) -> Self {
        unsafe { core::mem::transmute(a) }
    }
}

impl From<[u16; 16]> for Mask {
    fn from(a: [u16; 16]) -> Self {
        unsafe { core::mem::transmute(a) }
    }
}

impl From<Mask> for [u16; 16] {
    fn from(m: Mask) -> Self {
        unsafe { core::mem::transmute(m) }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Scores([uint8x16_t; 16]);

impl Scores {
    pub fn new(array: [[u8; 16]; 16]) -> Self {
        let mut out = [[0u8; 16]; 16];
        for i in 0..16 {
            for j in 0..16 {
                out[j][i] = array[j][i];
            }
        }
        unsafe { Self(core::mem::transmute(out)) }
    }
}

fn get_kth_one(mask: u64, mut k: u32) -> u64 {
    let mut shift = 0;

    macro_rules! iteration {
        ($x:expr) => {{
            #![allow(unused_assignments)]
            let submask = (mask >> shift) & (1 << $x) - 1;
            let count = submask.count_ones();
            let in_high_bits = k >= count;
            k -= count * u32::from(in_high_bits);
            shift += $x * u64::from(in_high_bits);
        }};
    }

    iteration!(32);
    iteration!(16);
    iteration!(8);
    iteration!(4);
    iteration!(2);
    iteration!(1);

    1 << shift
}
