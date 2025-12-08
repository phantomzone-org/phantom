use compiler::{CompileOpts, Phantom};
use std::env;
use std::ptr;

fn to_u8_slice<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((v as *const T) as *const u8, core::mem::size_of::<T>()) }
}

fn from_u8_slice<T>(v: &[u8]) -> T {
    unsafe { ptr::read(v.as_ptr() as *const T) }.into()
}

#[repr(C)]
struct Output {
    is_match: bool,
}

const MAX_LENGTH: usize = 20;
#[repr(C)]
struct UpperBoundedString {
    characters: [u8; MAX_LENGTH],
}

impl UpperBoundedString {
    fn new(input_string: &str) -> Self {
        if input_string.len() > MAX_LENGTH {
            panic!("String is too long");
        }
        let mut characters = [0u8; MAX_LENGTH];
        for (i, &byte) in input_string.as_bytes().iter().enumerate() {
            characters[i] = byte;
        }
        Self { characters }
    }

    fn eq(&self, other: &Self) -> bool {
        for i in 0..MAX_LENGTH {
            if self.characters[i] != other.characters[i] {
                return false;
            }
        }
        true
    }
}

#[repr(C)]
struct Input {
    input_string: UpperBoundedString,
}

fn main() {
    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build("string-match");
    let pz = Phantom::from_elf(elf_bytes);

    let max_cycles = env::var("MAX_CYCLES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);

    let input = Input {
        input_string: UpperBoundedString::new("not-mississippi"),
    };

    // Running the encrypted VM
    println!("Initializing Phantom...");
    let mut enc_vm = pz.encrypted_vm(to_u8_slice(&input), max_cycles);
    println!("Phantom initialized!");

    println!("Executing Encrypted Cycles...");
    enc_vm.execute();
    println!("Finished Executing Encrypted Cycles!");

    let encrypted_vm_output_tape = enc_vm.output_tape();

    // Running the cleartext VM for comparison and testing purposes
    let mut vm = pz.test_vm(max_cycles);
    vm.read_input_tape(to_u8_slice(&input));
    vm.execute();
    let output_tape = vm.output_tape();

    assert_eq!(output_tape, encrypted_vm_output_tape);
    println!("Encrypted Tape and Test VM Tape are equal");
    println!("output_tape={:?}", output_tape);

    fn expected_output(input: Input) -> Output {
        let hidden_string = UpperBoundedString::new("mississippi");
        let is_match = input.input_string.eq(&hidden_string);
        Output { is_match: is_match }
    }

    let have_output = from_u8_slice::<Output>(&output_tape);
    let have_is_match = have_output.is_match;

    let want_output = expected_output(input);
    let want_is_match = want_output.is_match;

    assert_eq!(have_is_match, want_is_match);
    println!("is_match={}", have_is_match);
}
