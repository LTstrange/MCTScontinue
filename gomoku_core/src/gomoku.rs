use super::interface::{Game, Winner};
use std::fmt::{Debug, Display};
pub struct Gomoku;

#[derive(Default, Clone)]
pub struct State {
    pub pieces: Vec<Move>,
}

impl State {
    pub fn new(moves: Vec<Move>) -> Self {
        Self { pieces: moves }
    }

    pub fn player_to_move(&self) -> Stone {
        if self.pieces.len() % 2 == 0 {
            Stone::Black
        } else {
            Stone::White
        }
    }

    pub fn player_just_moved(&self) -> Stone {
        if self.pieces.len() % 2 == 0 {
            Stone::White
        } else {
            Stone::Black
        }
    }

    fn is_full(&self) -> bool {
        self.pieces.len() == 225
    }

    #[inline]
    pub fn get_winner(&self) -> Option<Stone> {
        let (row, col) = self.pieces.last()?.get_coord();

        let same_pieces = self
            .pieces
            .iter()
            .enumerate()
            .filter_map(|(i, &m)| {
                if i % 2 != self.pieces.len() % 2 {
                    Some(m)
                } else {
                    None
                }
            })
            .collect::<Vec<Move>>();

        let mut board = [[None; 15]; 15];
        for m in same_pieces.iter() {
            let (row, col) = m.get_coord();
            board[row][col] = Some(Stone::Black);
        }

        let mut count;
        let mut max_count = 0;

        count = 0;
        // horizontal
        for i in 0..15 {
            if board[row][i].is_some() {
                count += 1;
            } else {
                max_count = max_count.max(count);
                count = 0;
            }
        }

        count = 0;
        // vertical
        #[allow(clippy::needless_range_loop)]
        for i in 0..15 {
            if board[i][col].is_some() {
                count += 1;
            } else {
                max_count = max_count.max(count);
                count = 0;
            }
        }

        count = 0;
        // diagonal \
        for (i, j) in (0..row)
            .rev()
            .zip((0..col).rev())
            .rev()
            .chain((row..15).zip(col..15))
        {
            if board[i][j].is_some() {
                count += 1;
            } else {
                max_count = max_count.max(count);
                count = 0;
            }
        }

        count = 0;
        // diagonal /
        for (i, j) in (0..row)
            .rev()
            .zip(col + 1..15)
            .rev()
            .chain((row..15).zip((0..=col).rev()))
        {
            if board[i][j].is_some() {
                count += 1;
            } else {
                max_count = max_count.max(count);
                count = 0;
            }
        }

        if max_count >= 5 {
            Some(self.player_just_moved())
        } else {
            None
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = [[None; 15]; 15];
        for (i, pieces) in self.pieces.iter().enumerate() {
            let (row, col) = pieces.get_coord();
            use Stone::*;
            let stone = if i % 2 == 0 { Black } else { White };
            board[row][col] = Some(stone);
        }
        writeln!(f, "   a b c d e f g h i j k l m n o")?;
        for (row, line) in board.iter().enumerate() {
            write!(f, "{:2} ", row)?;
            for stone in line {
                if let Some(stone) = stone {
                    write!(
                        f,
                        "{}",
                        match stone {
                            Stone::Black => "X ",
                            Stone::White => "O ",
                        }
                    )?;
                } else {
                    write!(f, "_ ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Stone {
    Black,
    White,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move(u8);
//   0   1   2  ...  12  13  14
//  15  16  17  ...  27  28  29
// ...
// 210 211 212  ... 222 223 224
impl Move {
    pub fn new(row: usize, col: usize) -> Self {
        let ind = col + row * 15;
        assert!(
            ind < 225,
            "coordinate out of bounds, should between 0..=224"
        );
        Move(ind as _)
    }
    pub fn get_coord(&self) -> (usize, usize) {
        // row, col
        (self.0 as usize / 15, self.0 as usize % 15)
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let coord = self.get_coord();
        let self_dis = coord.0.abs_diff(7) + coord.1.abs_diff(7);

        let coord = other.get_coord();
        let other_dis = coord.0.abs_diff(7) + coord.1.abs_diff(7);

        other_dis.cmp(&self_dis)
    }
}

#[test]
fn test_cmp() {
    assert_eq!(
        Move::new(8, 7).cmp(&Move::new(7, 8)),
        std::cmp::Ordering::Equal
    );
    assert_eq!(
        Move::new(7, 7).cmp(&Move::new(7, 8)),
        std::cmp::Ordering::Greater
    );
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0 / 15, self.0 % 15)
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "M({},{})", ((self.0 % 15) + b'a') as char, self.0 / 15)
    }
}

impl Game for Gomoku {
    type S = State;
    type M = Move;

    fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) {
        moves.clear();
        let mut pieces = [false; 225];
        for p in &state.pieces {
            pieces[p.0 as usize] = true;
        }
        // perfomance concern
        #[allow(clippy::needless_range_loop)]
        for i in 0..225 {
            if !pieces[i] {
                moves.push(Move(i as _));
            }
        }
    }

    fn apply(state: &mut Self::S, m: &Self::M) {
        state.pieces.push(*m);
    }

    fn undo(state: &mut Self::S, m: &Self::M) {
        let last_move = state.pieces.pop();
        assert_eq!(last_move, Some(*m));
    }

    fn get_winner(state: &Self::S) -> Option<Winner> {
        match (state.is_full(), state.get_winner()) {
            (_, Some(_)) => Some(Winner::PlayerJustMoved),
            (true, None) => Some(Winner::Draw),
            _ => None,
        }
    }
}
