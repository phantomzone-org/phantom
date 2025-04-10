use compiler::{interpreter::Phantom, CompileOpts};
use core::ptr;

#[repr(C)]
#[derive(Default)]
struct Output {
    value: u32,
}

#[repr(C)]
struct Input {
    value: u32,
}

fn to_u8_slice<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((v as *const T) as *const u8, core::mem::size_of::<T>()) }
}

fn from_u8_slice<T>(v: &[u8]) -> T {
    unsafe { ptr::read(v.as_ptr() as *const T) }.into()
}

fn main() {
    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build();
    // Initialise Phantom
    let pz = Phantom::init(elf_bytes);

    let input = Input { value: 2 };
    let mut enc_vm = pz.encrypted_vm(to_u8_slice(&input), 200);
    enc_vm.execute();
    println!("{:?}", enc_vm.output_tape());
    enc_vm.print_debug();
    // let output_tape =

    // let mut test_vm = pz.test_vm();

    // let input = Input { value: 2 };
    // test_vm.read_input_tape(to_u8_slice(&input));

    // let mut count = 0;
    // while test_vm.is_exec() && count < 200 {
    //     test_vm.run();
    //     count += 1;
    // }
    // dbg!(count);
    // let output_tape = test_vm.output_tape();
    // let output = from_u8_slice::<Output>(&output_tape);
    // dbg!(output.value);
}
