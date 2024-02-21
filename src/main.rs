use std::io::Write;

use gomoku_core::{
    interface::Game, Gomoku, MCTSOptions, MonteCarloTreeSearchContinue, Move, State,
};

fn human_play(game_state: &mut State) -> Move {
    // Human
    let mut input = String::new();
    print!("enter a move (e.g. \"h 7\"): ");
    std::io::stdout().flush().unwrap();
    loop {
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match input.trim().split_once(' ').map(|(c, r)| {
            (
                r.trim().parse::<usize>().ok(),
                c.trim()
                    .to_lowercase()
                    .bytes()
                    .next()
                    .map(|c| (c - b'a') as usize),
            )
        }) {
            Some((Some(row), Some(col))) if !game_state.pieces.contains(&Move::new(row, col)) => {
                return Move::new(row, col);
            }
            _ => {
                println!("error input, re enter:");
                input.clear();
                continue;
            }
        }
    }
}

fn main() {
    let mut game_state = State::new(vec![]);
    let option = MCTSOptions::default()
        .with_max_rollout_depth(225)
        .with_rollouts_before_expanding(10)
        .verbose();
    let mut strategy = MonteCarloTreeSearchContinue::new(option);
    strategy.start_simulating();

    println!("{}", game_state);
    let _ = loop {
        // AI
        let best_move = strategy.choose_move(&game_state).unwrap();
        println!("best move: {}", best_move);
        Gomoku::apply(&mut game_state, &best_move);
        strategy.change_cur_state(&game_state);
        println!("{}", game_state);
        if Gomoku::get_winner(&game_state).is_some() {
            break Gomoku::get_winner(&game_state);
        }
        // human
        let player_move = human_play(&mut game_state);
        Gomoku::apply(&mut game_state, &player_move);
        println!("{}", game_state);
        if Gomoku::get_winner(&game_state).is_some() {
            break Gomoku::get_winner(&game_state);
        }
    }
    .unwrap();

    println!("Player{:?} Win the game!", game_state.player_just_moved());
    println!("moves:{:?}", game_state.pieces);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_winner() {
        let test_state = State::new(vec![
            Move::new(5, 12),
            Move::new(7, 7),
            Move::new(3, 10),
            Move::new(7, 8),
            Move::new(4, 13),
            Move::new(7, 9),
            Move::new(6, 11),
            Move::new(7, 10),
            Move::new(3, 14),
        ]);
        println!("{}", test_state);
        assert_eq!(test_state.get_winner(), None);
    }
}
