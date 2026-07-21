use std::ops::ControlFlow;
use std::hash::Hash;

use crate::basic::*;
use crate::board::*;

#[macro_use]
mod testing;

pub mod minimax_basic;
pub mod minimax_alphabeta;
pub mod minimax_cached;
pub mod minimax_ab_cached;
pub mod minimax_symm;
pub mod minimax_ordered;
pub mod minimax_avoidant;
pub mod minimax_deepening;
pub mod minimax_quick_avoid;

pub use minimax_alphabeta::minimax_alphabeta;
pub use minimax_basic::minimax_clone;
pub use minimax_basic::minimax_mut;
pub use minimax_cached::minimax_cached;
pub use minimax_ab_cached::minimax_ab_cached;
pub use minimax_symm::minimax_symm;
pub use minimax_ordered::minimax_ordered;
pub use minimax_avoidant::minimax_avoidant;
pub use minimax_quick_avoid::minimax_quick_avoid;
pub use minimax_deepening::minimax_deepening;

use crate::solver_utils::solver_manager::*;

