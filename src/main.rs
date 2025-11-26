#![feature(stdarch_neon_dotprod)]
#![warn(clippy::pedantic)]

mod bot;
mod game;
mod mask;

use bot::*;

fn play<P1: Player, P2: Player>(seed: u64, player1: &mut P1, player2: &mut P2) -> i32 {
    let mut state = game::State::generate(seed);

    while !state.game_over() {
        let color = if state.player1_next() {
            player1.play(&state)
        } else {
            player2.play(&state)
        };
        state.play(color);
    }

    state.final_margin()
}

fn compare<F1, F2, P1: Player, P2: Player>(seed: u64, make_player1: F1, make_player2: F2) -> i32
where
    F1: Fn() -> P1,
    F2: Fn() -> P2,
{
    let mut player1 = make_player1();
    let mut player2 = make_player2();
    let margin1 = play(seed, &mut player1, &mut player2);

    let mut player1 = make_player1();
    let mut player2 = make_player2();
    let margin2 = play(seed, &mut player2, &mut player1);

    margin1 - margin2
}

fn main() {
    for seed in 989..999 {
        let result = compare(
            seed,
            || Negamax::new(TerritoryDiff2, 8),
            || Negamax::new(TerritoryDiff, 8),
        );
        println!("{result}");
    }
}
