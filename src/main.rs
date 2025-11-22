use core::arch::aarch64::{
    uint16x8_t, vaddlvq_u16, vaddq_u16, vbicq_u16, vcntq_u8, vdupq_n_u16, vextq_u16, vmaxvq_u16,
    vorrq_u16, vpaddlq_u8, vreinterpretq_u8_u16, vshlq_n_u16, vshrq_n_u16,
};

#[repr(C)]
#[derive(Clone, Copy)]
struct Mask {
    low: uint16x8_t,
    high: uint16x8_t,
}

impl Mask {
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
    
    fn left(self) -> Self {
        unsafe {
            Self {
                low: vshlq_n_u16(self.low, 1),
                high: vshlq_n_u16(self.high, 1),
            }
        }
    }

    fn right(self) -> Self {
        unsafe {
            Self {
                low: vshrq_n_u16(self.low, 1),
                high: vshrq_n_u16(self.high, 1),
            }
        }
    }

    fn up(self) -> Self {
        unsafe {
            Self {
                low: vextq_u16(vdupq_n_u16(0), self.low, 7),
                high: vextq_u16(self.low, self.high, 7),
            }
        }
    }

    fn down(self) -> Self {
        unsafe {
            Self {
                low: vextq_u16(self.low, self.high, 1),
                high: vextq_u16(self.high, vdupq_n_u16(0), 1),
            }
        }
    }

    fn neighbors(self) -> Self {
        unsafe {
            let left_low = vshlq_n_u16(self.low, 1);
            let left_high = vshlq_n_u16(self.high, 1);
            let right_low = vshrq_n_u16(self.low, 1);
            let right_high = vshrq_n_u16(self.high, 1);

            let zero = vdupq_n_u16(0);
            let up_low = vextq_u16(zero, self.low, 7);
            let up_high = vextq_u16(self.low, self.high, 7);
            let down_low = vextq_u16(self.low, self.high, 1);
            let down_high = vextq_u16(self.high, zero, 1);

            let horizontal_low = vorrq_u16(left_low, right_low);
            let horizontal_high = vorrq_u16(left_high, right_high);

            let vertical_low = vorrq_u16(up_low, down_low);
            let vertical_high = vorrq_u16(up_high, down_high);

            let result_low = vorrq_u16(horizontal_low, vertical_low);
            let result_high = vorrq_u16(horizontal_high, vertical_high);

            Mask {
                low: vbicq_u16(result_low, self.low),
                high: vbicq_u16(result_high, self.high),
            }
        }
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

    fn print(self) {
        let array = unsafe { core::mem::transmute::<_, [u16; 16]>(self) };
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

fn main() {
    let mut buf = [0u16; 16];
    rand::fill(&mut buf);
    let mask = unsafe { core::mem::transmute::<_, Mask>(buf) };
    mask.print();
    mask.left().print();
    mask.right().print();
}
