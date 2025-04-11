#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::panic::PanicInfo;
use k256::{
    ecdsa::{
        signature::{Signer, Verifier},
        Signature, SigningKey, VerifyingKey,
    },
    sha2::{Digest, Sha256},
};
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
struct Output {
    signature: [u8; 64],
    /// Anonymous identifier
    anon_id: [u8; 32],
}

impl Default for Output {
    fn default() -> Self {
        Self {
            signature: [0u8; 64],
            anon_id: [0u8; 32],
        }
    }
}

/// Inputs to the program
///
/// Contains the unique identifier of sybil resistant identity and the signature attesting to its validity from the issuing authority.
///
/// `signature` is verified using `PUBLIC_KEY_ISSUING_AUTH`, the public key of the issuing authority
#[repr(C)]
struct Input {
    /// Signature attesting `unique_id`'s validity
    signature: [u8; 64],
    /// Unique identifier
    unique_id: [u8; 64],
}

/// Secret key hardcoded inside risc-v program.
///
/// Corresponding public key `PUBLIC_KEY_PROGRAM` is known publicly to verify signatures produced by this secret
#[no_mangle]
static SECRET_KEY_PROGRAM: [u8; 32] = [
    147, 214, 11, 48, 26, 149, 162, 45, 159, 209, 214, 102, 175, 173, 208, 30, 199, 195, 8, 172,
    143, 146, 187, 186, 144, 173, 48, 132, 178, 59, 101, 148,
];

/// Hardcoded secert SALT
///
/// `SALT` is concatenated with `unique_id` and then hased (Sha256) to produce the anonymous identifier
#[no_mangle]
static SALT: [u8; 32] = [
    175, 142, 86, 41, 61, 122, 186, 56, 50, 101, 187, 215, 124, 127, 14, 221, 109, 201, 110, 189,
    174, 1, 87, 170, 113, 193, 170, 115, 85, 51, 79, 172,
];

/// Public key of the issuing authority
#[no_mangle]
static PUBLIC_KEY_ISSUING_AUTH: [u8; 33] = [
    2, 84, 34, 56, 151, 6, 19, 16, 128, 51, 127, 122, 130, 105, 166, 135, 58, 206, 146, 227, 10,
    105, 123, 3, 17, 226, 60, 250, 40, 21, 229, 13, 102,
];

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
    let signature = Signature::from_slice(&input.signature).unwrap();
    let unique_identifier = &input.unique_id;
    let pk = VerifyingKey::from_sec1_bytes(&PUBLIC_KEY_ISSUING_AUTH).unwrap();

    let mut out = Output::default();
    match pk.verify(unique_identifier.as_slice(), &signature) {
        Ok(_) => {
            // Signature by issuing authority on unique identifier is valid

            // Issue: anonyomus identifier = sha256(unique_identifier || SALT)
            let mut hasher = Sha256::new();
            hasher.update(unique_identifier.as_slice());
            hasher.update(SALT.as_slice());
            out.anon_id = hasher.finalize().into();

            // sign anonymous identifier using hardcoded `SECRET_KEY_PROGRAM`
            // the resulting signature can be verified with `PUBLIC_KEY_PROGRAM`
            let sk = SigningKey::from_slice(&SECRET_KEY_PROGRAM).unwrap();
            let signature: Signature = sk.sign(out.anon_id.as_slice());
            out.signature
                .copy_from_slice(signature.to_bytes().as_slice());
        }
        Err(_) => {}
    }

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
