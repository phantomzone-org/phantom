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

const MAX_LENGTH: usize = 20;
#[repr(C)]
struct UpperBoundedString {
    characters: [u8; MAX_LENGTH],
}

impl UpperBoundedString {
    fn new(string: &str) -> Self {
        if string.len() > MAX_LENGTH {
            panic!("String is too long");
        }
        let mut characters = [0u8; MAX_LENGTH];
        for (i, &byte) in string.as_bytes().iter().enumerate() {
            characters[i] = byte;
        }
        Self { characters }
    }

    fn eq(&self, other: &Self) -> bool {
        for i in 0..MAX_LENGTH {
            if self.characters[i] != other.characters[i] {
                return false;
            }
        }
        true
    }
}

#[repr(C)]
struct Input {
    input_string: UpperBoundedString,
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

    let hidden_string = UpperBoundedString::new("mississippi");

    let is_match = hidden_string.eq(&input_string);

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
