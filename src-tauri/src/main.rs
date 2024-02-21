#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;

use gomoku_core::{
    interface::Game, Gomoku, MCTSOptions, MonteCarloTreeSearchContinue, Move, State,
};

struct GameState {
    state: Mutex<State>,
    solver: Mutex<MonteCarloTreeSearchContinue>,
}

fn main() {
    let option = MCTSOptions::default()
        .with_max_rollout_depth(225)
        .with_rollouts_before_expanding(20)
        .with_num_threads(10)
        .verbose();
    let mut solver = MonteCarloTreeSearchContinue::new(option);
    solver.start_simulating();

    tauri::Builder::default()
        .manage(GameState {
            state: Mutex::new(State::default()),
            solver: Mutex::new(solver),
        })
        .invoke_handler(tauri::generate_handler![
            init_game, click, undo, step, restart
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn init_game(state: tauri::State<GameState>) {
    // println!("Init game");
    *state.state.lock().unwrap() = State::default();
}

#[tauri::command]
fn click(x: usize, y: usize, state: tauri::State<GameState>) -> Vec<u8> {
    let m = Move::new(y, x);
    let solver = state.solver.lock().unwrap();
    let mut state = state.state.lock().unwrap();
    if !state.pieces.contains(&m) {
        Gomoku::apply(&mut state, &m);
        solver.change_cur_state(&state);
    }

    state.pieces.iter().map(|m| m.0).collect()
}

#[tauri::command]
fn undo(state: tauri::State<GameState>) -> Vec<u8> {
    let solver = state.solver.lock().unwrap();
    let mut state = state.state.lock().unwrap();
    state.pieces.pop();
    solver.change_cur_state(&state);

    state.pieces.iter().map(|m| m.0).collect()
}

#[tauri::command]
fn step(state: tauri::State<GameState>) -> Vec<u8> {
    let solver = state.solver.lock().unwrap();
    let mut state = state.state.lock().unwrap();
    println!("Choosing move");
    let m = solver.choose_move(&state).unwrap();
    Gomoku::apply(&mut state, &m);
    solver.change_cur_state(&state);

    state.pieces.iter().map(|m| m.0).collect()
}

#[tauri::command]
fn restart(state: tauri::State<GameState>) -> Vec<u8> {
    let solver = state.solver.lock().unwrap();
    let mut state = state.state.lock().unwrap();
    state.pieces.clear();
    solver.change_cur_state(&state);

    state.pieces.iter().map(|m| m.0).collect()
}
