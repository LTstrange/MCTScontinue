use super::utils::*;
use super::{LOSS, WIN};
use crate::gomoku::{Gomoku, Move, State};
use crate::interface::Game;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering::*};

pub struct Node {
    // The Move to get from the parent to here.
    // Only None at the root.
    pub m: Option<Move>,
    pub visits: AtomicU32,
    // +1 for wins, -1 for losses, +0 for draws.
    // From perspective of the player that made this move.
    pub score: AtomicI32,
    // Lazily populated if this node guarantees a particular end state.
    // WIN for a guaranteed win, LOSS for a guaranteed loss.
    // Not bothering with draws.
    pub winner: AtomicI32,
    // Lazily populated.
    pub expansion: AtomicBox<NodeExpansion>,
}
pub struct NodeExpansion {
    pub children: Vec<Node>,
}

pub fn new_expansion(state: &State) -> Box<NodeExpansion> {
    let mut moves = Vec::new();
    Gomoku::generate_moves(state, &mut moves);
    let children = moves
        .into_iter()
        .map(|m| Node::new(Some(m)))
        .collect::<Vec<_>>();
    Box::new(NodeExpansion { children })
}

impl Node {
    pub fn new(m: Option<Move>) -> Self {
        Node {
            m,
            expansion: AtomicBox::default(),
            visits: AtomicU32::new(0),
            score: AtomicI32::new(0),
            winner: AtomicI32::new(0),
        }
    }

    // Choose best child based on UCT.
    pub fn best_child(&self, exploration_score: f32) -> Option<&Node> {
        let mut log_visits = (self.visits.load(SeqCst) as f32).log2();
        // Keep this numerator non-negative.
        if log_visits < 0.0 {
            log_visits = 0.0;
        }

        let expansion = self.expansion.get()?;
        random_best(expansion.children.as_slice(), |node| {
            node.uct_score(exploration_score, log_visits)
        })
    }

    pub fn pre_update_stats(&self) {
        // Use a technicque called virtual loss to assume we've lost any
        // ongoing simulation to bias concurrent threads against exploring it.
        self.visits.fetch_add(1, SeqCst);
        self.score.fetch_add(-1, SeqCst);
    }

    pub fn update_stats(&self, result: i32) -> i32 {
        if result == WIN || result == LOSS {
            self.winner.store(result, SeqCst);
        } else {
            // Adjust for virtual loss.
            self.score.fetch_add(result + 1, SeqCst);
        }
        // Always return Some, as we aren't timed out.
        result
    }

    pub fn get_to_node(&self, moves: &[Move]) -> Option<&Node> {
        let mut res: &Node = self;
        for m in moves {
            let expansion = res.expansion.get()?;
            for n in expansion.children.iter() {
                if n.m? == *m {
                    res = &n;
                }
            }
        }
        Some(res)
    }

    pub fn propagate_reward(&self, reward: i32, moves: &[Move]) {
        let mut res: &Node = self;
        res.pre_update_stats();
        res.update_stats(reward);
        for m in moves {
            let expansion = res.expansion.get().unwrap();
            for n in expansion.children.iter() {
                if n.m.unwrap() == *m {
                    res = &n;
                }
            }
            res.pre_update_stats();
            res.update_stats(reward);
        }
    }

    fn uct_score(&self, exploration_score: f32, log_parent_visits: f32) -> f32 {
        let winner = self.winner.load(Relaxed);
        if winner < 0 {
            // Large enough to be returned from best_move, smaller than any other value.
            // This effectively ignores any moves that we've proved guarantee losses.
            // The MCTS-Solver paper says not to do this, but I don't buy their argument.
            // Those moves effectivey won't exist in our search, and we'll
            // have to see if the remaining moves make the parent moves worthwhile.
            return -1.0;
        }
        if winner > 0 {
            return f32::INFINITY;
        }
        let visits = self.visits.load(Relaxed) as f32;
        let score = self.score.load(Relaxed) as f32;
        if visits == 0.0 {
            // Avoid NaNs.
            return if exploration_score > 0.0 {
                f32::INFINITY
            } else {
                0.0
            };
        }
        let win_ratio = (score + visits) / (2.0 * visits);
        win_ratio + exploration_score * (2.0 * log_parent_visits / visits).sqrt()
    }
}
