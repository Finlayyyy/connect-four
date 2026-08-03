use std::io::stdin;
use std::marker::PhantomData;
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::*;
use crate::solver_utils::*;
use crate::solvers::Solver;

pub trait Player<P: Position> {
    fn name(&self) -> &str;
    fn play(&mut self, pos: &P) -> column::Idx;
}

#[derive(Debug, Clone)]
pub struct Human {
    name: String,
}
impl Human {
    pub fn new() -> Self {
        let mut input = String::new();
        println!("What is your name?");
        stdin().read_line(&mut input).expect("Failed to read line");
        println!();
        Human {
            name: input.trim().to_owned(),
        }
    }

    fn get_move<P: Position>(pos: &P) -> column::Idx {
        loop {
            let mut input = String::new();
            println!("Input your move as a column index 1 to 7");
            stdin().read_line(&mut input).expect("Failed to read line");
            input = input.trim().to_owned();

            let mut chars = input.trim().chars();
            let Some(ch) = chars.next() else {
                println!("Expected column index, found \"{input}\"");
                continue;
            };
            if chars.next().is_some() {
                println!("Expected a single digit, found \"{input}\"");
                continue;
            }

            match column::Idx::from_digit(ch) {
                Ok(col) if pos.can_place(col) => return col,
                Ok(col) => println!("You cannot place in {col} as it is full."),
                Err(err) => println!("{err}"),
            }
        }
    }
}

impl<P: Position> Player<P> for Human {
    fn name(&self) -> &str {
        &self.name
    }
    fn play(&mut self, pos: &P) -> column::Idx {
        println!(
            "Your turn [move_count: {}, curr: {}]:",
            pos.move_count(),
            pos.curr()
        );
        pos.display();
        Self::get_move(pos)
    }
}

pub struct Minimax<P, S> {
    cache: Cache<P>,
    pd: PhantomData<S>,
}

impl<P: Position, S: Solver<P>> Minimax<P, S> {
    pub fn new() -> Self {
        println!("Beginning solver precomputation. Expected duration: 2 minutes");
        let pos = P::EMPTY;
        let mut boss = LaissezFaire {};
        let mut cache = Cache::new_large();
        let _ = S::solve(pos, &mut boss, &mut cache); // initialise cache
        println!("Initialisation complete.");
        println!();
        Minimax {
            cache,
            pd: PhantomData,
        }
    }
}

impl<P: Position + CloneBoard, S: Solver<P>> Player<P> for Minimax<P, S> {
    fn name(&self) -> &str {
        "Minimax"
    }

    fn play(&mut self, pos: &P) -> column::Idx {
        let mut boss = LaissezFaire {};
        println!(
            "{} <= eval <= {}",
            pos.will_lose_eval(),
            pos.will_win_eval()
        );
        print!("col :   1   2   3   4   5   6   7");

        println!();
        let mut evals = [None; 7];

        let mut best = isize::MIN;
        let mut best_col = column::Idx::LEFT;
        for (col, next) in pos.nexts(pos.curr()) {
            let eval = if next.is_won_at_col(col) {
                next.just_won_eval()
            } else {
                let ControlFlow::Continue(opp_eval) = S::solve(next, &mut boss, &mut self.cache);
                -opp_eval
            };

            evals[usize::from(col)] = Some(eval);
            if eval > best {
                best = eval;
                best_col = col;
            }
        }
        print!("eval:");
        for eval in evals {
            match eval {
                None => print!("   -"),
                Some(eval) => print!("{:4}", eval),
            }
        }
        println!();
        println!();
        best_col
    }
}
