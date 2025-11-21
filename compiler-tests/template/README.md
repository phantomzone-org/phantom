# Template Phantom Program

This example contains a minimal template for a Phantom program. It computes a polynomial, the coefficients of which are provided in the encrytped program, over an encrypted input.
For more advanced examples, check out the [otc](../otc/README.md) example.

## Project structure

The main program is implemented in the `main.rs` file of the guest crate. The guest crate is part of the workspace of its parent crate (`template` in this case). The parent crate is then responsible for compiling the guest crate and, subsequently, encrypting the compiled program.

Every Phantom program should follow the same project layout.

## Writing programs

### TL;DR Checklist for Writing a Phantom Program

To write a program for Phantom, inside the `guest` crate in `main.rs`:
- [ ] Define the input and output struct(s) for your program.
- [ ] Declare a panic handler function (or keep the default).
- [ ] Implement your desired logic in the `main` function.

To run the program, in `main.rs` the current directory:
- [ ] Redeclare the input and output struct(s) for your program.
- [ ] Provide sample inputs for your program.
- [ ] Run `PHANTOM_THREADS=[# of threads] PHANTOM_DEBUG=[true/false] PHANTOM_VERBOSE_TIMINGS=[true/false] cargo run --release` in this directory to run Phantom.
- [ ] For testing, implement the expected functionality to compare with Phantom's output.

### Explaining the Components of the Phantom Program

In the guest crate, the `main.rs` file must start with the line
```rust
#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
```

It should also declare a panic handler. The default choice is
```rust
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

The actual program must be within the `main` function of the `main.rs` file.

Inputs are declared with the `Input` struct, whose individual fields are the inputs.

```rust
#[repr(C)]
struct Input {
    point: u32,
}
```

Note that the struct has the `repr(C)` attribute. This ensures the storage layout of the struct is predictable, and all structs must have the `repr(C)` attribute.

Outputs are declared with the `Output` struct, where, like the `Input` struct, the fields of the struct are the outputs.

```rust
#[repr(C)]
struct Output {
    evaluation: u32,
}
```

Inputs and outputs are communicated to and from the program via RAM, so we need to reserve space for both. This is done by declaring the following static variables:

```rust
#[no_mangle]
#[link_section = ".inpdata"]
static INPUT: [u8; core::mem::size_of::<Input>()] = [0u8; core::mem::size_of::<Input>()];

#[no_mangle]
#[link_section = ".outdata"]
// Use SyncUnsafeCell when `static mut` gets deprecated: https://github.com/rust-lang/rust/issues/95439
static mut OUTPUT: [u8; core::mem::size_of::<Output>()] = [0u8; core::mem::size_of::<Output>()];
```

Notice that `OUTPUT` is declared `mut` whereas `INPUT` is not.

Inputs can be read in the program with
```rust
let mut input: Input =
    unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };
```

Notice that inputs are read from a preallocated space (because of the `INPUT` static) in memory (i.e., the program's RAM).

Outputs can be returned from the program with

```rust
let output_str = Output { quote };
unsafe {
    core::ptr::copy_nonoverlapping(
        (&output_str as *const Output) as *const u8,
        OUTPUT.as_mut_ptr(),
        core::mem::size_of::<Output>(),
    )
};
```

Again, notice that the output is written to a preallocated space (because of the `OUTPUT` static) in memory.

One last thing remaining is the `loop {}` declaration at the end of the program. This ensures that the program enters an infinite loop directly after writing the output to RAM. The program executes until it exhausts the maximum number of cycles.

## Compiling and running the program

We use `main.rs` in this directory to compile the program into RISC-V instructions, instantiate Phantom, and run the program.
To run the program, provide sample inputs for your program in `main.rs` and also choose how many cycles you want Phantom to execute.
Note that if Phantom does not execute enough cycles, it will not produce the expected output.

Then use the following command in this directory to run the encrypted program.
```bash
# Without AVX2 and FMA support
cargo run --release

# With AVX2 and FMA support
RUSTFLAGS="-C target-feature=+avx2,+fma" cargo run --release


```

For testing purposes, you can also implement the expected behavior in `main.rs` to compare with Phantom's output.
