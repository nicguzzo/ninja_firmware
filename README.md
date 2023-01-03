# Ninja keyboard firmware

## Requirements

- requires rust nightly
``` console
rustup toolchain install nightly
```
- arm toolchain
``` console
rustup target install thumbv7m-none-eabi
```

- flip-link stack overflow protection
``` console
cargo install flip-link
```

- probe-run
``` console
cargo install probe-run
```
## Build

### debug
``` console
cargo run --features model_ninja1 --no-default-features
```

### release
``` console
cargo run -r --features model_ninja1 --no-default-features
```
