# Connect Four
This project is intended to show multiple approaches to solving the game of Connect Four, with a variety of different algorithms and board implementations.
Much inspiration was taken from the well-loved guide on [Pascal Pons' Blog](http://blog.gamesolver.org/solving-connect-four/01-introduction/).

AI Use Statement:
All of the code in this project is my own work, written with my own hands.

## Usage 
Run `cargo run help` for usage patterns.

### Benching
Each bencher gives each solver a certain amount of time per testset and breaks if it was
not able to solve the entire testset in time. Displays the following information:
| [mean dur per position]ms [mean num explored positions]# [count succesful]/ |

Complete comparison: `cargo run -r bench`
  Runs several different combinations of approaches, grouped by their 
  capability, on increasingly difficult tesets. 
  Expect to take (<= 15 mins)

Solve: `cargo run -r bench solve`
  Runs the current best combination of solver and board (`Deepening<MinimaxQuick>` on `BitBoard` at the moment) on the hardest testset and then on an empty board i.e. attempt to completely solve the game.
  Expect to take (<= 45 mins)

### Play
todo!

## Todo

Todo:
- make testsets static
- check hash table statistical distribution
- add hashboard tests
