use rand::prelude::*;
use rand_pcg::Pcg64;
use termion::color::{
    Bg, Black, Blue, Cyan, Green, LightBlack, LightMagenta, LightRed, Magenta, Red, Reset, White,
    Yellow,
};

use crate::mask::Mask;

const PROTECTED: [(usize, usize); 24] = [
    (0, 1),
    (0, 2),
    (0, 3),
    (1, 0),
    (1, 1),
    (1, 2),
    (1, 3),
    (2, 0),
    (2, 1),
    (2, 2),
    (3, 0),
    (3, 1),
    (12, 14),
    (12, 15),
    (13, 13),
    (13, 14),
    (13, 15),
    (14, 12),
    (14, 13),
    (14, 14),
    (14, 15),
    (15, 12),
    (15, 13),
    (15, 14),
];

fn generate(seed: u64) -> ([Mask; 8], Mask) {
    let mut rng = Pcg64::seed_from_u64(seed);
    let mut colors = [Mask::empty(); 8];
    let mut used = Mask::one_hot(0, 0).or(Mask::one_hot(15, 15));

    macro_rules! add_color {
        ($mask:expr) => {
            let mask = $mask;
            if !mask.and(used).any() {
                let i = rng.random_range(0..8);
                colors[i] = colors[i].or(mask);
                used = used.or(mask);
            }
        };
    }

    for (r, c) in PROTECTED {
        add_color!(Mask::one_hot(r, c));
    }

    let mut directions = [false; 28];
    for i in 0..14 {
        directions[i] = true;
    }

    directions.shuffle(&mut rng);
    let mut position = Mask::one_hot(1, 1);
    for go_right in directions {
        if go_right {
            position = position.shift_right();
        } else {
            position = position.shift_down();
        }
        add_color!(position);
    }

    directions.shuffle(&mut rng);
    let mut position = Mask::one_hot(rng.random_range(13..16), rng.random_range(0..3));
    for go_right in directions {
        if go_right {
            position = position.shift_right();
        } else {
            position = position.shift_up();
        }
        add_color!(position);
    }

    let wall_count = 31 + 2 * rng.random_range(0..=16);

    let todo = 256 - used.count() - wall_count;
    for _ in 0..todo {
        let neighbors = used.neighbors();
        add_color!(neighbors.sample(&mut rng));
    }

    (colors, used.not())
}

#[derive(Clone, Copy)]
pub struct State {
    pub colors: [Mask; 8],
    pub walls: Mask,
    pub player1: Mask,
    pub player2: Mask,
    pub player1_last_move: Option<u8>,
    pub player2_last_move: Option<u8>,
}

impl State {
    pub fn generate(seed: u64) -> Self {
        let (colors, walls) = generate(seed);
        Self {
            colors,
            walls,
            player1: Mask::one_hot(0, 0),
            player2: Mask::one_hot(15, 15),
            player1_last_move: None,
            player2_last_move: None,
        }
    }

    pub fn print(&self) {
        for r in 0..16 {
            for c in 0..16 {
                macro_rules! test {
                    ($mask:expr, $color:ident) => {
                        if $mask.and(Mask::one_hot(r, c)).any() {
                            print!("{}  ", Bg($color));
                            continue;
                        }
                    };
                }

                test!(self.colors[0], Red);
                test!(self.colors[1], LightRed);
                test!(self.colors[2], Yellow);
                test!(self.colors[3], Green);
                test!(self.colors[4], Cyan);
                test!(self.colors[5], Blue);
                test!(self.colors[6], Magenta);
                test!(self.colors[7], LightMagenta);
                test!(self.walls, Black);
                test!(self.player1, White);
                test!(self.player2, LightBlack);

                panic!();
            }
            println!("{}", Bg(Reset));
        }
    }

    pub fn is_valid(&self) -> bool {
        let mut seen = Mask::empty();

        macro_rules! check {
            ($mask:expr) => {
                if seen.and($mask).any() {
                    return false;
                }
                seen = seen.or($mask);
            };
        }

        for color in self.colors {
            check!(color);
        }
        check!(self.walls);
        check!(self.player1);
        check!(self.player2);

        seen.eq(Mask::full())
    }
}
