#![feature(stdarch_neon_dotprod)]
#![warn(clippy::pedantic)]

mod bot;
mod game;
mod mask;

use bot::*;

fn play(seed: u64, player1: &mut impl Player, player2: &mut impl Player) -> i32 {
    let mut state = game::State::generate(seed);
    state.print();
    println!();

    while !state.game_over() {
        std::thread::sleep(std::time::Duration::from_millis(800));
        let color = if state.player1_next() {
            player1.play(&state)
        } else {
            player2.play(&state)
        };
        state.play(color);
        state.print();
        println!();
    }

    std::thread::sleep(std::time::Duration::from_millis(800));
    state.finalize();
    state.print();
    println!();
    std::thread::sleep(std::time::Duration::from_secs(2));

    state.final_margin()
}

fn main() {
    let seed = std::env::args().nth(1).unwrap().parse().unwrap();

    let mut player1 = Negamax::new(TerritoryDiff, 8);
    let mut player2 = Negamax::new(TerritoryDiff, 0);

    let margin1 = play(seed, &mut player1, &mut player2);
    let margin2 = play(seed, &mut player2, &mut player1);
    println!("{} ({margin1} vs {margin2})", margin1 - margin2);
}
