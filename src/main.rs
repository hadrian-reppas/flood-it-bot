#![feature(stdarch_neon_dotprod)]
#![warn(clippy::pedantic)]

use std::sync::{Mutex, mpsc};
use std::time::{Duration, Instant};

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
struct ContestantStats {
    elo: f32,
    matches: u32,
    time: Duration,
    rounds: u32,
}

impl ContestantStats {
    fn new() -> Self {
        Self {
            elo: 400.0,
            matches: 0,
            time: Duration::ZERO,
            rounds: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct MatchResult {
    p1: usize,
    p2: usize,
    margin: i32,
    p1_time: Duration,
    p2_time: Duration,
    rounds: u32,
}

impl MatchResult {
    fn update(&self, stats: &mut [ContestantStats]) {
        const K: f32 = 16.0;

        let ra = stats[self.p1].elo;
        let rb = stats[self.p2].elo;

        let ea = 1.0 / (1.0 + 10f32.powf((rb - ra) / 400.0));
        let eb = 1.0 - ea;

        let sa = if self.margin > 0 {
            1.0
        } else if self.margin < 0 {
            0.0
        } else {
            0.5
        };
        let sb = 1.0 - sa;

        stats[self.p1].elo = ra + K * (sa - ea);
        stats[self.p2].elo = rb + K * (sb - eb);

        stats[self.p1].matches += 1;
        stats[self.p2].matches += 1;

        stats[self.p1].time += self.p1_time;
        stats[self.p2].time += self.p2_time;

        stats[self.p1].rounds += self.rounds;
        stats[self.p2].rounds += self.rounds;
    }
}

#[derive(Clone, Copy, Debug)]
struct GameResult {
    margin: i32,
    p1_time: Duration,
    p2_time: Duration,
    rounds: u32,
}

const CONTESTANTS: &[Contestant] = &[
    contestant!(Negamax(CloserCaptured, 0)),
    contestant!(Negamax(CloserCaptured, 1)),
    contestant!(Negamax(CloserCaptured, 2)),
    contestant!(Negamax(CloserCaptured, 3)),
    contestant!(Negamax(CloserCaptured, 4)),
    contestant!(Negamax(CloserCaptured, 5)),
    contestant!(Negamax(CloserCaptured, 6)),
    contestant!(Negamax(CloserCaptured, 7)),
    contestant!(Negamax(CloserCaptured, 8)),
    contestant!(Negamax(Accessible, 8)),
    contestant!(Negamax(AccessibleCaptured, 8)),
    contestant!(Negamax(AccessibleCloser, 8)),
];

fn play_game(seed: u64, player1: &mut dyn Player, player2: &mut dyn Player) -> GameResult {
    let mut state = game::State::generate(seed);
    let mut p1_time = Duration::ZERO;
    let mut p2_time = Duration::ZERO;

    while !state.game_over() {
        let start = Instant::now();
        let color;
        if state.player1_next() {
            color = player1.play(&state);
            p1_time += start.elapsed();
        } else {
            color = player2.play(&state);
            p2_time += start.elapsed();
        };
        state.play(color);
    }

    GameResult {
        margin: state.final_margin(),
        p1_time,
        p2_time,
        rounds: state.round,
    }
}

fn play_match(seed: u64, p1: usize, p2: usize) -> MatchResult {
    let mut player1 = (CONTESTANTS[p1].make)();
    let mut player2 = (CONTESTANTS[p2].make)();
    let game1 = play_game(seed, player1.as_mut(), player2.as_mut());

    player1 = (CONTESTANTS[p1].make)();
    player2 = (CONTESTANTS[p2].make)();
    let game2 = play_game(seed, player2.as_mut(), player1.as_mut());

    MatchResult {
        p1,
        p2,
        margin: game1.margin - game2.margin,
        p1_time: game1.p1_time + game2.p1_time,
        p2_time: game1.p2_time + game2.p2_time,
        rounds: game1.rounds + game2.rounds,
    }
}

fn get_job() -> (usize, usize) {
    static QUEUE: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());

    let mut queue = QUEUE.lock().unwrap();
    if queue.is_empty() {
        for i in 0..CONTESTANTS.len() {
            for j in 0..CONTESTANTS.len() {
                if i != j {
                    queue.push((i, j));
                }
            }
        }
        queue.shuffle(&mut rand::rng());
    }
    queue.pop().unwrap()
}

fn runner(tx: mpsc::Sender<MatchResult>) {
    loop {
        let (p1, p2) = get_job();
        let seed = rand::random();

        let mut player1 = (CONTESTANTS[p1].make)();
        let mut player2 = (CONTESTANTS[p2].make)();
        let game1 = play_game(seed, player1.as_mut(), player2.as_mut());

        player1 = (CONTESTANTS[p1].make)();
        player2 = (CONTESTANTS[p2].make)();
        let game2 = play_game(seed, player2.as_mut(), player1.as_mut());

        tx.send(MatchResult {
            p1,
            p2,
            margin: game1.margin - game2.margin,
            p1_time: game1.p1_time + game2.p1_time,
            p2_time: game1.p2_time + game2.p2_time,
            rounds: game1.rounds + game2.rounds,
        })
        .unwrap();
    }
}

fn scorekeeper(rx: mpsc::Receiver<MatchResult>) {
    let mut stats = [ContestantStats::new(); CONTESTANTS.len()];

    while let Ok(result) = rx.recv() {
        result.update(&mut stats);

        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
        println!(
            "+------------------------------------------+--------+---------+----------------+"
        );
        println!(
            "| Name                                     | Elo    | Matches | Time           |"
        );
        println!(
            "+------------------------------------------+--------+---------+----------------+"
        );
        for (contestant, stats) in CONTESTANTS.iter().zip(&stats) {
            let avg_time = format!("{:?}", stats.time / stats.rounds.max(1));

            println!(
                "| {:>40} | {:>6.1} | {:>7} | {:>14} |",
                contestant.name, stats.elo, stats.matches, avg_time
            );
        }
        println!(
            "+------------------------------------------+--------+---------+----------------+"
        );
    }
}

fn main() {
    print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));

    let (tx, rx) = mpsc::channel();
    for _ in 0..8 {
        let tx = tx.clone();
        std::thread::spawn(|| runner(tx));
    }
    scorekeeper(rx);
}
