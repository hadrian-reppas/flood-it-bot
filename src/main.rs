#![feature(stdarch_neon_dotprod)]
#![warn(clippy::pedantic)]

mod game;
mod mask;

fn main() {
    for seed in 0..100 {
        let state = game::State::generate(seed);
        state.print();
        println!();
    }

    let start = std::time::Instant::now();
    for seed in 0..1_000_000 {
        assert!(game::State::generate(seed).is_valid());
    }
    println!("{:?}", start.elapsed());
}
