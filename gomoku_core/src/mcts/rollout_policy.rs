use rand::{rngs::ThreadRng, seq::SliceRandom};

use crate::{
    gomoku::{Gomoku, Move, State},
    interface::{Game, Winner},
};

use super::{algorithm::MCTSOptions, LOSS, WIN};

pub struct RolloutPolicy;

/// Advanced random rollout policy for Monte Carlo Tree Search.
impl RolloutPolicy {
    /// Custom function to choose random move during rollouts.
    /// Implementations can bias towards certain moves, ensure winning moves, etc.
    /// The provided move vec is for scratch space.
    fn random_move(state: &mut State, move_scratch: &mut Vec<Move>, rng: &mut ThreadRng) -> Move {
        Gomoku::generate_moves(state, move_scratch);
        *move_scratch
            .choose_weighted(rng, |m| {
                let coord = m.get_coord();
                15 - (coord.0.abs_diff(7) + coord.1.abs_diff(7))
            })
            .unwrap()
    }

    /// Implementation of a rollout over many random moves. Not needed to be overridden.
    pub fn rollout(options: &MCTSOptions, state: &State) -> i32 {
        let mut rng = rand::thread_rng();
        let mut depth = options.max_rollout_depth;
        let mut state = state.clone();
        let mut moves = Vec::new();
        let mut sign = 1;
        loop {
            if let Some(winner) = Gomoku::get_winner(&state) {
                let first = depth == options.max_rollout_depth;
                return match winner {
                    Winner::PlayerJustMoved => {
                        if first {
                            WIN
                        } else {
                            1
                        }
                    }
                    Winner::PlayerToMove => {
                        if first {
                            LOSS
                        } else {
                            -1
                        }
                    }
                    Winner::Draw => 0,
                } * sign;
            }

            if depth == 0 {
                return 0;
            }

            moves.clear();
            let m = Self::random_move(&mut state, &mut moves, &mut rng);
            Gomoku::apply(&mut state, &m);
            sign = -sign;
            depth -= 1;
        }
    }
}
