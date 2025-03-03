#[cfg(test)]
use crate::instructions::{Instruction, Instructions, OpID};
#[test]
fn instruction_parsing() {
    // 00000 | 00 | rs2[24:20] | rs1[19:15] | 100 | rd[11:7] | 0110011
    let op_code: u8 = 0b0110011;
    let funct3: u8 = 0b100;
    let funct7: u8 = 0b0000000;
    let imm_31: u8 = 0;
    let imm_27: u8 = 0;
    let imm_23: u8 = 0;
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0;
    let imm_7: u8 = 0;
    let imm_3: u8 = 0;
    let rs2: u8 = 0b11011;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let (rd_w, mem_w, pc_w) = OpID::XOR;

    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_funct3(funct3);
    instruction.encode_funct7(funct7);
    instruction.encode_rs2(rs2);
    instruction.encode_rs1(rs1);
    instruction.encode_rd(rd);

    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_31, imm_27, imm_23, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w,
        pc_w,
    );
}
