# Phantom

Phantom is prototype of a fully encrypted risc-v virtual machine. It runs encrypted risc-v binaries on encrypted inputs.

Phantom enables encrypted programs, programs that are complete black-box to any observer. This allows developers to write programs with hidden instructions, constants, and states. And can be used for variety of new types of applications.

## How it works

Developers write their program as a rust program. Rust program is then compiled to a risc-v binary. The risc-v binary is transformed to a representation that is efficient to execute in plaintext space of FHE schemes. Concretely, the binary is transformed into a collection of polynomials. The polynomials are then encrypted by the developer and the resulting collection of encrypted polynomials is the encrypted program. The encrypted program can then be executed in the phantom executor on arbitrary encrypted inputs.

To we recommed to look at full end to end examples in `compiler-tests` directory. In particular, the [sybil-resistant-id](./compiler-tests/sybil-resistant-id/) example.

Phantom executor is a collection of FHE circuits that as a whole simulate risc-v virtual machine. The executor is implemented in `fhevm` directory. The specs of the implementaion are described inside [./fhevm/doc](./fhevm/doc) directory. There's also a [poster]() that captures the end to end execution flow at once. Both the implementation and the specs are under development and will change in future. We encourage others to ask questions, if any, by opening an issue.

## Current status

Phantom is under construction. At the moment, it only implements a prototype of the actual encrypted executore. All the circuits are implemented using plaintext polynomials. Doing so provides us with good estimations of runtime of a single cylce. In coming months, the plaintext circuits will be replaced with circuits that work over encrypted polynomials.

## Program obfuscation

Phantom may sound like program obfuscation, but it isn't. However, phantom may well be in future used for practical program obfuscation (in case, program obfuscation scheme off-loads all compute inside FHE while restricting itself to obfuscation of a [conditional decryptor](https://eprint.iacr.org/2017/240.pdf)).

Phantom, at the moment, can at-best offer `T-out-of-N` security. That is, assuming the ideal FHE secret key is sharded among N parties, Phantom remains secure unless `>T` parties collude.

## Setup

After cloning the repository, run the following script at the root

```
./setup.sh
```

The script clones `pouply`, our FHE backend library, and initialises necessary submodules. `poulpy` is cloned at `..`, that is parent directory of `phantom`.

After this you're ready to run examples under `compiler-tests`.

## Acknowledgement

-   We thank authors of [nexus-zkvm](https://github.com/nexus-xyz/nexus-zkvm) for their `runtime`, `sdk` crate using which, as a reference, we've implemented our `runtime`, `compiler` crates.
-   We thank authors of [risc0](https://github.com/risc0/risc0) and [nexus-zkvm](https://github.com/nexus-xyz/nexus-zkvm) from which we've taken `alloc.rs` implementation.
