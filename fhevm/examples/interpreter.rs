use base2k::{
    alloc_aligned, alloc_aligned_u8, Encoding, Module, VecZnx, VecZnxBig, VecZnxBigOps, VecZnxDft,
    VecZnxDftOps, VecZnxOps, VmpPMat, VmpPMatOps, MODULETYPE,
};
use fhevm::{
    circuit_bootstrapping::{circuit_bootstrap_tmp_bytes, CircuitBootstrapper},
    instructions::{Instruction, InstructionsParser},
    interpreter::{next_tmp_bytes, Interpreter},
    parameters::{LOGBASE2K, LOGN_LWE, LOGN_PBS},
};
use itertools::izip;

fn main() {
    let n: usize = 1 << LOGN_LWE;
    let n_acc = 1 << LOGN_PBS;
    let module_lwe: Module = Module::new(n, MODULETYPE::FFT64);
    let module_pbs: Module = Module::new(n_acc, MODULETYPE::FFT64);

    let mut tmp_bytes: Vec<u8> = alloc_aligned(next_tmp_bytes(&module_pbs, &module_lwe));

    let instructions: Vec<u32> = vec![0b00000000000100010000000110110011];

    let mut parser: InstructionsParser = InstructionsParser::new();
    instructions
        .iter()
        .for_each(|x| parser.add(Instruction(*x)));

    println!("{:?}", parser.imm);
    println!("{:?}", parser.get(0));

    let mut interpreter: Interpreter = Interpreter::new(&module_lwe);

    let registers: [u32; 32] = [
        0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let memory: Vec<u32> = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];

    interpreter.init_pc(&module_lwe);
    interpreter.init_instructions(parser);
    interpreter.init_registers(&registers);
    interpreter.init_memory(&memory);

    interpreter.next(&module_pbs, &module_lwe, &mut tmp_bytes);

    interpreter.register.print();
    println!();
    interpreter.memory.print();
}
