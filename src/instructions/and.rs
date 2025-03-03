//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] & x[rs2]

use super::{decomp, reconstruct, Arithmetic};

pub struct And();

impl Arithmetic for And {
    fn apply(&self, _imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8] {
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        let x_rs2_u32: u32 = reconstruct(x_rs2);
        decomp(x_rs1_u32 & x_rs2_u32)
    }
}

#[cfg(test)]
use crate::instructions::{Instruction, Instructions, OpID};
#[test]
fn instruction_parsing() {
    // 00000 | 00 | rs2[24:20] | rs1[19:15] | 111 | rd[11:7] | 0110011
    let op_code: u8 = 0b0110011;
    let funct3: u8 = 0b111;
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
    let (rd_w, mem_w, pc_w) = OpID::AND;

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
