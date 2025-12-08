# Phantom

Phantom is a fully encrypted RISC-V virtual machine that executes encrypted RISC-V binaries on encrypted inputs.

It enables black-box execution of any RISC-V program, allowing developers to write code with hidden instructions, constants, states, and make its encrypted version public. The encrypted version can then be executed on any arbitrary encrypted inputs using the Phantom VM.

<p align="center">
  <img src="doc/phantom.png" alt="Phantom Overview"/>
</p>

## Building Phantom

We provide setup scripts to build Phantom and required dependencies from scratch:
- **Ubuntu/Debian**: [setup.sh](./scripts/setup.sh)
- **macOS**: [setup-macos.sh](./scripts/setup-macos.sh)

### macOS Setup Notes

On macOS, this project requires `rustup` (not Homebrew Rust) to add the `riscv32i-unknown-none-elf` target needed for compiling guest programs. If you have Rust installed via Homebrew, please uninstall it first:

```bash
brew uninstall rust rust-analyzer
```

Then run the macOS setup script:

```bash
./scripts/setup-macos.sh
```

## How to use Phantom

Developers write their programs in Rust, which are then compiled into RISC-V binaries. These binaries are transformed into a polynomial representation, optimized for execution within the plaintext space of RLWE-based FHE.
These polynomials are then encrypted, producing the encrypted program, which can then be executed by the Phantom VM on arbitrary encrypted and/or plaintext inputs.

To use, we recommend to look at full end to end examples in `compiler-tests` directory. In particular, the [template](./compiler-tests/template/) example to start programming in Phantom, and [otc](./compiler-tests/otc/) for a more advanced example.

## Architecture

Phantom VM is a collection of FHE circuits that collectively simulate a RISC-V virtual machine.
The Phantom VM is implemented in the [fhevm](./fhevm) directory.
The architecture of the Phantom VM is described in [doc/spec.png](./doc/spec.png).
It consists of 6 major components:
- Reading the instruction components from the ROM
- Reading the registers
- Reading the RAM
- Updating the registers
- Updating the RAM
- Updating the PC

The dependency graph of the operations performed in these components is described in [doc/costs.md](./doc/costs.md), which shows how Phantom can be further parallelized.

## Benchmark

We benchmark Phantom on a AWS r6i.metal, with support for AVX2 and FMA instructions, parallelized across 32 cores, and measure the runtime of a single cycle and all 6 components.

Average Cycle Time: 655.711279ms
  1. Read and prepare instruction components: 128.463434ms
     - Read instruction components: 28.019331ms
     - Prepare instruction components: 100.443453ms
  2. Read and prepare registers: 106.049704ms
     - Read registers: 7.524631ms
     - Prepare registers: 98.524193ms
  3. Read ram: 71.201851ms
  4. Update registers: 203.944757ms
     - Evaluate rd ops: 133.086641ms
     - Blind selection: 1.544176ms
     - Write rd: 69.312716ms
  5. Update ram: 72.566678ms
  6. Update pc: 73.433935ms
     - PC update BDD: 18.872475ms
     - PC prepare: 54.560689ms


To reproduce benchmarks for a single cycle, set the number of threads and run:
```
# Without AVX2 and FMA support
PHANTOM_THREADS=[# of threads] cargo bench --package fhevm --bench cycle

# With AVX2 and FMA support
RUSTFLAGS="-C target-feature=+avx2,+fma" PHANTOM_THREADS=[# of threads] cargo bench --package fhevm --bench cycle
```

## Acknowledgement

Development of Phantom is primarily supported by the [Ethereum foundation](https://ethereum.foundation/).
