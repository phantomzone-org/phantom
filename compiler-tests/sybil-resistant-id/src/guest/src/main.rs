#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::panic::PanicInfo;
use k256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, SigningKey, VerifyingKey,
};
use macros::entry;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern crate alloc;
extern crate runtime;

#[repr(C)]
struct Output {
    b: bool,
}

#[repr(C)]
struct Input {
    signature: [u8; 64],
    message: [u8; 64],
}

static PUBLIC_KEY: [u8; 33] = [
    2, 84, 34, 56, 151, 6, 19, 16, 128, 51, 127, 122, 130, 105, 166, 135, 58, 206, 146, 227, 10,
    105, 123, 3, 17, 226, 60, 250, 40, 21, 229, 13, 102,
];

#[no_mangle]
#[link_section = ".inpdata"]
static INPUT: [u8; core::mem::size_of::<Input>()] = [0u8; core::mem::size_of::<Input>()];

#[no_mangle]
#[link_section = ".outdata"]
// Use SyncUnsafeCell when `static mut` gets decpreated: https://github.com/rust-lang/rust/issues/95439
static mut OUTPUT: [u8; core::mem::size_of::<Output>()] = [0u8; core::mem::size_of::<Output>()];

#[entry]
fn main() {
    // READ INPUT
    let mut input: Input =
        unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };

    let signature = Signature::from_slice(&input.signature).unwrap();
    let message = &input.message;
    let public_key = VerifyingKey::from_sec1_bytes(&PUBLIC_KEY).unwrap();

    let mut out = Output { b: false };
    if public_key.verify(message, &signature).is_ok() {
        out.b = true;
    }

    // WRITE OUTPUT
    unsafe {
        core::ptr::copy_nonoverlapping(
            (&out as *const Output) as *const u8,
            OUTPUT.as_mut_ptr(),
            core::mem::size_of::<Output>(),
        )
    };
}
