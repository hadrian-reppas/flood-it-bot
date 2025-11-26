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
const ROUND_LIMIT: u32 = 100;

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

    let wall_count = rng.random_range(32..=64);

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
    pub round: u32,
    pub seed: u64,
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
            round: 0,
            seed,
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

                test!(self.walls, Black);
                test!(self.player1, White);
                test!(self.player2, LightBlack);
                test!(self.colors[0], Red);
                test!(self.colors[1], LightRed);
                test!(self.colors[2], Yellow);
                test!(self.colors[3], Green);
                test!(self.colors[4], Cyan);
                test!(self.colors[5], Blue);
                test!(self.colors[6], Magenta);
                test!(self.colors[7], LightMagenta);

                panic!();
            }
            println!("{}", Bg(Reset));
        }
    }

    pub fn is_valid(&self) -> bool {
        if (self.round >= 1) != self.player1_last_move.is_some()
            || (self.round >= 2) != self.player2_last_move.is_some()
        {
            return false;
        }

        let mut seen = self.walls;

        macro_rules! check {
            ($mask:expr) => {{
                #![allow(unused_assignments)]
                if seen.and($mask).any() {
                    return false;
                }
                seen = seen.or($mask);
            }};
        }

        check!(self.player1);
        check!(self.player2);

        seen = Mask::one_hot(0, 0).or(Mask::one_hot(15, 15));

        for color in self.colors {
            check!(color);
        }
        check!(self.walls);

        seen.eq(Mask::full())
    }

    pub fn play(&mut self, color: u8) {
        debug_assert!(self.is_valid());
        debug_assert!(!self.game_over());
        debug_assert!(color < 8);
        debug_assert!(Some(color) != self.player1_last_move);
        debug_assert!(Some(color) != self.player2_last_move);

        if self.player1_next() {
            self.player1 = self
                .player1
                .bfs(self.colors[color as usize].and_not(self.player2));
            self.player1_last_move = Some(color);
        } else {
            self.player2 = self
                .player2
                .bfs(self.colors[color as usize].and_not(self.player1));
            self.player2_last_move = Some(color);
        }

        self.round += 1;
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            players: self.player1.or(self.player2),
            player1_last_move: self.player1_last_move,
            player2_last_move: self.player2_last_move,
            round: self.round,
        }
    }

    pub fn restore(&mut self, checkpoint: Checkpoint) {
        self.player1 = self.player1.and(checkpoint.players);
        self.player2 = self.player2.and(checkpoint.players);
        self.player1_last_move = checkpoint.player1_last_move;
        self.player2_last_move = checkpoint.player2_last_move;
        self.round = checkpoint.round;
    }

    pub fn player1_next(&self) -> bool {
        self.round % 2 == 0
    }

    pub fn game_over(&self) -> bool {
        if self.round == ROUND_LIMIT {
            return true;
        }

        let accessible = self.player1.or(self.player2).or(self.walls).not();
        let player1_accessible = self.player1.bfs(accessible);
        let player2_accessible = self.player2.bfs(accessible);
        player1_accessible.and(player2_accessible).is_empty()
    }

    pub fn final_margin(&self) -> i32 {
        debug_assert!(self.game_over());

        let accessible = self.player1.or(self.player2).or(self.walls).not();
        let player1 = self.player1.bfs(accessible);
        let player2 = self.player2.bfs(accessible);
        player1.count() as i32 - player2.count() as i32
    }

    pub fn finalize(&mut self) {
        debug_assert!(self.game_over());

        let accessible = self.player1.or(self.player2).or(self.walls).not();
        self.player1 = self.player1.bfs(accessible);
        self.player2 = self.player2.bfs(accessible);
    }
}

#[derive(Clone, Copy)]
pub struct Checkpoint {
    pub players: Mask,
    pub player1_last_move: Option<u8>,
    pub player2_last_move: Option<u8>,
    pub round: u32,
}
