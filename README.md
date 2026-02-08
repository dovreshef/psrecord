# psrecord

Small CLI to run a command, sample its memory/CPU usage, and generate graphs.

## What it does

- Runs your command and monitors RSS + CPU over time.
- Prints ASCII graphs to `stdout` (optional).
- Writes PNG graphs to an output directory.
- Returns the wrapped command's exit code.

## Usage

```bash
cargo run -- -- <command> [args...]
```

Examples:

```bash
# default output dir: ./psrecord-output
cargo run -- -- python3 -c "import time; x=bytearray(100_000_000); time.sleep(2)"

# disable ASCII output
cargo run -- --no-ascii -- sleep 2

# custom interval/output/image size
cargo run -- --interval 200 --output out --width 1280 --height 720 -- sleep 3
```

## Memory scale behavior

Memory graphs auto-select one unit per run based on peak RSS:

- `< 1 MiB` -> `KB`
- `< 1 GiB` -> `MB`
- `< 1 TiB` -> `GB`
- `>= 1 TiB` -> `TB`

The same unit is used for both ASCII and PNG memory graphs.

## Development

```bash
cargo +nightly fmt --all
cargo clippy --all-targets --all-features
cargo test
```

## License

Apache License 2.0. See `LICENSE`.
