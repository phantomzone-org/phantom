use compiler::{interpreter::TestVM, CompileOpts};
use core::ptr;
use k256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, SigningKey,
};
use rand::{rng, RngCore};

#[derive(Debug)]
#[repr(C)]
struct Output {
    b: bool,
}

#[repr(C)]
struct Input {
    signature: [u8; 64],
    message: [u8; 64],
}

fn to_u8_slice<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((v as *const T) as *const u8, core::mem::size_of::<T>()) }
}

fn from_u8_slice<T>(v: &[u8]) -> T {
    unsafe { ptr::read(v.as_ptr() as *const T) }.into()
}

static SECRET_KEY: [u8; 32] = [
    112, 241, 22, 238, 183, 253, 29, 115, 104, 106, 220, 187, 201, 91, 199, 254, 63, 240, 149, 71,
    141, 43, 156, 246, 48, 169, 197, 254, 208, 40, 218, 192,
];

static PUBLIC_KEY: [u8; 33] = [
    2, 84, 34, 56, 151, 6, 19, 16, 128, 51, 127, 122, 130, 105, 166, 135, 58, 206, 146, 227, 10,
    105, 123, 3, 17, 226, 60, 250, 40, 21, 229, 13, 102,
];

fn gen_random_case() -> Input {
    let mut rng = rng();

    let mut message: [u8; 64] = [0u8; 64];
    rng.fill_bytes(message.as_mut_slice());

    // Sign
    let sk = SigningKey::from_slice(&SECRET_KEY).unwrap();
    let signature: Signature = sk.sign(message.as_slice());
    let mut signature_bytes: [u8; 64] = [0u8; 64];
    signature_bytes.copy_from_slice(signature.to_bytes().as_slice());

    let input = Input {
        signature: signature_bytes,
        message,
    };

    input
}

fn main() {
    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build();

    let mut vm = TestVM::init(elf_bytes);
    let random_input = gen_random_case();
    let input_tape = to_u8_slice(&random_input);
    vm.read_input_tape(input_tape);
    while vm.is_exec() {
        vm.run();
    }

    let output_tape = vm.output_tape();
    let output: Output = from_u8_slice(&output_tape);
    println!("Output: {:?}", output);
}
