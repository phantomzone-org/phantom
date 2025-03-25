use compiler::{interpreter::Phantom, CompileOpts};
use core::ptr;
use k256::{
    ecdsa::{
        signature::{Signer, Verifier},
        Signature, SigningKey, VerifyingKey,
    },
    sha2::{Digest, Sha256},
};
use rand::{thread_rng, Rng, RngCore};

#[derive(Debug)]
#[repr(C)]
struct Output {
    signature: [u8; 64],
    anon_id: [u8; 32],
}

#[derive(Debug)]
#[repr(C)]
struct Input {
    signature: [u8; 64],
    unique_id: [u8; 64],
}

fn to_u8_slice<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((v as *const T) as *const u8, core::mem::size_of::<T>()) }
}

fn from_u8_slice<T>(v: &[u8]) -> T {
    unsafe { ptr::read(v.as_ptr() as *const T) }.into()
}

/// Secret key of issuing authority of the sybil resistant identify system
static SECRET_KEY_ISSUING_AUTH: [u8; 32] = [
    112, 241, 22, 238, 183, 253, 29, 115, 104, 106, 220, 187, 201, 91, 199, 254, 63, 240, 149, 71,
    141, 43, 156, 246, 48, 169, 197, 254, 208, 40, 218, 192,
];

/// Public key corresponding to `SECRET_KEY_PROGRAM` to verify signature on output anonymous identifier
static PUBLIC_KEY_PROGRAM: [u8; 33] = [
    3, 128, 202, 66, 109, 176, 158, 195, 170, 63, 235, 157, 169, 240, 223, 241, 219, 176, 69, 135,
    131, 122, 224, 218, 124, 199, 18, 38, 235, 56, 237, 159, 192,
];

/// Secret SALT harcoded inside risc-v program
static SALT: [u8; 32] = [
    175, 142, 86, 41, 61, 122, 186, 56, 50, 101, 187, 215, 124, 127, 14, 221, 109, 201, 110, 189,
    174, 1, 87, 170, 113, 193, 170, 115, 85, 51, 79, 172,
];

/// Generates and prints random secret key, public key pair
#[allow(unused)]
fn gen_random_key_pair() {
    let mut rng = thread_rng();
    let sk = SigningKey::random(&mut rng);
    let pk = sk.verifying_key();

    println!("SK={:?} \n PK={:?}", sk.to_bytes(), pk.to_sec1_bytes());
}

/// Generates a random sybil-resistant identity for testing purposes
fn gen_random_case() -> (bool, Input, [u8; 32]) {
    let mut rng = thread_rng();

    // Simulate sybil-resistant issuing authority

    // Generate random unique identifier
    let mut unique_identifier: [u8; 64] = [0u8; 64];
    rng.fill_bytes(unique_identifier.as_mut_slice());

    // Issue identity: sign the unique identifier using issuing authority's secret `SECRET_KEY_ISSUING_AUTH`
    let sk = SigningKey::from_slice(&SECRET_KEY_ISSUING_AUTH).unwrap();
    let signature: Signature = sk.sign(unique_identifier.as_slice());
    let mut signature_bytes: [u8; 64] = [0u8; 64];
    signature_bytes.copy_from_slice(signature.to_bytes().as_slice());

    // force signature to be invalid at random for testing purposes. Doing so confirms that
    // encrypted `guest` program does not issues anonymous identifier if signature on unique identifier is invalid.
    let mut signature_valid = true;
    if rng.gen_bool(0.5) {
        rng.fill_bytes(unique_identifier.as_mut_slice());
        signature_valid = false;
    }

    let input = Input {
        signature: signature_bytes,
        unique_id: unique_identifier,
    };

    // the expected anonymous identifier sha256(unique_identifier || SALT)
    let mut hasher = Sha256::new();
    hasher.update(unique_identifier.as_slice());
    hasher.update(SALT.as_slice());
    let output_hash: [u8; 32] = hasher.finalize().into();

    (signature_valid, input, output_hash)
}

/// Compiles and tests the "guest" program on random inputs
fn test_main_guest() {
    // Compiler risc-v guest program
    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build();
    // Initialise Phantom
    let pz = Phantom::init(elf_bytes);

    for _ in 0..10 {
        let mut vm = pz.test_vm();

        let (is_sig_valid, random_input, output_hash) = gen_random_case();

        // Load inputs to the VM
        let input_tape = to_u8_slice(&random_input);
        vm.read_input_tape(input_tape);
        let mut count = 0usize;
        while vm.is_exec() && count < 20_000_000 {
            vm.run();
            count += 1;
        }
        // println!("Total instructions: {}", count);

        // read outputs of the VM
        let output_tape = vm.output_tape();
        let output: Output = from_u8_slice(&output_tape);
        if is_sig_valid {
            assert!(
                output.anon_id == output_hash,
                "Expected anonymous identifier={:?} but got={:?}",
                output_hash,
                output.anon_id
            );

            // verify signature
            let pk_program = VerifyingKey::from_sec1_bytes(&PUBLIC_KEY_PROGRAM).unwrap();
            assert!(pk_program
                .verify(
                    output.anon_id.as_slice(),
                    &Signature::from_slice(output.signature.as_slice()).unwrap()
                )
                .is_ok())
        } else {
            assert!(output.anon_id == [0u8; 32]);
            assert!(output.signature == [0u8; 64]);
        }
    }
}

fn main() {
    // gen_random_key_pair();
    test_main_guest();
}
