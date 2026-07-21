use crate::basic::*;
use crate::board::*;

pub const MAX_MOVES: usize = column::COUNT * row::COUNT;
pub const MIN_SCORE: isize = -(MAX_MOVES as isize) / 2 + 3;
pub const MAX_SCORE: isize = (MAX_MOVES as isize + 1) / 2 - 3;

pub trait Position: Board {
    fn move_count(&self) -> usize;

    fn completed(&self) -> bool {
        self.move_count() == MAX_MOVES
    }

    /// Score if the next move is a win for the current
    /// player
    fn will_win_score(&self) -> isize {
        ((MAX_MOVES + 1 - self.move_count()) / 2) as isize
    }

    /// Score if the opponents next move will let
    /// them win.
    fn will_lose_score(&self) -> isize {
        -((MAX_MOVES - self.move_count()) as isize / 2)
    }

    /// Score if the position was just won on the
    /// previous move
    fn just_won_score(&self) -> isize {
        ((MAX_MOVES + 2 - self.move_count()) / 2) as isize
    }

    fn curr(&self) -> Token;

    fn opp(&self) -> Token {
        !self.curr()
    }

    fn place_curr(&mut self, col: column::Idx) -> Option<Cell> {
        self.place(col, self.curr())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WithInfo<B> {
    board: B,
    curr: Token,
    move_count: usize,
}
impl<B: Board> Board for WithInfo<B> {
    const EMPTY: Self = WithInfo {
        board: B::EMPTY,
        curr: Token::STARTING,
        move_count: 0,
    };

    fn get(&self, cell: Cell) -> Option<Token> {
        self.board.get(cell)
    }

    fn can_place(&self, col: column::Idx) -> bool {
        self.board.can_place(col)
    }

    fn force_place(&mut self, col: column::Idx, token: Token) {
        debug_assert!(self.can_place(col));
        self.board.force_place(col, token);
        self.curr = self.curr.next();
        self.move_count += 1;
    }
}

impl<B: CloneBoard> CloneBoard for WithInfo<B> {}

impl<B: MutBoard> MutBoard for WithInfo<B> {
    fn unplace(&mut self, cell: Cell) {
        self.board.unplace(cell);
        self.curr = self.curr.prev();
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
    fn curr(&self) -> Token {
        self.curr
    }
}
