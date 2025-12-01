use core::cmp::max;

use crate::game::State;

const INFINITY: i32 = 1_000_000_000;

pub fn negamax<E: Eval>(
    state: &mut State,
    eval: &E,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    sign: i32,
) -> i32 {
    if state.game_over() {
        return sign * (INFINITY + state.final_margin() + depth as i32);
    } else if depth == 0 {
        return sign * eval.eval(state);
    }

    let mut max_value = i32::MIN;
    let checkpoint = state.checkpoint();
    for color in 0..8 {
        if Some(color) == state.player1_last_move || Some(color) == state.player2_last_move {
            continue;
        }

        state.play(color);
        let value = -negamax(state, eval, depth - 1, -beta, -alpha, -sign);
        state.restore(checkpoint);

        max_value = max(max_value, value);
        alpha = max(alpha, value);
        if alpha >= beta {
            break;
        }
    }
    max_value
}

pub trait Player {
    fn play(&mut self, state: &State) -> u8;
}

pub struct Greedy;

impl Player for Greedy {
    fn play(&mut self, state: &State) -> u8 {
        (0..8)
            .filter(|c| Some(*c) != state.player1_last_move && Some(*c) != state.player2_last_move)
            .max_by_key(|c| {
                let mut state = *state;
                state.play(*c);
                if state.player1_next() {
                    state.player2.count()
                } else {
                    state.player1.count()
                }
            })
            .unwrap()
    }
}

pub trait Eval {
    fn eval(&self, state: &State) -> i32;
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Negamax<E>(pub E, pub u32);

impl<E: Eval> Player for Negamax<E> {
    fn play(&mut self, state: &State) -> u8 {
        let mut state = *state;
        let checkpoint = state.checkpoint();
        let sign = if state.player1_next() { 1 } else { -1 };

        let mut max_value = i32::MIN;
        let mut best_move = u8::MAX;

        for color in 0..8 {
            if Some(color) == state.player1_last_move || Some(color) == state.player2_last_move {
                continue;
            }

            state.play(color);
            let value = -negamax(&mut state, &self.0, self.1, -INFINITY, INFINITY, -sign);
            state.restore(checkpoint);

            if value > max_value {
                max_value = value;
                best_move = color;
            }
        }
        best_move
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Captured;

impl Eval for Captured {
    fn eval(&self, state: &State) -> i32 {
        state.player1.count() as i32 - state.player2.count() as i32
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Accessible;

impl Eval for Accessible {
    fn eval(&self, state: &State) -> i32 {
        let accessible = state.player1.or(state.player2).or(state.walls).not();
        let player1_accessible = state.player1.bfs(accessible);
        let player2_accessible = state.player2.bfs(accessible);
        player1_accessible.count() as i32 - player2_accessible.count() as i32
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Closer;

impl Eval for Closer {
    fn eval(&self, state: &State) -> i32 {
        let accessible = state.player1.or(state.player2).or(state.walls).not();
        let (player1_closer, _, player2_closer) = state.player1.closer(state.player2, accessible);
        player1_closer.count() as i32 - player2_closer.count() as i32
    }
}

impl<A: Eval, B: Eval> Eval for (A, B) {
    fn eval(&self, state: &State) -> i32 {
        256 * self.0.eval(state) + self.1.eval(state)
    }
}

impl<A: Eval, B: Eval, C: Eval> Eval for (A, B, C) {
    fn eval(&self, state: &State) -> i32 {
        65536 * self.0.eval(state) + 256 * self.1.eval(state) + self.2.eval(state)
    }
}
