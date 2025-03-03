//! slli
//!
//! # Format
//!
//! srai rd,rs1,shamt
//!
//! # Description
//!
//! Performs arithmetic right shift on the value in register
//! rs1 by the shift amount held in the lower 5 bits of the
//! immediate In RV64, bit-25 is used to shamt[5].
//!
//! # Implementation
//!
//! x[rd] = x[rs1] >>s shamt

#[cfg(test)]
use crate::instructions::{Instruction, Instructions, OpID};
#[test]
fn instruction_parsing() {
    // 01000 | 00 | shamt[24:20] | rs1[19:15] | 101 | rd[11:7] | 00100 | 11
    let op_code: u8 = 0b0010011;
    let funct3: u8 = 0b101;
    let funct7: u8 = 0b0100000;
    let shamt: u8 = 0b10010;
    let imm_31: u8 = 0;
    let imm_27: u8 = 0;
    let imm_23: u8 = 0;
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0;
    let imm_7: u8 = (shamt >> 1) & 1;
    let imm_3: u8 = shamt & 0xF;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let (rd_w, mem_w, pc_w) = OpID::SRAI;

    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_funct3(funct3);
    instruction.encode_funct7(funct7);
    instruction.encode_shamt(shamt);
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
