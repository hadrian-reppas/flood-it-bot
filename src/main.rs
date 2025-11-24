#![feature(stdarch_neon_dotprod)]
#![warn(clippy::pedantic)]

mod bot;
mod game;
mod mask;

fn main() {
    let mut state = game::State::generate(2);
    let mut player1 = true;
    let mut game_over = false;

    state.print();

    while !game_over {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let color = bot::greedy_move(&state, player1);
        println!("\nPlayer{}: {}", 2 - i32::from(player1), color);
        game_over = state.play(color, player1);
        state.print();
        player1 = !player1;
    }

    let player1_points = state.player1.count();
    let player2_points = state.player2.count();
    if player1_points > player2_points {
        println!("\nPlayer1 wins {player1_points}-{player2_points}");
    } else {
        println!("\nPlayer2 wins {player2_points}-{player1_points}");
    }
}
