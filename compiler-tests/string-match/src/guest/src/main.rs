#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::panic::PanicInfo;
use macros::entry;

extern crate alloc;
extern crate runtime;

use alloc::string::String;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
struct Output {
    is_match: bool,
}

#[repr(C)]
struct Input {
    input_string: String,
}

// Loading the input data on the tape. No need to change this.
#[no_mangle]
#[link_section = ".inpdata"]
static INPUT: [u8; core::mem::size_of::<Input>()] = [0u8; core::mem::size_of::<Input>()];

// Loading the output data from the tape. No need to change this.
#[no_mangle]
#[link_section = ".outdata"]
// Use SyncUnsafeCell when `static mut` gets decpreated: https://github.com/rust-lang/rust/issues/95439
static mut OUTPUT: [u8; core::mem::size_of::<Output>()] = [0u8; core::mem::size_of::<Output>()];

#[entry]
fn main() {
    // READ INPUT. No need to change this.
    let mut input: Input =
        unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };

    let input_string = input.input_string;

    let hidden_string = "mississippi";

    let is_match = (input_string == hidden_string);

    let output_str = Output { is_match };
    
    // write output to tape. No need to change this.
    unsafe {
        core::ptr::copy_nonoverlapping(
            (&output_str as *const Output) as *const u8,
            OUTPUT.as_mut_ptr(),
            core::mem::size_of::<Output>(),
        )
    };

    loop {}
}
