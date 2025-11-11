# OTC quote with encrypted trade intent

The example implements a OTC desk quote generation program for BTC/USD pair. The program produces encrypted quotes for encrypted trade intents received from the clients.

Executing quote generation on encrypted intents mitigates front-running risks that usually restrict clients from seeking quotes from only 1 or 2 OTC desks. Now the client can send their trade intent to many OTC desks without reveling the trade volume nor the direction. Once the client receives encrypted quotes from multiple OTC desks, client decrypts the quotes, and selects the OTC desk that provides the best quote.

The implemented quote generation algorithm works over f32 values and is a simplified model that an OTC desk may use.

## Project structure

The main program is implemented inside main.rs file of the guest crate. The guest crate is part of the workspace of its parent crate ( otc in this case ). The parent crate is then responsible for compiling the guest crate and, subseuqntly, encrypting the compiled program.

Every phantom program should follow the same prject layout.

## Writing programs

The main.rs file must start with line
```rust
#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
```

And should declare a panic handler. The default choice is
```rust
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

The actual program must be within the main function of the main.rs file.

Inputs are declared with the Input struct, where the individuals fields of the struct are the inputs.

```rust
#[repr(C)]
struct Input {
    market_data: MarketData,
    client: ClientProfile,
    trade: Trade,
}
```

Note that the struct is has repr(C) attribute. This is to ensure the storage layout of the struct is predictable and all the structs must have the repre(C) attribute.

Outputs are declared with the Output struct, where, like Input struct, fields of the struct are the outputs.

```rust
#[repr(C)]
struct Output {
    quote: Quote,
}
```

Inputs and outputs are communicated to/from the program via the RAM, thus we need to reserve space for both. This is done by declaring the following static variables:

```rust
#[no_mangle]
#[link_section = ".inpdata"]
static INPUT: [u8; core::mem::size_of::<Input>()] = [0u8; core::mem::size_of::<Input>()];

#[no_mangle]
#[link_section = ".outdata"]
// Use SyncUnsafeCell when `static mut` gets decpreated: https://github.com/rust-lang/rust/issues/95439
static mut OUTPUT: [u8; core::mem::size_of::<Output>()] = [0u8; core::mem::size_of::<Output>()];
```

Notice that `OUTPUT` is declared `mut` whereas `INPUT` is not.

Inputs can read in the program with
```rust
let mut input: Input =
    unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };
```

Notice that inputs are read from a preallocated space ( due to INPUT static ) in the memory ( i.e. program's RAM )

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

Again, notice that output is written to a preallocated space ( due to OUTPUT` static ) in the memory.

One last thing remaning is `loop {}` declaration at the end of the program. This ensures that program enters an infinite loop directly after writing the output to the RAM. The program executes until it exhausts maximum number of cycles.
