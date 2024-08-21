alias cli := stackify
alias d := stackifyd

stackify *PARAMS:
    cargo run --package stackify --bin stackify -- {{PARAMS}}

stackifyd *PARAMS:
    cargo run --package stackify-daemon --bin stackifyd -- {{PARAMS}}