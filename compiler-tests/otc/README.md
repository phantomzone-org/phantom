# OTC quote with encrypted trade intent

The example implements an OTC desk quote generation program for the BTC/USD pair. The program produces encrypted quotes for encrypted trade intents received from clients.

Executing quote generation on encrypted intents mitigates front-running risks that usually restrict clients to seeking quotes from only one or two OTC desks. Now clients can send their trade intents to many OTC desks without revealing either the trade volume or the direction. Once clients receive encrypted quotes from multiple OTC desks, they decrypt the quotes and select the OTC desk that provides the best quote.

The implemented quote generation algorithm works over f32 values and is a simplified model that an OTC desk may use.

## Project structure
For details about the structure of the project, please refer to the [template](../template/README.md) example.

## Compiling and running the program
To compile and run the program, run the following command in this directory to execute the encrypted program.
```bash
# Without AVX2 and FMA support
PHANTOM_THREADS=32 PHANTOM_VERBOSE_TIMINGS=true PHANTOM_DEBUG=false MAX_CYCLES=9000 cargo run --release

# With AVX2 and FMA support
RUSTFLAGS="-C target-feature=+avx2,+fma" PHANTOM_THREADS=32 PHANTOM_VERBOSE_TIMINGS=true PHANTOM_DEBUG=false MAX_CYCLES=700 cargo run --release
```

<!-- ## Project structure

The main program is implemented in the `main.rs` file of the guest crate. The guest crate is part of the workspace of its parent crate (`otc` in this case). The parent crate is then responsible for compiling the guest crate and, subsequently, encrypting the compiled program.

Every Phantom program should follow the same project layout.

## Writing programs

The `main.rs` file must start with the line
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
    market_data: MarketData,
    client: ClientProfile,
    trade: Trade,
}
```

Note that the struct has the `repr(C)` attribute. This ensures the storage layout of the struct is predictable, and all structs must have the `repr(C)` attribute.

Outputs are declared with the `Output` struct, where, like the `Input` struct, the fields of the struct are the outputs.

```rust
#[repr(C)]
struct Output {
    quote: Quote,
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

## Running the program

To run the program, use the following command in this directory:
```bash
cargo run --release
``` -->
