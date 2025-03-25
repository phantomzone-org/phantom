# Anonymous sybil resistant identity system

Assume a sybil-resistant identity system where an identity is represented as a tuple (`Signature`, `UniqueIdentifier`). Here, `UniqueIdentifier` is a unique, not necessarily hidden, identifier that distinguishes the identity holder, while `Signature` attests to its validity. An example of such a system is e-passports, where `UniqueIdentifier` consists of the holder’s personally identifiable data (e.g., Name, DOB, etc.), and `Signature` is a digital signature issued by the relevant authority.  

This example demonstrates how to use `phantom` to bridge any sybil-resistant identity system to an anonymous identity. The encrypted RISC-V program embeds two secrets: `SECRET_KEY_PROGRAM` and `SALT`. Additionally, we assume that the public key `PUBLIC_KEY_PROGRAM` corresponding to `SECRET_KEY_PROGRAM` is publicly known, allowing verification of signatures produced with `SECRET_KEY_PROGRAM`.  

The program takes a sybil-resistant identity tuple (`Signature`, `UniqueIdentifier`) as input. It first verifies `Signature` using the fixed public key `PUBLIC_KEY_ISSUING_AUTH` of the issuing authority. If verification succeeds, it computes a new anonymous identifier as `Sha256(UniqueIdentifier || SALT)`. It then signs this anonymous identifier using `SECRET_KEY_PROGRAM` and outputs both the anonymous identifier and its corresponding signature.  

Since the hardcoded secret `SALT` remains unknown, the resulting anonymous identifier is deterministic yet unlinkable to `UniqueIdentifier`.

## Structure

The RISC-V program intended for encryption is defined in `src/guest/src/main.rs`, with its corresponding tests located in `main.rs`.  

Every `phantom` program consists of two separate Cargo projects: `guest` and `main`. The `guest` project defines the actual encrypted program, while `main` handles compilation, encryption, and execution of `guest`. To compile successfully, `guest` must be included in `main`'s workspace (check [Cargo.toml](./Cargo.toml)).

## Writing programs

The encrypted program is defined in the `main.rs` of the `guest` project and the [guest](./src/guest/src/main.rs) project of this example can be used a reference to write other programs.

### Input and Output

Refer to [main.rs](./src/guest/src/main.rs) in the `guest` project for an example of how inputs and outputs are managed.  

Inputs are provided via memory, and outputs are retrieved from a predefined memory section. This approach reserves a dedicated memory space for inputs and outputs. It is implemented by declaring a sufficiently sized byte array as a static constant and linking these constants to the appropriate sections in the linker script.

```rust
#[no_mangle]
#[link_section = ".inpdata"]
static INPUT: [u8; core::mem::size_of::<Input>()] = [0u8; core::mem::size_of::<Input>()];

#[no_mangle]
#[link_section = ".outdata"]
// Use SyncUnsafeCell after `static mut` gets decpreated: https://github.com/rust-lang/rust/issues/95439
static mut OUTPUT: [u8; core::mem::size_of::<Output>()] = [0u8; core::mem::size_of::<Output>()];
```

The `INPUT` constant reserves space in memory for a byte array matching the size of the `Input` struct, placing it in the `.inpdata` section. Similarly, the `OUTPUT` constant reserves space for a byte array corresponding to the `Output` struct in the `.outdata` section.  

After compilation, `INPUT` and `OUTPUT` have dedicated memory allocations within the program. Before execution, inputs are mapped to their allocated memory, and after execution, outputs are retrieved from their respective memory locations.

Inside the `main()` function, inputs are parsed into an `Input` struct using:  

```rust
let mut input: Input =
        unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };

```

This requires all inputs to be defined within the `Input` struct. For example:  

```rust
#[repr(C)]
struct Input {
    /// Signature attesting to the validity of `unique_id`
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

Note that both the `Input` and `Output` structs must include the `#[repr(C)]` attribute. This ensures that the structs follow the `C` language's struct representation, guaranteeing a deterministic layout.

### Selective reveal

In [main.rs](./src/guest/src/main.rs) within the `guest` project, you'll notice that `SALT`, `SECRET_KEY_PROGRAM`, and `PUBLIC_KEY_ISSUING_AUTH` are defined as static constants. When the binary is encrypted, all three constants will be encrypted. However, only `SALT` and `SECRET_KEY_PROGRAM` need to remain secret, while `PUBLIC_KEY_ISSUING_AUTH` does not.  

In some cases, it may be necessary for parts of the program to remain in the clear. For example, in this case, `PUBLIC_KEY_ISSUING_AUTH` might need to be visible so it can be verified as the public key of a valid issuing authority. This concept is referred to as "selective reveal."

Selective revealing of program parts is not currently supported, but it will be enabled in the future during the encryption stage. At that point, parts of the program intended to remain in the clear can simply be left unencrypted.

## Run tests

Run the tests with the following command at the root

```
cargo run --release
```

## Acknowledgement

-   We thank [Sora Suegami](https://github.com/SoraSuegami) for suggesting anonymous sybil resistant identity system as a use-case for encrypted programs.
