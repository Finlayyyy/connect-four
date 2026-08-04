# Connect Four
This project is intended to show multiple approaches to (strongly) solving the game of Connect Four, with a variety of different algorithms and board implementations. The objective was to be able to solve the game in a reasonable amount of time, without using a pre-made book. Currently that time sits at less than 1m30. Much inspiration was taken from the well-loved guide on [Pascal Pons' Blog](http://blog.gamesolver.org/solving-connect-four/01-introduction/).

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
  Expect to take <= 90 mins in total (≈ 1000 five-second testcases !)

### Play
Play against the current best combination of solver and board (`Deepening<MinimaxQuick>` on `BitBoard` at the moment). At each step, the solver will display the range of evaluations it considers (from it immediately losing to it having immediately winning move). It will then display the evaluation it has calculated for each possible move (and then choose the best one !). 

`cargo run -r play first`: the solver will take the first move.

`cargo run -r play second`: the solver will take the second move.


## Todo
- cleanup

## See also
[Pascal Pons' Solver](https://github.com/PascalPons/connect4)
[Ben Rall's Solver](https://github.com/benjaminrall/connect-four-ai/tree/main)

### Benchmark

| Testset             | \[av. time\]ms \[av. pos\]# \[num pos\] |
| ------------------- | --------------------------------------- |
| END_EASY      /1000 | 0ms 5.21e1# 1000/    |
| MIDDLE_EASY   /1000 | 0ms 4.53e2# 1000/    |
| MIDDLE_MEDIUM /1000 | 3ms 3.98e4# 1000/    |
| BEGIN_EASY    /1000 | 0ms 3.69e3# 1000/    |
| BEGIN_MEDIUM  /1000 | 90ms 1.12e6# 1000/   |
| SOLVE         /1    | 77654ms 9.45e8#  1/  |
| BEGIN_HARD    /100  | 3282ms 4.01e7#  100/ |
| BEGIN_HARD    /1000 | 638ms 2.02e7# 1000/  |

