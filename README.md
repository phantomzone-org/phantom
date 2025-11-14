# Phantom

Phantom is a fully encrypted RISC-V virtual machine that executes encrypted RISC-V binaries on encrypted inputs.

It enables black-box execution of any RISC-V program, allowing developers to write code with hidden instructions, constants, states, and make its encrypted version public. The encrypted version can then be executed on any arbitrary encrypted inputs using the Phantom VM.

## How to use

Developers write their programs in Rust, which are then compiled into RISC-V binaries. These binaries are transformed into a polynomial representation, optimized for execution within the plaintext space of RLWE-based FHE. These polynomials are then encrypted, producing the encrypted program, which can then be executed by the Phantom VM on arbitrary encrypted and/or plaintext inputs.

To use, we recommend to look at full end to end examples in `compiler-tests` directory. In particular, the [otc](./compiler-tests/otc/) example.

## Architecture

Phantom VM is a collection of FHE circuits that, collectively, simulate a RISC-V virtual machine. It is implemented in the `./fhevm` directory. 

TODO: provide directions to the different parts of the architecture here

## Benchmark

TODO

## Contribute
