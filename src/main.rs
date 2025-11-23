#![feature(stdarch_neon_dotprod)]
#![warn(clippy::pedantic)]

use core::arch::aarch64::{
    uint8x16_t, uint16x8_t, vaddlvq_u16, vaddq_u16, vaddvq_u32, vandq_u16, vbicq_u16, vcntq_u8,
    vcombine_u8, vdotq_u32, vdupq_n_u16, vdupq_n_u32, veorq_u16, vextq_u16, vmaxvq_u16, vmovn_u16,
    vmvnq_u16, vorrq_u16, vpaddlq_u8, vrbitq_u8, vreinterpretq_u8_u16, vreinterpretq_u16_u8,
    vrev16q_u8, vrev64q_u16, vshlq_n_u16, vshrq_n_u16,
};

#[repr(C)]
#[derive(Clone, Copy)]
struct Mask {
    low: uint16x8_t,
    high: uint16x8_t,
}

impl Mask {
    fn ne(self, rhs: Self) -> bool {
        self.xor(rhs).any()
    }

    fn eq(self, rhs: Self) -> bool {
        !self.ne(rhs)
    }

    fn and(self, rhs: Self) -> Self {
        unsafe {
            Self {
                low: vandq_u16(self.low, rhs.low),
                high: vandq_u16(self.high, rhs.high),
            }
        }
    }

    fn and_not(self, rhs: Self) -> Self {
        unsafe {
            Self {
                low: vbicq_u16(self.low, rhs.low),
                high: vbicq_u16(self.high, rhs.high),
            }
        }
    }

    fn or(self, rhs: Self) -> Self {
        unsafe {
            Self {
                low: vorrq_u16(self.low, rhs.low),
                high: vorrq_u16(self.high, rhs.high),
            }
        }
    }

    fn xor(self, rhs: Self) -> Self {
        unsafe {
            Self {
                low: veorq_u16(self.low, rhs.low),
                high: veorq_u16(self.high, rhs.high),
            }
        }
    }

    fn not(self) -> Self {
        unsafe {
            Self {
                low: vmvnq_u16(self.low),
                high: vmvnq_u16(self.high),
            }
        }
    }

    fn shift_left(self) -> Self {
        unsafe {
            Self {
                low: vshlq_n_u16(self.low, 1),
                high: vshlq_n_u16(self.high, 1),
            }
        }
    }

    fn shift_right(self) -> Self {
        unsafe {
            Self {
                low: vshrq_n_u16(self.low, 1),
                high: vshrq_n_u16(self.high, 1),
            }
        }
    }

    fn shift_up(self) -> Self {
        unsafe {
            Self {
                low: vextq_u16(vdupq_n_u16(0), self.low, 7),
                high: vextq_u16(self.low, self.high, 7),
            }
        }
    }

    fn shift_down(self) -> Self {
        unsafe {
            Self {
                low: vextq_u16(self.low, self.high, 1),
                high: vextq_u16(self.high, vdupq_n_u16(0), 1),
            }
        }
    }

    fn neighbors(self) -> Self {
        self.shift_left()
            .or(self.shift_right())
            .or(self.shift_up())
            .or(self.shift_down())
            .and_not(self)
    }

    fn neighbors2(self) -> Self {
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

    fn count(self) -> u32 {
        unsafe {
            let pc_low = vcntq_u8(vreinterpretq_u8_u16(self.low));
            let pc_high = vcntq_u8(vreinterpretq_u8_u16(self.high));
            let h_low = vpaddlq_u8(pc_low);
            let h_high = vpaddlq_u8(pc_high);
            vaddlvq_u16(vaddq_u16(h_low, h_high))
        }
    }

    fn any(self) -> bool {
        unsafe { vmaxvq_u16(vorrq_u16(self.low, self.high)) != 0 }
    }

    fn score(self, scores: &Scores) -> u32 {
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

    fn get(self, row: usize, col: usize) -> bool {
        debug_assert!(row < 16, "row out of bounds");
        debug_assert!(col < 16, "col out of bounds");

        let array: [u16; 16] = self.into();
        (array[row] >> col) & 1 == 1
    }

    fn flip_horizontal(self) -> Self {
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

    fn flip_vertical(self) -> Self {
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

    fn flip(self) -> Self {
        self.flip_horizontal().flip_vertical()
    }

    fn print(self) {
        let array: [u16; 16] = self.into();
        println!("+----------------+");
        for row in array {
            print!("|");
            for i in (0..16).rev() {
                if (row >> i) & 1 == 1 {
                    print!("#");
                } else {
                    print!(" ");
                }
            }
            println!("|");
        }
        println!("+----------------+");
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
struct Scores([uint8x16_t; 16]);

impl From<[[u8; 16]; 16]> for Scores {
    fn from(array: [[u8; 16]; 16]) -> Self {
        let mut out = [[0u8; 16]; 16];
        for i in 0..16 {
            for j in 0..16 {
                out[j][i] = array[j][i];
            }
        }
        unsafe { Self(core::mem::transmute(out)) }
    }
}

fn main() {
    let mut buf = [0u16; 16];
    rand::fill(&mut buf);

    let mask = Mask::from(buf);

    mask.print();
}
