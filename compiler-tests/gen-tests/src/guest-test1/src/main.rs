#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::panic::PanicInfo;
use macros::entry;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern crate alloc;
extern crate runtime;

#[repr(C)]
struct Output {
    out: u32,
}

#[repr(C)]
struct Input {
    in_a: u32,
    branch: bool,
}

#[no_mangle]
static HARDCODED_VALUE: [u32; 4] = [10, 20, 30, 40];

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

    let in_a = input.in_a;

    let out = if input.branch {
        HARDCODED_VALUE.iter().fold(0, |acc, v| acc + in_a + *v)
    } else {
        HARDCODED_VALUE.iter().fold(0, |acc, v| acc + (in_a * *v))
    };

    // WRITE OUTPUT
    let output_str = Output { out };
    unsafe {
        core::ptr::copy_nonoverlapping(
            (&output_str as *const Output) as *const u8,
            OUTPUT.as_mut_ptr(),
            core::mem::size_of::<Output>(),
        )
    };

    loop {}
}
