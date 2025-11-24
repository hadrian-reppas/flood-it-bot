use crate::game::State;

pub fn greedy_move(state: &State, player1: bool) -> u8 {
    let player = if player1 {
        state.player1
    } else {
        state.player2
    };
    (0..8u8)
        .filter(|&c| Some(c) != state.player1_last_move && Some(c) != state.player2_last_move)
        .max_by_key(|&c| player.expand(state.colors[c as usize]).count())
        .unwrap()
}
