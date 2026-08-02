use crate::basic::{Cell, Token, column, row};
use crate::board::{Board, MutBoard};

macro_rules! make_test {
    ($b:ty, $mod:ident, $func:ident) => {
        #[test]
        fn $func() {
            crate::board::testing::$mod::$func::<$b>(stringify!($b));
        }
    };
}

macro_rules! make_board_tests {
    ($b:ty) => {
        make_test!($b, board_tests, empty_is_empty);
        make_test!($b, board_tests, cannot_place_in_full_column);
        make_test!($b, board_tests, won_at_basic);
        make_test!($b, board_tests, next_cells);
    };
}

macro_rules! make_mut_board_tests {
    ($b:ty) => {
        make_test!($b, mut_board_tests, place_unplace_eq);
    };
}

macro_rules! make_hash_board_tests {
    ($b:ty) => {
        make_test!($b, hash_board_tests, key_49_bits);
        make_test!($b, hash_board_tests, key_unique);
    }
}

pub mod board_tests {
    use super::*;
    use crate::board::Moves;
    use crate::bench::END_EASY;

    pub fn empty_is_empty<B: Board>(name: &str) {
        let empty = B::EMPTY;
        for col in column::COLUMNS {
            for row in row::BOTTOM_UP {
                assert!(
                    empty.get(Cell { col, row }).is_none(),
                    "`{name}::EMPTY` is not empty at ({col}, {row})."
                );
            }
        }
    }

    pub fn cannot_place_in_full_column<B: Board>(name: &str) {
        let mut board = B::EMPTY;
        let mut curr = Token::STARTING;

        for col in column::COLUMNS {
            for _ in 0..6 {
                assert!(
                    board.can_place(col),
                    "`{name}::can_place` returned false on a non-full column."
                );
                assert_eq!(
                    curr,
                    board.calc_curr(),
                    "`{name}::curr_player` returned incorrect."
                );
                let cell = board.place(col, curr);
                assert!(
                    cell.is_some(),
                    "`{name}::place` returned `None` even though `can_place` is true"
                );
                curr = curr.next();
            }

            assert!(
                !board.can_place(col),
                "`{name}::can_place returned true on a full column."
            );
        }
    }

    pub fn won_at_basic<B: Board>(name: &str) {
        let moves = "1122334";
        let moves = Moves::from_string(moves);
        let board = B::from_moves(&moves);

        assert!(
            board.is_won_at_col(column::Idx::CENTRE),
            "`{name}::won_at returned false on a winning cell."
        );
    }

    pub fn next_cells<B: Board>(name: &str) {
        let mut curr = Token::STARTING;
        for (moves, _) in &*END_EASY {
            let mut b = B::from_moves(moves);
            let nexts = b.next_cells().collect::<Vec<_>>();
            for cell in nexts {
                let count_prev = b.col_count(cell.col);
                let placed = b.place(cell.col, curr);
                let count_post = b.col_count(cell.col);
                assert_eq!(
                    count_prev + 1,
                    count_post,
                    "{name}::col_count should increment after place @ {}.",
                    cell
                );
                assert_eq!(
                    Some(cell),
                    placed,
                    "`{name}::next_cells` predicted incorrectly."
                );

                curr = curr.next();
            }
        }
    }
}

pub mod mut_board_tests {
    use super::*;

    pub fn place_unplace_eq<B: Clone + MutBoard>(name: &str) {
        let mut board = B::EMPTY;
        let mut token = Token::STARTING;
        for row in row::BOTTOM_UP {
            for col in column::COLUMNS {
                let temp = board.clone();

                let Some(cell) = board.place(col, token) else {
                    panic!(
                        "`{name}::place returned None on a non-full column @ cell={}.",
                        Cell { row, col }
                    );
                };
                board.unplace(col);
                assert_eq!(
                    temp,
                    board,
                    "`{name}::unplace∘{name}::place` != id @ cell={cell}."
                );
                board.place(col, token);
                token = token.next();
            }
        }
    }
}

pub mod hash_board_tests {
    use crate::bench::{BEGIN_EASY, END_EASY};
    use crate::board::HashBoard;
    use std::collections::HashMap;

    pub fn key_49_bits<B: HashBoard>(name: &str) {
        const MASK_49: u64 = (1 << 49) - 1;
        for (moves, _) in END_EASY.iter().chain(BEGIN_EASY.iter()) {
            let board = B::from_moves(moves);
            assert!(board.key() & (!MASK_49) == 0,
                "`{name}::key` returned a key that used more than 49 bits.");
            assert!(board.key() != 0,
                "`{name}::key` returned zero");
        }

    }

    pub fn key_unique<B: HashBoard>(name: &str) {
        let mut keys = HashMap::new();
        for (moves, _) in END_EASY.iter().chain(BEGIN_EASY.iter()) {
            let board = B::from_moves(moves);
            if let Some(other) = keys.get(&board.key()) {
                assert!(board == *other, "`{name}::key` is not unique.");
            }
            keys.insert(board.key(), board);
        }
    }
}
