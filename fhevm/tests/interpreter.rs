use fhevm::{
    instructions::sext,
    instructions::{Instruction, InstructionsParser},
    interpreter::Interpreter,
    parameters::Parameters,
};

const MEMORY_SIZE: usize = 64;
const REGISTERS_SIZE: usize = 32;

fn setup(instruction: u32) -> (Parameters, Interpreter) {
    let params: Parameters = Parameters::new();

    let instructions: Vec<u32> = vec![instruction];

    let mut parser: InstructionsParser = InstructionsParser::new();
    instructions
        .iter()
        .for_each(|x| parser.add(Instruction(*x)));

    let mut interpreter: Interpreter = Interpreter::new(&params);

    interpreter.init_pc(&params);
    interpreter.init_instructions(parser);
    interpreter.init_registers(&REGISTERS.to_vec());
    interpreter.init_memory(&MEMORY.to_vec());

    interpreter.cycle(&params);

    (params, interpreter)
}

#[test]
fn interpreter_arithmetic_ops() {
    // 00000 | 00 | rs2[24:20] | rs1[19:15] | funct3 | rd[11:7] |
    let funct3: u8 = 0b000;
    let funct7: u8 = 0b0000000;
    let op_code: u8 = 0b0110011;
    let rs2: u8 = 0b00010;
    let rs1: u8 = 0b00001;
    let rd: u8 = 0b00110;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.set_funct3(funct3);
    instruction.set_funct7(funct7);
    instruction.set_rs2(rs2);
    instruction.set_rs1(rs1);
    instruction.set_rd(rd);

    let (params, interpreter) = setup(instruction.get());

    let pc_want: u32 = 1;

    let mut memory_want: [u32; MEMORY_SIZE] = [0; MEMORY_SIZE];
    memory_want.copy_from_slice(&MEMORY);

    let mut registers_want: [u32; REGISTERS_SIZE] = [0; REGISTERS_SIZE];
    registers_want.copy_from_slice(&REGISTERS);
    registers_want[rd as usize] = REGISTERS[rs1 as usize].wrapping_add(REGISTERS[rs2 as usize]);

    assert_eq!(
        interpreter.addr_pc.debug_as_u32(params.module_lwe()),
        pc_want
    );
    assert_eq!(
        interpreter.memory.debug_as_u32()[..MEMORY_SIZE],
        memory_want
    );
    assert_eq!(
        interpreter.registers.debug_as_u32()[..REGISTERS_SIZE],
        registers_want
    );
}

#[test]
fn interpreter_store_op() {
    let op_code: u8 = 0b0100011;
    let funct3: u8 = 0b001;
    let imm: u32 = 0x5;
    let rs2: u8 = 31;
    let rs1: u8 = 0b00001;
    let mut instruction: Instruction = Instruction::new(op_code as u32);

    instruction.set_immediate(imm);
    instruction.set_funct3(funct3);
    instruction.set_rs2(rs2);
    instruction.set_rs1(rs1);

    let (params, interpreter) = setup(instruction.get());

    let pc_want: u32 = 1;

    let mut memory_want: [u32; MEMORY_SIZE] = [0; MEMORY_SIZE];
    memory_want.copy_from_slice(&MEMORY);

    let address: usize = REGISTERS[rs1 as usize].wrapping_add(imm as u32) as usize;
    let address_aligned: usize = address >> 2;
    let address_offset: usize = address & 0x3;

    let mut loaded: u32 = memory_want[address_aligned];

    loaded &= !(0xFFFF << (address_offset << 3));
    loaded |= (REGISTERS[rs2 as usize] & 0xFFFF) << (address_offset << 3);
    memory_want[address_aligned] = loaded;

    let mut registers_want: [u32; REGISTERS_SIZE] = [0; REGISTERS_SIZE];
    registers_want.copy_from_slice(&REGISTERS);

    assert_eq!(
        interpreter.addr_pc.debug_as_u32(params.module_lwe()),
        pc_want
    );
    assert_eq!(
        interpreter.memory.debug_as_u32()[..MEMORY_SIZE],
        memory_want
    );
    assert_eq!(
        interpreter.registers.debug_as_u32()[..REGISTERS_SIZE],
        registers_want
    );
}

#[test]
fn interpreter_load_op() {
    let op_code: u8 = 0b0000011;
    let funct3: u8 = 0b001;
    let imm: u32 = 252;
    let rs1: u8 = 0b00010; // R[2] = 2
    let rd: u8 = 0b00011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);

    instruction.set_immediate(imm);
    instruction.set_funct3(funct3);
    instruction.set_rs1(rs1);
    instruction.set_rd(rd);

    instruction.print();

    let (params, interpreter) = setup(instruction.get());

    let pc_want: u32 = 1;

    let mut memory_want: [u32; MEMORY_SIZE] = [0; MEMORY_SIZE];
    memory_want.copy_from_slice(&MEMORY);

    let mut registers_want: [u32; REGISTERS_SIZE] = [0; REGISTERS_SIZE];
    registers_want.copy_from_slice(&REGISTERS);

    let address: usize = REGISTERS[rs1 as usize].wrapping_add(imm as u32) as usize;
    let address_aligned: usize = address >> 2;
    let address_offset: usize = address & 0x3;

    let loaded: u32 = sext(memory_want[address_aligned] >> (address_offset << 3), 15);
    registers_want[rd as usize] = loaded;

    assert_eq!(
        interpreter.addr_pc.debug_as_u32(params.module_lwe()),
        pc_want
    );
    assert_eq!(
        interpreter.memory.debug_as_u32()[..MEMORY_SIZE],
        memory_want
    );
    assert_eq!(
        interpreter.registers.debug_as_u32()[..REGISTERS_SIZE],
        registers_want
    );
}

#[test]
fn interpreter_pc_ops() {
    // JALR
    let funct3: u8 = 0b000;
    let op_code: u8 = 0b1100111;
    let imm: u32 = 0x007;
    let rs1: u8 = 0b00001;
    let rd: u8 = 0b00110;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.set_funct3(funct3);
    instruction.set_immediate(imm);
    instruction.set_rs1(rs1);
    instruction.set_rd(rd);

    let (params, interpreter) = setup(instruction.get());

    let pc_want: u32 = REGISTERS[rs1 as usize].wrapping_add(sext(imm, 12)) & !1;

    let mut memory_want: [u32; MEMORY_SIZE] = [0; MEMORY_SIZE];
    memory_want.copy_from_slice(&MEMORY);

    let mut registers_want: [u32; REGISTERS_SIZE] = [0; REGISTERS_SIZE];
    registers_want.copy_from_slice(&REGISTERS);
    registers_want[rd as usize] = 4;

    assert_eq!(
        interpreter.addr_pc.debug_as_u32(params.module_lwe()) << 2,
        pc_want
    );
    assert_eq!(
        interpreter.memory.debug_as_u32()[..MEMORY_SIZE],
        memory_want
    );
    assert_eq!(
        interpreter.registers.debug_as_u32()[..REGISTERS_SIZE],
        registers_want
    );
}

// 128 bytes
static REGISTERS: [u32; REGISTERS_SIZE] = [
    0x00000000, 0x00000001, 0x00000002, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0xAABBCCDD,
];

// 256 bytes
static MEMORY: [u32; MEMORY_SIZE] = [
    0x00ABCDEF, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0xAABBCCDD,
];
