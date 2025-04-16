# Simple

The risc-v program defined in `src/guest/main.rs` is harcoded with a secret `SALT`. It takes as input a random `message` and outputs `Sha256(message | SALT)`.

## Run test

```
cargo run --release
```
