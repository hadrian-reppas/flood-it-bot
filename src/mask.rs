use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use core::simd::{self, ToBytes, prelude::*};

use rand::Rng;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
pub struct Mask(u16x16);

impl Mask {
    pub const fn empty() -> Self {
        Self(u16x16::splat(0))
    }

    pub const fn full() -> Self {
        Self(u16x16::splat(u16::MAX))
    }

    pub fn one_hot(row: usize, col: usize) -> Self {
        debug_assert!(row < 16, "row out of bounds");
        debug_assert!(col < 16, "col out of bounds");

        let mut array = [0u16; 16];
        array[row] = 1 << col;
        Self(u16x16::from_array(array))
    }

    pub fn shift_left(self) -> Self {
        Self(self.0 << 1)
    }

    pub fn shift_right(self) -> Self {
        Self(self.0 >> 1)
    }

    pub fn shift_up(self) -> Self {
        Self(self.0.shift_elements_left::<1>(0))
    }

    pub fn shift_down(self) -> Self {
        Self(self.0.shift_elements_right::<1>(0))
    }

    pub fn neighbors(self) -> Self {
        (self.shift_left() | self.shift_right() | self.shift_up() | self.shift_down()) & !self
    }

    pub fn neighbors2(self) -> Self {
        let neighbors =
            self.shift_left() | self.shift_right() | self.shift_up() | self.shift_down();
        (neighbors
            | neighbors.shift_left()
            | neighbors.shift_right()
            | neighbors.shift_up()
            | neighbors.shift_down())
            & !self
    }

    pub fn count_ones(self) -> u32 {
        self.0.count_ones().reduce_sum() as u32
    }

    pub fn is_empty(self) -> bool {
        self == Self::empty()
    }

    pub fn is_full(self) -> bool {
        self == Self::full()
    }

    pub fn score(self, scores: &[[u8; 16]; 16]) -> u32 {
        let mut sum = 0;
        let array = self.0.as_array();
        for i in 0..16 {
            let row = u8x16::from_array(scores[i]);
            let mask = simd::Mask::<i8, 16>::from_bitmask(array[i] as u64);
            let select = mask.select(row, u8x16::splat(0));
            sum += select.cast::<u16>().reduce_sum();
        }
        sum as u32
    }

    pub fn get(self, row: usize, col: usize) -> bool {
        debug_assert!(row < 16, "row out of bounds");
        debug_assert!(col < 16, "col out of bounds");

        (self.0.as_array()[row] >> col) & 1 == 1
    }

    pub fn flip_horizontal(self) -> Self {
        Self(self.0.reverse_bits())
    }

    pub fn flip_vertical(self) -> Self {
        Self(self.0.reverse())
    }

    pub fn flip(self) -> Self {
        self.flip_horizontal().flip_vertical()
    }

    pub fn sample(self, rng: &mut impl Rng) -> Self {
        let array = u64x4::from_ne_bytes(self.0.to_ne_bytes()).to_array();
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
        Self(u16x16::from_ne_bytes(u64x4::from_array(out).to_le_bytes()))
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        let mut array: [u16; 16] = [0; 16];
        rng.fill(&mut array);
        array.into()
    }

    pub fn bfs(mut self, accessible: Self) -> Self {
        loop {
            let captured = self.neighbors() & accessible;
            if captured.is_empty() {
                return self;
            } else {
                self |= captured;
            }
        }
    }

    pub fn closer(mut self, mut other: Self, mut accessible: Self) -> (Self, Self, Self) {
        let mut both = Mask::empty();
        loop {
            let self_neighbors = self.neighbors() & accessible & !other & !both;
            let other_neighbors = other.neighbors() & accessible & !self & !both;

            if self_neighbors.is_empty() && other_neighbors.is_empty() {
                return (self, both, other);
            }

            let self_captured = self_neighbors & !other_neighbors;
            let other_captured = other_neighbors & !self_neighbors;
            let both_captured = self_captured & other_captured;

            self |= self_captured;
            other |= other_captured;
            both |= both_captured;
            accessible &= !self_neighbors & !other_neighbors;
        }
    }
}

impl BitAnd for Mask {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Mask {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitXor for Mask {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Mask {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl BitOr for Mask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Mask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Not for Mask {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl From<[u16; 16]> for Mask {
    fn from(array: [u16; 16]) -> Self {
        Self(u16x16::from_array(array))
    }
}

impl From<Mask> for [u16; 16] {
    fn from(mask: Mask) -> Self {
        mask.0.to_array()
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
