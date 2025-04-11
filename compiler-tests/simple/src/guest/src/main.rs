#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::panic::PanicInfo;
use macros::entry;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern crate alloc;
extern crate runtime;

/// Outputs of the program
///
/// Contains the resulting anonynous identifier and the corresponding signature
///
/// `signature` is verified using `PUBLIC_KEY_PROGRAM`, the public key corresponding to `SECRET_KEY_PROGRAM`
#[repr(C)]
#[derive(Default)]
struct Output {
    value: u32,
}

/// Inputs to the program
///
/// Contains the unique identifier of sybil resistant identity and the signature attesting to its validity from the issuing authority.
///
/// `signature` is verified using `PUBLIC_KEY_ISSUING_AUTH`, the public key of the issuing authority
#[repr(C)]
struct Input {
    value: u32,
}

#[no_mangle]
#[link_section = ".inpdata"]
static INPUT: [u8; core::mem::size_of::<Input>()] = [0u8; core::mem::size_of::<Input>()];

#[no_mangle]
#[link_section = ".outdata"]
// Use SyncUnsafeCell after `static mut` gets decpreated: https://github.com/rust-lang/rust/issues/95439
static mut OUTPUT: [u8; core::mem::size_of::<Output>()] = [0u8; core::mem::size_of::<Output>()];

#[entry]
fn main() {
    // READ INPUT
    let mut input: Input =
        unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };

    // parse inputs
    let mut input_value = input.value;

    while input_value < 100_000 {
        input_value = input_value * 2;
    }

    let mut out = Output::default();
    out.value = input_value;

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
