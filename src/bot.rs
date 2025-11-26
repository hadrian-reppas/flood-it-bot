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
        return sign * state.final_margin();
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

pub trait Eval {
    fn eval(&self, state: &State) -> i32;
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Negamax<E> {
    eval: E,
    depth: u32,
}

impl<E> Negamax<E> {
    pub fn new(eval: E, depth: u32) -> Self {
        Self { eval, depth }
    }
}

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
            let value = -negamax(
                &mut state, &self.eval, self.depth, -INFINITY, INFINITY, -sign,
            );
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
pub struct TerritoryDiff;

impl Eval for TerritoryDiff {
    fn eval(&self, state: &State) -> i32 {
        let accessible = state.player1.or(state.player2).or(state.walls).not();
        let player1_accessible = state.player1.bfs(accessible);
        let player2_accessible = state.player2.bfs(accessible);
        player1_accessible.count() as i32 - player2_accessible.count() as i32
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct TerritoryDiff2;

impl Eval for TerritoryDiff2 {
    fn eval(&self, state: &State) -> i32 {
        let accessible = state.player1.or(state.player2).or(state.walls).not();
        let player1_accessible = state.player1.bfs(accessible);
        let player2_accessible = state.player2.bfs(accessible);
        1000 * (player1_accessible.count() as i32 - player2_accessible.count() as i32)
            + state.player1.count() as i32
            + state.player2.count() as i32
    }
}
