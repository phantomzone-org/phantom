use base2k::{Module, MODULETYPE};
use fhevm::{
    instructions::{Instruction, InstructionsParser},
    interpreter::Interpreter,
    parameters::{LOGN_LWE, LOGN_PBS},
};

fn main() {
    let n: usize = 1 << LOGN_LWE;
    let n_acc = 1 << LOGN_PBS;
    let module_lwe: Module = Module::new(n, MODULETYPE::FFT64);
    let module_pbs: Module = Module::new(n_acc, MODULETYPE::FFT64);

    let instructions: Vec<u32> = vec![0b00000000000100010000000110110011];

    let mut parser: InstructionsParser = InstructionsParser::new();
    instructions
        .iter()
        .for_each(|x| parser.add(Instruction(*x)));

    println!("{:?}", parser.imm);
    println!("{:?}", parser.get(0));

    let mut interpreter: Interpreter = Interpreter::new(&module_pbs, &module_lwe);

    interpreter.init_pc(&module_lwe);
    interpreter.init_instructions(parser);
    interpreter.init_registers(&REGISTERS);
    interpreter.init_memory(&MEMORY.to_vec());

    interpreter.step(&module_pbs, &module_lwe);

    interpreter.registers.print();
    println!();
    interpreter.memory.print();
}

static REGISTERS: [u32; 32] = [
    0x0000000, 0x0000001, 0x0000002, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000,
    0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000,
    0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000,
    0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000,
];

static MEMORY: [u32; 32] = [
    0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000,
    0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000,
    0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000,
    0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000, 0x0000000,
];
