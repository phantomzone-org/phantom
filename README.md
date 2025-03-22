# Phantom

Phantom is a prototype fully encrypted RISC-V virtual machine that executes encrypted RISC-V binaries on encrypted inputs.

It enables black-box execution of any RISC-V program, allowing developers to write code with hidden instructions, constants, and states. This opens the door to new types of applications.

## How it works

Developers write their programs in Rust, which are then compiled into RISC-V binaries. These binaries are transformed into a polynomial representation, optimized for execution within the plaintext space of RLWE-base FHE. These polynomials are then encrypted, producing the encrypted program, which can then be executed by the Phantom executor on arbitrary encrypted and/or plaintext inputs.

To we recommend to look at full end to end examples in `compiler-tests` directory. In particular, the [sybil-resistant-id](./compiler-tests/sybil-resistant-id/) example.

The Phantom executor is a collection of FHE circuits that, collectively, simulate a RISC-V virtual machine. It is implemented in the `./fhevm` directory, with specifications detailed in [./fhevm/doc](./fhevm/doc). A [poster]() also provides a high-level overview of the end-to-end execution flow.

Both the implementation and specifications are actively evolving and subject to change. We encourage feedback and questions—feel free to open an issue.

## Current status

Phantom is still in active development and currently implements only a prototype of the polynomial executor. All circuits are built using plaintext polynomials, allowing for accurate runtime estimations of a single cycle. In the coming months, these plaintext circuits will be replaced with versions that operate on encrypted polynomials.

## Program obfuscation

Phantom may resemble program obfuscation, but it is not. However, it could potentially be used for practical program obfuscation in the future, by offloading the main computation to an FHE-scheme and limiting the obfuscation to a [conditional decryptor](https://eprint.iacr.org/2017/240.pdf)).

## Setup

After cloning the repository, run the following script at the root

```
./setup.sh
```

The script clones `pouply`, our FHE backend library, and initializes the submodules. `poulpy` is cloned at `..`, that is parent directory of `phantom`.

After this you're ready to run examples under `compiler-tests`.

## Acknowledgement

-   We thank authors of [nexus-zkvm](https://github.com/nexus-xyz/nexus-zkvm) for their `runtime`, `sdk` crate using which, as a reference, we've implemented our `runtime`, `compiler` crates.
-   We thank authors of [risc0](https://github.com/risc0/risc0) and [nexus-zkvm](https://github.com/nexus-xyz/nexus-zkvm) from which we've taken `alloc.rs` implementation.
