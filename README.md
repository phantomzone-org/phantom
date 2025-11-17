# Phantom

Phantom is a fully encrypted RISC-V virtual machine that executes encrypted RISC-V binaries on encrypted inputs.

It enables black-box execution of any RISC-V program, allowing developers to write code with hidden instructions, constants, states, and make its encrypted version public. The encrypted version can then be executed on any arbitrary encrypted inputs using the Phantom VM.

<p align="center">
  <img src="doc/phantom.png" alt="Phantom Overview"/>
</p>

## How to use Phantom

Developers write their programs in Rust, which are then compiled into RISC-V binaries. These binaries are transformed into a polynomial representation, optimized for execution within the plaintext space of RLWE-based FHE.
These polynomials are then encrypted, producing the encrypted program, which can then be executed by the Phantom VM on arbitrary encrypted and/or plaintext inputs.

To use, we recommend to look at full end to end examples in `compiler-tests` directory. In particular, the [template](./compiler-tests/otc/) example to start programming in Phantom, and [otc](./compiler-tests/otc/) for a more advanced example.

## Architecture

Phantom VM is a collection of FHE circuits that collectively simulate a RISC-V virtual machine.
The Phantom VM is implemented in the `./fhevm` directory.
The architecture of the Phantom VM is described in [doc/spec.png](./doc/spec.png).
The dependency graph of operations performed in a single cycle of Phantom is described in [doc/costs.md](./doc/costs.md), which shows how Phantom can be parallelized.

## Benchmark

TODO:

## Contribute

