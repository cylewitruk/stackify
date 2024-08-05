alias cli := stackify

stackify *PARAMS:
    cargo run --package stackify-cli --bin stackify -- {{PARAMS}}