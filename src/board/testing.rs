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

pub mod board_tests {
    use super::*;
    use crate::board::Moves;

    pub fn empty_is_empty<B: Board>(name: &str) {
        let empty = B::EMPTY;
        for col in column::LEFT..=column::RIGHT {
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

        for col in column::LEFT..=column::RIGHT {
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
        let board = B::from_display(
            "|       |
             |       |
             |       |
             |RRR    |
             |YYYY   |",
        );
        assert!(
            board.is_won_at_col(column::CENTRE),
            "`{name}::won_at returned false on a winning cell."
        );
    }

    pub fn next_cells<B: Board>(name: &str) {
        let mut curr = Token::STARTING;
        for _ in 0..100 {
            for len in 0..42 {
                let mut b = B::from_moves(&Moves::random(len));
                let nexts = b.next_cells().collect::<Vec<_>>();
                for cell in nexts {
                    let count_prev = b.col_count(cell.col);
                    let placed = b.place(cell.col, curr);
                    let count_post = b.col_count(cell.col);
                    assert_eq!(count_prev + 1, count_post, "{name}::col_count should increment after place @ {}.", cell);
                    assert_eq!(Some(cell), placed ,"`{name}::next_cells` predicted incorrectly.");
                    
                }
                curr = curr.next();
            }
            
        }
    }
}

pub mod mut_board_tests {
    use crate::board::Moves;
    use super::*;

    pub fn place_unplace_eq<B: Clone + MutBoard>(name: &str) {
        let mut board = B::EMPTY;
        let mut token = Token::STARTING;
        for row in row::BOTTOM_UP {
            for col in column::LEFT..=column::RIGHT {
                let temp = board.clone();

                let Some(cell) = board.place(col, token) else {
                    panic!("`{name}::place returned None on a non-full column @ cell={}.", Cell { row, col });
                };
                board.unplace(cell);
                assert_eq!(temp, board, "`{name}::unplace∘{name}::place` != id @ cell={}.", Cell { row, col });
                board.place(col, token);
                token = token.next();
            }
        }
    }

    pub fn from_moves_eq_to_moves<B: Clone + MutBoard>(name: &str) {
        for _ in 0..100 {
            for len in 0..42 {
                let mut b = B::from_moves(&Moves::random(len));
                assert_eq!(b, B::from_moves(&b.clone().to_moves()), "`{name}::from_moves∘`{name}::to_moves != id");

            }
            
        }
    }
}
