# Checkers Redux

There are many features and optimizations that can make the Minimax algorithm faster at identifying valid moves and more successful at finding winning moves. The purpose of this project is to explore some of those features and optimizations in order to understand their impact on a Checkers engine.

## Program Requirements

* Rust
* Python 3.11+ (to run `./scripts/`)

## Program Usage

To run the program:

```sh
cargo run
```

The following command-line options are available:

```sh
Usage: checkers-redux [OPTIONS]

Options:
      --p1-engine <P1_ENGINE>   Player 1 engine [default: ai] [possible values: ai, random]
      --p1-alpha-beta           Enable Alpha-Beta Pruning for Player 1
      --p1-transposition-table  Enable the use of a Transposition Table with Alpha-Beta Pruning for Player 1
      --p1-quiescence           Enable quiescence search for Player 1
      --p1-iterative            Enable iterative deepening search for Player 1
      --p1-depth <P1_DEPTH>     AI search depth limit for Player 1 [default: 6]
      --p1-eval <P1_EVAL>       Player 1 evaluation function [default: v1] [possible values: v1, v2, v3]
      --p2-engine <P2_ENGINE>   Player 2 engine [default: random] [possible values: ai, random]
      --p2-alpha-beta           Enable Alpha-Beta Pruning for Player 2
      --p2-transposition-table  Enable the use of a Transposition Table with Alpha-Beta Pruning for Player 2
      --p2-quiescence           Enable quiescence search for Player 2
      --p2-iterative            Enable iterative deepening search for Player 2
      --p2-depth <P2_DEPTH>     AI search depth limit for Player 2 [default: 6]
      --p2-eval <P2_EVAL>       Player 2 evaluation function [default: v1] [possible values: v1, v2, v3]
      --play                    You (Player 1) against the engine (Player 2)
  -g, --games <GAMES>           How many games to simulate [default: 1]
  -v, --verbose                 Show moves made by engines during simulation
  -h, --help                    Print help
```


## Example Output

```sh
$ ./checkers-redux --p1-transposition-table --p1-quiescence --p1-eval v3
config.games = 1
config.verbose = false
config.player1.engine = ai
config.player1.alpha_beta = false
config.player1.transposition_table = true
config.player1.quiescence = true
config.player1.depth = 6
config.player1.iterative = false
config.player1.eval = v3
config.player2.engine = random
config.player2.alpha_beta = false
config.player2.transposition_table = false
config.player2.quiescence = false
config.player2.depth = 6
config.player2.iterative = false
config.player2.eval = v1
game.d2965032-dcad-431b-9346-4144c68a08b0.winner = player1
game.d2965032-dcad-431b-9346-4144c68a08b0.player1.moves = 19
game.d2965032-dcad-431b-9346-4144c68a08b0.player1.explored = 60827
game.d2965032-dcad-431b-9346-4144c68a08b0.player1.beta_cuts = 20600
game.d2965032-dcad-431b-9346-4144c68a08b0.player1.tt_exact = 505
game.d2965032-dcad-431b-9346-4144c68a08b0.player1.tt_cuts = 2486
game.d2965032-dcad-431b-9346-4144c68a08b0.player1.max_depth = 14
game.d2965032-dcad-431b-9346-4144c68a08b0.player2.moves = 18
game.d2965032-dcad-431b-9346-4144c68a08b0.player2.explored = 0
game.d2965032-dcad-431b-9346-4144c68a08b0.player2.beta_cuts = 0
game.d2965032-dcad-431b-9346-4144c68a08b0.player2.tt_exact = 0
game.d2965032-dcad-431b-9346-4144c68a08b0.player2.tt_cuts = 0
game.d2965032-dcad-431b-9346-4144c68a08b0.player2.max_depth = 0
```
