#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::panic::PanicInfo;
use macros::entry;
extern crate alloc;
extern crate runtime;

///////////////////////////////////////////////// 
///////////////////////////////////////////////// 
// Define input and output structs here.

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
struct Output {
    evaluation: u32,
}

#[repr(C)]
struct Input {
    point: u32,
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

    // Read inputs into local variables.
    let point = input.point;

    // Write your code here.
    // As an example, we evaluate a polynomial

    // Define coefficients for the polynomial.
    let coefficients: [u32; 7] = [123, 456, 789, 12, 3456, 7, 89];

    let mut evaluation = 0;
    let mut pow_point = 1;
    for coeff in coefficients.iter() {
        evaluation += coeff * pow_point;
        pow_point *= input.point;
    }

    // Write output to Output struct.
    let output_str = Output { evaluation };
    
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
