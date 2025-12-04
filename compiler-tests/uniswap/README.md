# Uniswap AMM

This example implements the constant function automated algorithm as an encrypted program.

The pool, the trade, and the user account (with respective token balances) are provided as encrypted inputs to the program. The program executes the trade and produces the updated pool and user's account post trade as the output.

## Project structure
For more information related to the program structure, please refer to readme of the [template](../template/README.md).

## Compiling and running the program
To compile and run the program, run the following command in this directory to execute the encrypted program.
```bash
# Without AVX2 and FMA support
PHANTOM_THREADS=32 PHANTOM_VERBOSE_TIMINGS=true PHANTOM_DEBUG=false MAX_CYCLES=300 cargo run --release

# With AVX2 and FMA support
RUSTFLAGS="-C target-feature=+avx2,+fma" PHANTOM_THREADS=32 PHANTOM_VERBOSE_TIMINGS=true PHANTOM_DEBUG=false MAX_CYCLES=700 cargo run --release
```