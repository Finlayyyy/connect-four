use crate::basic::*;
use crate::board::*;

/// Maximum possible number of moves in a game
pub const MAX_MOVES: usize = column::COUNT * row::COUNT;
/// Minimum possible position score
pub const MIN_SCORE: isize = -(MAX_MOVES as isize) / 2 + 3;
/// Maximum possible position score
pub const MAX_SCORE: isize = (MAX_MOVES as isize + 1) / 2 - 3;

pub trait Position: Board {
    /// Number of moves made in the game
    fn move_count(&self) -> usize;

    /// Has the game run out of empty cells?
    #[inline(always)]
    fn completed(&self) -> bool {
        self.move_count() == MAX_MOVES
    }

    /// Score if the next move is a win for the current
    /// player
    fn will_win_score(&self) -> isize {
        ((MAX_MOVES + 1 - self.move_count()) / 2) as isize
    }

    /// Score if the opponent will win on their next move
    fn will_lose_score(&self) -> isize {
        -((MAX_MOVES - self.move_count()) as isize / 2)
    }

    /// Score if the position was just won on the
    /// previous move
    fn just_won_score(&self) -> isize {
        ((MAX_MOVES + 2 - self.move_count()) / 2) as isize
    }

    /// Current player's token
    fn curr(&self) -> Token {
        match self.move_count() % 2 {
            0 => Token::STARTING,
            1 => Token::SECOND,
            _ => unreachable!()
        }
    }

    /// Opponent's token
    fn opp(&self) -> Token {
        !self.curr()
    }

    /// Place the current player's token in the given column
    fn place_curr(&mut self, col: column::Idx) -> Option<Cell> {
        self.place(col, self.curr())
    }
}

/// A wrapper around a board that tracks
/// the number of moves made so far for quicker
/// computation
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WithInfo<B> {
    board: B,
    move_count: usize,
}

impl<B: Board> Board for WithInfo<B> {
    const EMPTY: Self = WithInfo {
        board: B::EMPTY,
        move_count: 0,
    };

    fn count_moves(&self) -> usize {
        self.move_count
    }

    fn calc_curr(&self) -> Token {
        self.curr()
    }

    fn get(&self, cell: Cell) -> Option<Token> {
        self.board.get(cell)
    }

    fn can_place(&self, col: column::Idx) -> bool {
        self.board.can_place(col)
    }

    fn force_place(&mut self, col: column::Idx, token: Token) {
        debug_assert!(self.can_place(col));
        self.board.force_place(col, token);
        self.move_count += 1;
    }
}

impl<B: CloneBoard> CloneBoard for WithInfo<B> {}

impl<B: MutBoard> MutBoard for WithInfo<B> {
    fn unplace(&mut self, col: column::Idx) {
        self.board.unplace(col);
        self.move_count -= 1;
    }
}

impl<B: HashBoard> HashBoard for WithInfo<B> {
    fn key(&self) -> u64 {
        self.board.key()
    }
}

impl<B: Board> Position for WithInfo<B> {
    fn move_count(&self) -> usize {
        self.move_count
    }
}
