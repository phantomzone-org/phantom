#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::{ops::Div, panic::PanicInfo};
use macros::entry;
extern crate alloc;
extern crate runtime;

///////////////////////////////////////////////// 
///////////////////////////////////////////////// 
// TODO: Define input and output structs here.

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
struct Output {
    volume: f32,
}

#[repr(C)]
struct Input {
    radius: f32,
    height: f32,
}

// End of input and output structs.
/////////////////////////////////////////////////
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
    // READ INPUT. No need to change this.
    let mut input: Input =
        unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };

    // TODO: Read inputs into local variables.
    let radius = input.radius;
    let height = input.height;

    // TODO: Write your code here.
    // As an example, we calculate the volume of a cylinder here.
    let volume = core::f64::consts::PI as f32 * radius * radius * height;

    // TODO: Write output to Output struct.
    let output_str = Output { volume };
    
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
