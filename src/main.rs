#![feature(stdarch_neon_dotprod)]
#![warn(clippy::pedantic)]

use std::sync::{Mutex, mpsc};

use rand::prelude::*;

mod bot;
mod game;
mod mask;

use bot::*;

struct Contestant {
    name: &'static str,
    make: fn() -> Box<dyn Player>,
}

macro_rules! contestant {
    ($name:expr, $make:expr) => {
        Contestant {
            name: $name,
            make: || Box::new($make),
        }
    };

    ($make:expr) => {
        contestant!(stringify!($make), $make)
    };
}

#[derive(Clone, Copy, Debug)]
struct Outcome {
    p1: usize,
    p2: usize,
    margin: i32,
}

const CONTESTANTS: &[Contestant] = &[
    contestant!(Negamax(CloserCaptured, 2)),
    contestant!(Negamax(CloserCaptured, 4)),
    contestant!(Negamax(CloserCaptured, 6)),
    contestant!(Negamax(Accessible, 8)),
    contestant!(Negamax(AccessibleCaptured, 8)),
    contestant!(Negamax(CloserCaptured, 8)),
    contestant!(Negamax(AccessibleCloser, 8)),
];

fn play(seed: u64, player1: &mut dyn Player, player2: &mut dyn Player) -> i32 {
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

fn compare(seed: u64, c1: &Contestant, c2: &Contestant) -> i32 {
    let mut player1 = (c1.make)();
    let mut player2 = (c2.make)();
    let margin1 = play(seed, player1.as_mut(), player2.as_mut());

    player1 = (c1.make)();
    player2 = (c2.make)();
    let margin2 = play(seed, player2.as_mut(), player1.as_mut());

    margin1 - margin2
}

fn get_job() -> (usize, usize) {
    static QUEUE: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());

    let mut queue = QUEUE.lock().unwrap();
    if queue.is_empty() {
        for i in 0..CONTESTANTS.len() {
            for j in 0..i {
                queue.push((i, j));
            }
        }
        queue.shuffle(&mut rand::rng());
    }
    queue.pop().unwrap()
}

fn runner(tx: mpsc::Sender<Outcome>) {
    loop {
        let (p1, p2) = get_job();
        let margin = compare(rand::random(), &CONTESTANTS[p1], &CONTESTANTS[p2]);
        tx.send(Outcome { p1, p2, margin }).unwrap();
    }
}

fn scorekeeper(rx: mpsc::Receiver<Outcome>) {
    const K: f64 = 16.0;

    let mut scores = [400.0; CONTESTANTS.len()];
    let mut matches = [0; CONTESTANTS.len()];

    while let Ok(Outcome { p1, p2, margin }) = rx.recv() {
        let ra = scores[p1];
        let rb = scores[p2];

        let ea = 1.0 / (1.0 + 10f64.powf((rb - ra) / 400.0));
        let eb = 1.0 - ea;

        let sa = if margin > 0 {
            1.0
        } else if margin < 0 {
            0.0
        } else {
            0.5
        };
        let sb = 1.0 - sa;

        scores[p1] = ra + K * (sa - ea);
        scores[p2] = rb + K * (sb - eb);

        matches[p1] += 1;
        matches[p2] += 1;

        println!("+------------------------------------------+--------+---------+");
        println!("| Name                                     | Elo    | Matches |");
        println!("+------------------------------------------+--------+---------+");
        for ((contestant, score), matches) in CONTESTANTS.iter().zip(scores).zip(matches) {
            println!(
                "| {:>40} | {:>6.1} | {:>7} |",
                contestant.name, score, matches
            );
        }
        println!("+------------------------------------------+--------+---------+");
    }
}

fn main() {
    /*
    let mut state = game::State::generate(1351235);
    let (a, b, c) = state.player1.closer(state.player2, state.walls.not());
    state.colors[0] = a;
    state.colors[1] = b;
    state.colors[2] = c;
    state.colors[3] = mask::Mask::full();

    state.print();

    return;
    */

    let (tx, rx) = mpsc::channel();
    for _ in 0..8 {
        let tx = tx.clone();
        std::thread::spawn(|| runner(tx));
    }
    scorekeeper(rx);
}
