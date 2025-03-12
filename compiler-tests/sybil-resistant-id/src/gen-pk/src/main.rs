#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::panic::PanicInfo;
use k256::ecdsa::{SigningKey, VerifyingKey};
use macros::entry;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern crate alloc;
extern crate runtime;

#[repr(C)]
struct Output {
    pk_bytes: [u8; 33],
    b: u8,
}

#[repr(C)]
struct Input {}

static SECRET_KEY: [u8; 32] = [
    112, 241, 22, 238, 183, 253, 29, 115, 104, 106, 220, 187, 201, 91, 199, 254, 63, 240, 149, 71,
    141, 43, 156, 246, 48, 169, 197, 254, 208, 40, 218, 192,
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

    let mut out = Output {
        pk_bytes: [0u8; 33],
        b: 0,
    };

    let sk = SigningKey::from_slice(&SECRET_KEY);
    if sk.is_ok() {
        let sk = sk.unwrap();
        let pk = VerifyingKey::from(sk).to_sec1_bytes();
        out.pk_bytes.copy_from_slice(pk.iter().as_slice());
        out.b = pk.len() as u8;
    } else {
        out.b = 1;
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
