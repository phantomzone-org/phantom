#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::{ops::Div, panic::PanicInfo};
use macros::entry;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern crate alloc;
extern crate runtime;

///////////////////////////////////////////////// 
// TODO: Define input and output structs here.

#[repr(C)]
struct Output {
    volume: f64,
}

#[repr(C)]
struct Input {
    radius: f64,
    height: f64,
}

// End of input and output structs.
/////////////////////////////////////////////////

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
    // READ INPUT
    let mut input: Input =
        unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };

    let radius = input.radius;
    let height = input.height;

    // Write you code here
    // As an exmaple, we calculate the volume of a cylinder here.

    let volume = core::f64::consts::PI * radius * radius * height;

    // WRITE OUTPUT
    let output_str = Output { volume };
    unsafe {
        core::ptr::copy_nonoverlapping(
            (&output_str as *const Output) as *const u8,
            OUTPUT.as_mut_ptr(),
            core::mem::size_of::<Output>(),
        )
    };

    loop {}
}
