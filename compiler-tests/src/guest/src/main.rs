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
struct Pool {
    t0: u32,
    t1: u32,
}

#[repr(C)]
struct Output {
    pool: Pool,
    out0: u32,
    out1: u32,
}

#[repr(C)]
struct Input {
    pool: Pool,
    inp0: u32,
    inp1: u32,
}

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

    let mut pool = input.pool;
    let inp0 = input.inp0;
    let inp1 = input.inp1;

    let mut out0 = 0;
    let mut out1 = 0;

    if pool.t0 > inp0 {
        pool.t0 -= inp0;
        out0 = inp0;
    }

    if pool.t1 > inp1 {
        pool.t1 -= inp1;
        out1 = inp1;
    }

    // WRITE OUTPUT
    let output_str = Output { pool, out0, out1 };
    unsafe {
        core::ptr::copy_nonoverlapping(
            (&output_str as *const Output) as *const u8,
            OUTPUT.as_mut_ptr(),
            core::mem::size_of::<Output>(),
        )
    };
}
