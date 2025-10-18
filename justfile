export RUSTFLAGS := "-Dwarnings"
export RUSTDOCFLAGS := "-Dwarnings"
export CARGO_TERM_COLOR := "always"

clippy:
  cargo clippy

clippy-fix:
  cargo clippy --fix

fmt-check:
  cargo fmt --check

fmt:
  cargo fmt

check:
  cargo check

test:
  cargo test

build:
  nix build

run:
  waybar -c waybar.json -s style.css
