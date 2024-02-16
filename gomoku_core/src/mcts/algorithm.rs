use std::sync::{atomic::Ordering::*, Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use super::rollout_policy::RolloutPolicy;
use super::{LOSS, WIN};

use crate::gomoku::{Gomoku, Move, State};
use crate::interface::{Game, Winner};

use super::node::{new_expansion, Node};

/// Options for MonteCarloTreeSearch.
#[derive(Clone)]
pub struct MCTSOptions {
    verbose: bool,
    pub(super) max_rollout_depth: u32,
    rollouts_before_expanding: u32,
    // None means use num_cpus.
    num_threads: Option<usize>,
}

impl Default for MCTSOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            max_rollout_depth: 100,
            rollouts_before_expanding: 5,
            num_threads: None,
        }
    }
}

impl MCTSOptions {
    /// Enable verbose print statements after each search.
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Set a maximum depth for rollouts. Rollouts that reach this depth are
    /// stopped and assigned a Draw value.
    pub fn with_max_rollout_depth(mut self, depth: u32) -> Self {
        self.max_rollout_depth = depth;
        self
    }

    /// How many rollouts to run on a single leaf node before expanding its
    /// children. The default value is 0, where every rollout expands some
    /// leaf node.
    pub fn with_rollouts_before_expanding(mut self, rollouts: u32) -> Self {
        self.rollouts_before_expanding = rollouts;
        self
    }

    /// How many threads to run. Defaults to num_cpus.
    pub fn with_num_threads(mut self, threads: usize) -> Self {
        self.num_threads = Some(threads);
        self
    }
}

pub struct MonteCarloTreeSearchContinue {
    tree: Arc<Node>,
    cur_state: Arc<RwLock<State>>,
    options: MCTSOptions,
    time_out: Duration,
    pre_rollouts_count: u32,
    pre_choose_move_time: Instant,
}

impl MonteCarloTreeSearchContinue {
    pub fn new(options: MCTSOptions) -> Self {
        let cur_state = State::default();
        let tree = Node::new(None);
        tree.expansion.try_set(new_expansion(&cur_state));

        Self {
            tree: Arc::new(tree),
            cur_state: Arc::new(RwLock::new(cur_state)),
            options,
            time_out: Duration::from_secs(5),
            pre_rollouts_count: 0,
            pre_choose_move_time: Instant::now(),
        }
    }
    pub fn choose_move(&mut self, state: &State) -> Option<Move> {
        self.change_cur_state(state);
        if self.cur_state.read().unwrap().pieces.is_empty() {
            return Some(Move::new(7, 7));
        }

        thread::sleep(self.time_out);

        let cur_node = self
            .tree
            .get_to_node(&self.cur_state.read().unwrap().pieces)
            .expect("cur_node and cur_state not match!");

        if self.options.verbose {
            let total_visits = self.tree.visits.load(Relaxed);
            let duration = Instant::now().duration_since(self.pre_choose_move_time);
            let rate = (total_visits - self.pre_rollouts_count) as f64 / duration.as_secs_f64();
            eprintln!(
                "Did {} total simulations with {:.1} rollouts/sec",
                total_visits, rate
            );
            // Sort moves by visit count, largest first.
            let mut children = cur_node
                .expansion
                .get()?
                .children
                .iter()
                .map(|node| (node.visits.load(Relaxed), node.score.load(Relaxed), node.m))
                .collect::<Vec<_>>();
            children.sort_by_key(|t| !t.0);

            // Dump stats about the top 10 nodes.
            for (visits, score, m) in children.into_iter().take(10) {
                // Normalized so all wins is 100%, all draws is 50%, and all losses is 0%.
                let win_rate = (score as f64 + visits as f64) / (visits as f64 * 2.0);
                println!(
                    "{:>6} visits, {:.02}% wins: {}",
                    visits,
                    win_rate * 100.0,
                    m.unwrap()
                );
            }
        }

        // get most visited node
        let node = cur_node
            .expansion
            .get()?
            .children
            .iter()
            .max_by_key(|n| n.visits.load(Relaxed))?;
        println!("final visits: {}", node.visits.load(Relaxed));
        node.m
    }

    pub fn change_cur_state(&self, state: &State) {
        let mut cur_state = self.cur_state.write().unwrap();
        *cur_state = state.clone();
    }

    pub fn start_simulating(&mut self) {
        self.pre_choose_move_time = Instant::now();
        let num_threads = self.options.num_threads.unwrap_or_else(num_cpus::get) as u32;

        for _ in 0..num_threads {
            let state = Arc::clone(&self.cur_state);
            let tree = Arc::clone(&self.tree);
            let options = self.options.clone();
            thread::spawn(move || loop {
                let mut state = { state.read().unwrap().clone() };
                let node = tree
                    .get_to_node(&state.pieces)
                    .expect("cur_node and cur_state not match!");
                let reward = Self::simulate_once(&options, node, &mut state, false);
                tree.propagate_reward(reward, &state.pieces);
            });
        }
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.time_out = timeout;
    }

    fn rollout(options: &MCTSOptions, state: &State) -> i32 {
        RolloutPolicy::rollout(options, state)
    }

    fn simulate_once(
        options: &MCTSOptions,
        node: &Node,
        state: &mut State,
        mut force_rollout: bool,
    ) -> i32 {
        let winner = node.winner.load(Relaxed);
        if winner != 0 {
            return winner;
        }
        node.pre_update_stats();

        if force_rollout {
            return node.update_stats(Self::rollout(options, state));
        }

        let expansion = match node.expansion.get() {
            Some(expansion) => expansion,
            None => {
                // This is a leaf node.
                if node.visits.load(SeqCst) <= options.rollouts_before_expanding {
                    // Just rollout from here.
                    return node.update_stats(Self::rollout(options, state));
                }
                // Check for terminal node.
                match Gomoku::get_winner(state) {
                    Some(Winner::PlayerJustMoved) => return node.update_stats(WIN),
                    Some(Winner::PlayerToMove) => return node.update_stats(LOSS),
                    Some(Winner::Draw) => return node.update_stats(0),
                    _ => {}
                }
                // Expand this node, and force a rollout when we recurse.
                force_rollout = true;
                node.expansion.try_set(new_expansion(state))
            }
        };

        // Recurse.
        let next = match node.best_child(1.) {
            Some(child) => child,
            // TODO: Weird race condition?
            None => return 0,
        };
        let m = next.m.as_ref().unwrap();
        Gomoku::apply(state, m);
        let child_result = Self::simulate_once(options, next, state, force_rollout);
        Gomoku::undo(state, m);

        // Propagate up forced wins and losses.
        let result = if child_result == WIN {
            // Having a guaranteed win child makes you a loser parent.
            LOSS
        } else if child_result == LOSS {
            // Having all guaranteed loser children makes you a winner parent.
            if expansion
                .children
                .iter()
                .all(|node| node.winner.load(Relaxed) == LOSS)
            {
                WIN
            } else {
                -1
            }
        } else {
            -child_result
        };

        // Backpropagate.
        node.update_stats(result)
    }
}
