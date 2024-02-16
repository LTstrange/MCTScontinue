use std::collections::BinaryHeap;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use gomoku_core::{
    gomoku::{Gomoku, Move, State},
    interface::Game,
};

use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

fn build_a_board(num_pieces: u8) -> State {
    let mut state = State::default();
    let mut rng = thread_rng();
    for _ in 0..num_pieces {
        let mut moves = Vec::new();
        Gomoku::generate_moves(&state, &mut moves);
        let m = moves.choose(&mut rng).unwrap();
        Gomoku::apply(&mut state, m);
    }
    state
}

pub fn criterion_get_winner(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_winner");
    for num_pieces in (0..=225).step_by(25) {
        let test_state = build_a_board(num_pieces);
        // group.bench_with_input(
        //     BenchmarkId::new("naive", num_pieces),
        //     &test_state,
        //     |b, i| b.iter(|| i.get_winner_naive()),
        // );
        group.bench_with_input(
            BenchmarkId::new("board", num_pieces),
            &test_state,
            |b, i| b.iter(|| i.get_winner()),
        );
    }
    group.finish();
}

fn rand_move_heap(state: &State, move_scratch: &mut Vec<Move>) -> Move {
    Gomoku::generate_moves(state, move_scratch);
    let mut heap = BinaryHeap::from_iter(move_scratch);
    *heap.pop().unwrap()
}

#[allow(unused)]
fn rand_move_sort(state: &State, move_scratch: &mut Vec<Move>) -> Move {
    Gomoku::generate_moves(state, move_scratch);
    move_scratch.sort_unstable_by_key(|m| {
        let coord = m.get_coord();
        coord.0.abs_diff(7) + coord.1.abs_diff(7)
    });
    move_scratch.swap_remove(0)
}

fn rand_move_naive(state: &State, move_scratch: &mut Vec<Move>, rng: &mut ThreadRng) -> Move {
    Gomoku::generate_moves(state, move_scratch);
    move_scratch.swap_remove(rng.gen_range(0..move_scratch.len()))
}

fn rand_move_weighted(state: &State, move_scratch: &mut Vec<Move>, rng: &mut ThreadRng) -> Move {
    Gomoku::generate_moves(state, move_scratch);
    *move_scratch
        .choose_weighted(rng, |m| {
            let coord = m.get_coord();
            14 - (coord.0.abs_diff(7) + coord.1.abs_diff(7))
        })
        .unwrap()
}

pub fn criterion_rand_move(c: &mut Criterion) {
    let mut group = c.benchmark_group("rand_move");
    for num_pieces in (0..=225).step_by(50) {
        let test_state = build_a_board(num_pieces);

        group.bench_function(BenchmarkId::new("heap", num_pieces), |b| {
            b.iter(|| rand_move_heap(&test_state, &mut Vec::new()))
        });
        // too slow
        // group.bench_function(BenchmarkId::new("sort", num_pieces), |b| {
        //     b.iter(|| rand_move_sort(&test_state, &mut Vec::new()))
        // });
        group.bench_function(BenchmarkId::new("naive", num_pieces), |b| {
            b.iter(|| rand_move_naive(&test_state, &mut Vec::new(), &mut thread_rng()))
        });
        group.bench_function(BenchmarkId::new("weighted", num_pieces), |b| {
            b.iter(|| rand_move_weighted(&test_state, &mut Vec::new(), &mut thread_rng()))
        });
    }
    group.finish();
}

pub fn criterion_gen_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_moves");
    for num_pieces in (0..=225).step_by(25) {
        let test_state = build_a_board(num_pieces);
        group.bench_with_input(
            BenchmarkId::new("board", num_pieces),
            &test_state,
            |b, i| b.iter(|| Gomoku::generate_moves(i, &mut Vec::new())),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    criterion_rand_move,
    criterion_gen_moves,
    criterion_get_winner
);
criterion_main!(benches);
