# Anonymous sybil resistant identity system

Assume there exists sybil resistant identity system where a identity is the tuple (`Signature`, `UniqueIdentifier`). `UniqueIdentifier` is a unique identifier, not necessarily hiding, that identifies the identify holder and `Signature` attests to validity of the identity. An example of a sybil resistant identity system is e-passports where `UniqueIdentifier` is holder's personally identifiable data (i.e. Name, DOB, etc.) and `Signature` is signature on `UniqueIdentifier` by the issuing authority.

This examples shows how to bridge any sybil resistant identiy system to anonyomous identity using `phantom`. The encrypted risc-v program hardcodes two secrets: `SECRET_KEY_PROGRAM` and `SALT`. More so, we assume that public key `PUBLIC_KEY_PROGRAM` for `SECRET_KEY_PROGRAM` is known publicly, s.t. it can used to verify signatures produced using `SECRET_KEY_PROGRAM`. The program takes a sybil resistant identity tuple (`Signature`, `UniqueIdentifier`) as inputs. It first verifies `Signature` using fixed public key, `PUBLIC_KEY_ISSUING_AUTH`, of the issuing authority. If verification succeeds it calculates a new anonymous identifier as `Sha256(UniqueIdentifier || SALT)`. It then signs the resulting anonymous identifier using `SECRET_KEY_PROGRAM` and outputs both the anonynous identifier and corresponding signature.

It's clear that the resulting anonyomus identifier is deterministic from, and cannot be linked with, `UniqueIdentifier`. This is because harcoded secret `SALT` remains unknown.

## Structure

The risc-v program meant to be encrypted is defined in `src/guest/src/main.rs`. The corresponding tests are defined in `main.rs`.

Any phantom program will have two different cargo projects - the `guest` and the `main` project. The `guest` defines the actual encrypted program and the `main` can be used to compile, encrypt, and execute the `guest` program. For compilation, `guest` must be in the workspace of `main` (check [Cargo.toml](./Cargo.toml)).

## Writing programs

The encrypted program is defined in `main.rs` file of the `guest` project and the [guest](./src/guest/src/main.rs) project of this example can be used a reference to write programs.

### Input and Output

Take [main.rs](./src/guest/src/main.rs) of the `guest` project as an example to understand how inputs and outputs are handled.

Inputs are fed to program via memory and the ouputs are obtained from a pre-defined section of the memory. This means the memory reserves a special space for inputs and outputs. This is done by first declaring sufficient size byte array as static constant and linking respective constants with respective sections in linker file.

```rust
#[no_mangle]
#[link_section = ".inpdata"]
static INPUT: [u8; core::mem::size_of::<Input>()] = [0u8; core::mem::size_of::<Input>()];

#[no_mangle]
#[link_section = ".outdata"]
// Use SyncUnsafeCell after `static mut` gets decpreated: https://github.com/rust-lang/rust/issues/95439
static mut OUTPUT: [u8; core::mem::size_of::<Output>()] = [0u8; core::mem::size_of::<Output>()];
```

`INPUT` constant reserves space for struct `Input` size byte array in memory at the `.inpdata` section. And `OUTPUT` constant reserves space for struct `Output` size byte array in memory at the `.outdata` section.

After compilation, `INPUT` and `OUTPUT` constants will have allocated space in the memory of the program. The inputs are mapped to respective allocated memory before execution. And the ouputs are retrieved from respective allocated memory after execution.

Inside `main()` function, inputs are parsed as `Input` struct with

```rust
let mut input: Input =
        unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };

```

This means all inputs must be declared inside the `Input` struct. For example, the following struct

```rust
#[repr(C)]
struct Input {
    /// Signature attesting `unique_id`'s validity
    signature: [u8; 64],
    /// Unique identifier
    unique_id: [u8; 64],
}
```

contains two inputs `signature` and `unique_id`

At the end of `main` function, `Output` struct is constructed with required outputs. For example, the following struct

```rust
#[repr(C)]
struct Output {
    signature: [u8; 64],
    /// Anonymous identifier
    anon_id: [u8; 32],
}
```

has two outputs `signature` and `anon_id`.

The `Output` struct is then written to pre-allocated output memory with

```rust
unsafe {
        core::ptr::copy_nonoverlapping(
            (&out as *const Output) as *const u8,
            OUTPUT.as_mut_ptr(),
            core::mem::size_of::<Output>(),
        )
    };
```

Note that both `Input` and `Output` struct must have `#[repr(C)]` attribute. This makes sure that the structs follow `C`'s struct representation and have determinsitic layout.

### Selective reveal

In [main.rs](./src/guest/src/main.rs) inside `guest`, observe that `SALT`, `SECRET_KEY_PROGRAM`, `PUBLIC_KEY_ISSUING_AUTH` are defined as static constants in the program. When the resulting binary is encrypted, all 3 will be encrypted. However, only `SALT`, `SECRET_KEY_PROGRAM` need to remain secret not `PUBLIC_KEY_ISSUING_AUTH`. In fact, it may be necessary sometimes to have some part of the program in the clear. For instance, in this example it may be necessary for `PUBLIC_KEY_ISSUING_AUTH` to appear in clear such that it can be verified that it is infact the public key of valid issuing authority. We call this property "selective reveal".

Selectively revealing parts of the program is not allowed yet. But it'll be enabled in the future at the stage of encryption: simply don't encrypt parts of the program that are meant to appear in the clear.

## Run tests

Run the tests with the following command at the root

```
cargo run --release
```

## Acknowledgement

-   We thank [Sora Suegami](https://github.com/SoraSuegami) for suggesting anonymous sybil resistant identity system as a use-case for encrypted programs.
