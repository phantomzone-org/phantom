use fhevm::{
    instructions::{Instruction, InstructionsParser},
    interpreter::Interpreter,
    parameters::Parameters,
};

fn main() {
    let params: Parameters = Parameters::new();

    let instructions: Vec<u32> = vec![0b00000000000100010000000110110011];

    let mut parser: InstructionsParser = InstructionsParser::new();
    instructions
        .iter()
        .for_each(|x| parser.add(Instruction(*x)));

    println!("{:?}", parser.imm);
    println!("{:?}", parser.get(0));

    let mut interpreter: Interpreter = Interpreter::new(&params);

    interpreter.init_pc(&params);
    interpreter.init_instructions(parser);
    interpreter.init_registers(&REGISTERS.to_vec());
    interpreter.init_memory(&MEMORY.to_vec());

    interpreter.step(&params);

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
