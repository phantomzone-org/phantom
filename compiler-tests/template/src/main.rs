use compiler::{CompileOpts, Phantom};
use std::{ptr};

fn to_u8_slice<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((v as *const T) as *const u8, core::mem::size_of::<T>()) }
}

fn from_u8_slice<T>(v: &[u8]) -> T {
    unsafe { ptr::read(v.as_ptr() as *const T) }.into()
}

// TODO: Repeat the input and output structs here
#[repr(C)]
struct Output {
    volume: f32,
}

#[repr(C)]
struct Input {
    radius: f32,
    height: f32,
}

// For testing purposes only
pub fn expected_output(input: Input) -> Output {
    Output {
        volume: core::f64::consts::PI as f32 * input.radius * input.radius * input.height
    }
}

fn main() {
    let threads = 16;

    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build();
    let pz = Phantom::from_elf(elf_bytes);

    // Allow enough cycles for the guest program to reach the point where it
    // writes the output buffer before hitting the busy loop at the end.
    let max_cycles = 2000;

    let input = Input {
        radius: 4.1,
        height: 5.1,
    };

    // Running the encrypted VM
    let mut enc_vm = pz.encrypted_vm::<true>(to_u8_slice(&input), max_cycles);
    enc_vm.execute(threads);
    let encrypted_vm_output_tape = enc_vm.output_tape();

    // Running the cleartext VM for comparison
    let mut vm = pz.test_vm(max_cycles);
    vm.read_input_tape(to_u8_slice(&input));
    vm.execute();
    let output_tape = vm.output_tape();

    assert_eq!(output_tape, encrypted_vm_output_tape);
    println!("Encrypted Tape and Test VM Tape are equal");
    println!("output_tape={:?}", output_tape);

    let have_output = from_u8_slice::<Output>(&output_tape);
    let have_volume = have_output.volume;

    let want_output = expected_output(input);
    let want_volume = want_output.volume;
    
    assert_eq!(have_volume, want_volume);
}
