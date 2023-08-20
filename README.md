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
