use std::ptr;

use compiler::{interpreter::TestVM, CompileOpts};
use rand::{rng, Rng};

fn to_u8_slice<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((v as *const T) as *const u8, core::mem::size_of::<T>()) }
}

fn from_u8_slice<T>(v: &[u8]) -> T {
    unsafe { ptr::read(v.as_ptr() as *const T) }.into()
}

#[derive(Debug, Clone)]
#[repr(C)]
struct Pool {
    t0: u32,
    t1: u32,
}

#[derive(Debug)]
#[repr(C)]
struct Output {
    pool: Pool,
    out0: u32,
    out1: u32,
}

#[derive(Debug)]
#[repr(C)]
struct Input {
    pool: Pool,
    inp0: u32,
    inp1: u32,
}

fn main() {
    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build();
    // let mut vm = TestVM::init(elf_bytes);

    let mut rng = rng();
    let mut pool = Pool {
        t0: rng.random(),
        t1: rng.random(),
    };
    for _ in 0..100 {
        let input = Input {
            pool: pool.clone(),
            inp0: rng.random(),
            inp1: rng.random(),
        };

        // Init -> read input tape -> run -> read output tape
        let mut vm = TestVM::init(elf_bytes.clone());
        let input_tape = to_u8_slice(&input);
        vm.read_input_tape(input_tape);
        while vm.is_exec() {
            vm.run();
        }
        let output_tape = vm.output_tape();

        // Check output
        let output = from_u8_slice::<Output>(&output_tape);
        if pool.t0 > input.inp0 {
            assert!(output.out0 == input.inp0);
            assert!(output.pool.t0 == input.pool.t0 - input.inp0);
        }
        if pool.t1 > input.inp1 {
            assert!(output.out1 == input.inp1);
            assert!(output.pool.t1 == input.pool.t1 - input.inp1);
        }

        pool = output.pool;
    }
}
