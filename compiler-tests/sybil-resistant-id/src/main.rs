use compiler::{interpreter::TestVM, CompileOpts};
use core::ptr;
use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
    sha2::{Digest, Sha256},
};
use rand::{rng, Rng, RngCore};

#[derive(Debug)]
#[repr(C)]
struct PkOutput {
    pk_bytes: [u8; 33],
    b: u8,
}

#[derive(Debug)]
#[repr(C)]
struct Output {
    sig_valid: bool,
    output_hash: [u8; 32],
}

#[derive(Debug)]
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

static SALT: [u8; 32] = [
    175, 142, 86, 41, 61, 122, 186, 56, 50, 101, 187, 215, 124, 127, 14, 221, 109, 201, 110, 189,
    174, 1, 87, 170, 113, 193, 170, 115, 85, 51, 79, 172,
];

#[allow(unused)]
fn main_gen_pk() {
    let compiler = CompileOpts::new("gen-pk");
    let elf_bytes = compiler.build();

    let mut vm = TestVM::init(elf_bytes);
    let mut count = 0usize;
    while vm.is_exec() && count < 10_000_000 {
        vm.run();
        count += 1;
    }
    // println!("Total instructions: {}", count);

    let output_tape = vm.output_tape();
    let output: PkOutput = from_u8_slice(&output_tape);

    // Expected output
    let sk = SigningKey::from_slice(&SECRET_KEY);
    let pk_bytes = VerifyingKey::from(sk.unwrap()).to_sec1_bytes();

    assert_eq!(pk_bytes.iter().as_slice(), output.pk_bytes.as_slice());
}

fn gen_random_case() -> (bool, Input, [u8; 32]) {
    let mut rng = rng();

    // Generate random message
    let mut message: [u8; 64] = [0u8; 64];
    rng.fill_bytes(message.as_mut_slice());

    // Sign message
    let sk = SigningKey::from_slice(&SECRET_KEY).unwrap();
    let signature: Signature = sk.sign(message.as_slice());
    let mut signature_bytes: [u8; 64] = [0u8; 64];
    signature_bytes.copy_from_slice(signature.to_bytes().as_slice());

    // force signature to be invalid at random
    let mut signature_valid = true;
    if rng.random_bool(0.5) {
        rng.fill_bytes(message.as_mut_slice());
        signature_valid = false;
    }

    let input = Input {
        signature: signature_bytes,
        message,
    };

    // Produce sha256(mesage || salt)
    let mut hasher = Sha256::new();
    hasher.update(message.as_slice());
    hasher.update(SALT.as_slice());
    let output_hash: [u8; 32] = hasher.finalize().into();

    (signature_valid, input, output_hash)
}

fn test_main_guest() {
    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build();

    for _ in 0..10 {
        let mut vm = TestVM::init(elf_bytes.clone());

        let (is_sig_valid, random_input, output_hash) = gen_random_case();

        let input_tape = to_u8_slice(&random_input);
        vm.read_input_tape(input_tape);
        let mut count = 0usize;
        while vm.is_exec() && count < 10_000_000 {
            vm.run();
            count += 1;
        }
        // println!("Total instructions: {}", count);

        let output_tape = vm.output_tape();
        let output: Output = from_u8_slice(&output_tape);
        assert!(output.sig_valid == is_sig_valid);
        if is_sig_valid {
            assert!(output.output_hash == output_hash);
        } else {
            assert!(output.output_hash == [0u8; 32]);
        }
    }
}

fn main() {
    // main_gen_pk();
    test_main_guest();
}
