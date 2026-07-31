# Connect Four
This project is intended to show multiple approaches to (strongly) solving the game of Connect Four, with a variety of different algorithms and board implementations.
Much inspiration was taken from the well-loved guide on [Pascal Pons' Blog](http://blog.gamesolver.org/solving-connect-four/01-introduction/).

AI Use Statement:
All of the code in this project is my own work; written with my own hands.

## Usage 
Run `cargo run help` for usage patterns.

### Benching
Each bencher gives each solver a certain amount of time per testset and moves on if the solver was
unable to solve the entire testset in time. Displays the following information:

`| [mean dur per position]ms [mean num explored positions]# [count successful]/ |`

Complete comparison: `cargo run -r bench`

  Runs several different combinations of approaches, grouped by their 
  estimated capability, on increasingly difficult testsets. 
  Expect to take <= 15 mins.

Solve: `cargo run -r bench solve`

  Runs the current best combination of solver and board (`Deepening<MinimaxQuick>` on `BitBoard` at the moment) on all testsets and then on an empty board i.e. attempt to completely solve the game.
  Expect to take <= 90 mins (≈ 1000 five-second testcases !)

### Play
todo!

## Todo
- check hash table statistical distribution
- add hashboard tests
- order cache check in MinimaxQuick
- add Hasher to Cache (+ depth comparison)
