use compiler::{interpreter::Phantom, CompileOpts};
use core::ptr;
use rand::{thread_rng, RngCore};
use sha2::{Digest, Sha256};

#[repr(C)]
#[derive(Default)]
struct Output {
    hash_value: [u8; 32],
}

#[repr(C)]
struct Input {
    message: [u8; 32],
}

#[no_mangle]
static SALT: [u8; 32] = [
    175, 142, 86, 41, 61, 122, 186, 56, 50, 101, 187, 215, 124, 127, 14, 221, 109, 201, 110, 189,
    174, 1, 87, 170, 113, 193, 170, 115, 85, 51, 79, 172,
];

fn to_u8_slice<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((v as *const T) as *const u8, core::mem::size_of::<T>()) }
}

fn from_u8_slice<T>(v: &[u8]) -> T {
    unsafe { ptr::read(v.as_ptr() as *const T) }.into()
}

fn main() {
    // compile guest/main.rs to risc-v target
    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build();

    // initialise phantom
    //
    // the `init` function transforms the risc-v binary
    // into suitable format for execution on fhe-vm
    let pz = Phantom::init(elf_bytes);

    // prepare input
    let mut input_message = [0u8; 32];
    thread_rng().fill_bytes(&mut input_message);
    let input = Input {
        message: input_message,
    };
    // input is provided as buffer of bytes
    let input_buffer = to_u8_slice(&input);

    // encrypted vm runs for `max_cycles`
    //
    // this is necessary because vm cannot know when to halt
    let max_cycles = 8725;

    // create an encrypted vm instance
    let mut enc_vm = pz.encrypted_vm(input_buffer, max_cycles);

    // execute the vm
    enc_vm.execute();

    // read output
    let output_tape = enc_vm.output_tape();

    // TEST VM //
    let mut test_vm = pz.test_vm();
    test_vm.read_input_tape(to_u8_slice(&input));
    let mut count = 0;
    while test_vm.is_exec() && count < max_cycles {
        test_vm.run();
        count += 1;
    }

    // Check equivalance of encrypted vm output tape with test vm output tape
    let test_output_tape = test_vm.output_tape();
    assert_eq!(output_tape, test_output_tape);
    // End TEST VM //

    // Check output's correctness
    let mut hasher = Sha256::new();
    hasher.update(&input_message);
    hasher.update(SALT);
    let expected_hash = hasher.finalize().to_vec();
    let output = from_u8_slice::<Output>(&test_output_tape);
    assert!(
        &output.hash_value == expected_hash.as_slice(),
        "Expected {:?}, but got {:?}",
        expected_hash.as_slice(),
        &output.hash_value,
    );
}
