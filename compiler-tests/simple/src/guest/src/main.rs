#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::panic::PanicInfo;
use macros::entry;
use sha2::{Digest, Sha256};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern crate alloc;
extern crate runtime;

/// Outputs of the program
///
/// Contains the output hash = Sha256(message | SALT)
#[repr(C)]
#[derive(Default)]
struct Output {
    hash_value: [u8; 32],
}

/// Inputs to the program
#[repr(C)]
struct Input {
    message: [u8; 32],
}

/// Hardcoded hidden SALT
#[no_mangle]
static SALT: [u8; 32] = [
    175, 142, 86, 41, 61, 122, 186, 56, 50, 101, 187, 215, 124, 127, 14, 221, 109, 201, 110, 189,
    174, 1, 87, 170, 113, 193, 170, 115, 85, 51, 79, 172,
];

#[no_mangle]
#[link_section = ".inpdata"]
static INPUT: [u8; core::mem::size_of::<Input>()] = [0u8; core::mem::size_of::<Input>()];

#[no_mangle]
#[link_section = ".outdata"]
// Use SyncUnsafeCell after `static mut` gets deprecated: https://github.com/rust-lang/rust/issues/95439
static mut OUTPUT: [u8; core::mem::size_of::<Output>()] = [0u8; core::mem::size_of::<Output>()];

#[entry]
fn main() {
    // READ INPUT
    let mut input: Input =
        unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };

    // Exectue
    let mut out = Output::default();
    let mut hasher = Sha256::new();
    hasher.update(input.message.as_slice());
    hasher.update(SALT.as_slice());
    out.hash_value = hasher.finalize().into();

    // WRITE OUTPUT
    unsafe {
        core::ptr::copy_nonoverlapping(
            (&out as *const Output) as *const u8,
            OUTPUT.as_mut_ptr(),
            core::mem::size_of::<Output>(),
        )
    };

    loop {}
}
