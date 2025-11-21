use compiler::{CompileOpts, Phantom};
use std::ptr;

fn to_u8_slice<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((v as *const T) as *const u8, core::mem::size_of::<T>()) }
}

fn from_u8_slice<T>(v: &[u8]) -> T {
    unsafe { ptr::read(v.as_ptr() as *const T) }.into()
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
// Repeat the input and output structs here.
#[repr(C)]
struct Output {
    evaluation: u32,
}

#[repr(C)]
struct Input {
    point: u32,
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

fn main() {
    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build("template");
    let pz = Phantom::from_elf(elf_bytes);

    // Set the number of cycles you want to run
    // Allow enough cycles for the guest program to reach the point where it
    // writes the output buffer before hitting the busy loop at the end.
    let max_cycles = 700;

    // Provide sample Inputs
    let input = Input {
        point: 123,
    };

    // Running the encrypted VM
    let input_tape = to_u8_slice(&input);
    println!("Initializing Phantom...");    
    let mut enc_vm = pz.encrypted_vm(input_tape, max_cycles);
    println!("Phantom initialized!");

    println!("Executing Encrypted Cycles...");
    enc_vm.execute();
    println!("Finished Executing Encrypted Cycles!");
    
    let encrypted_vm_output_tape = enc_vm.output_tape();

    // Running the cleartext VM for comparison and testing purposes
    let mut vm = pz.test_vm(max_cycles);

    vm.read_input_tape(input_tape);
    vm.execute();
    let output_tape = vm.output_tape();

    assert_eq!(output_tape, encrypted_vm_output_tape);
    println!("Encrypted Tape and Test VM Tape are equal");
    println!("output_tape={:?}", output_tape);

    // Below is for testing purposes only
    // Comparing Phantom's output with the expected behaviour

    fn expected_output(input: Input) -> Output {
        let coefficients = vec![123, 456, 789, 12, 3456, 7, 89];
        let mut evaluation = 0;
        let mut pow_point = 1;
        for coeff in coefficients.iter() {
            evaluation += coeff * pow_point;
            pow_point *= input.point;
        }
        Output {
            evaluation: evaluation,
        }
    }

    let have_output = from_u8_slice::<Output>(&output_tape);
    let have_evaluation = have_output.evaluation;

    let want_output = expected_output(input);
    let want_evaluation = want_output.evaluation;

    assert_eq!(have_evaluation, want_evaluation);
}
