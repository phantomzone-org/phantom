//! JALR
//!
//! # Format
//!
//! jalr rd,rs1,offset
//!
//! # Description
//!
//! Jump to address and place return address in rd.
//!
//! Implementation
//!
//! jalr   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | t = pc + 4; pc = (x[rs1] + sext(imm[11:0])) & ~1; x[rd] = t

use super::{decomp, reconstruct, PcUpdates};

pub struct Jalr();

impl PcUpdates for Jalr {
    fn apply(
        &self,
        imm: &[u8; 8],
        x_rs1: &[u8; 8],
        _x_rs2: &[u8; 8],
        pc: &[u8; 8],
    ) -> ([u8; 8], [u8; 8]) {
        (
            decomp(reconstruct(pc) + 4),
            decomp(reconstruct(x_rs1).wrapping_add(reconstruct(imm)) & !1),
        )
    }
}

#[cfg(test)]
use crate::instructions::{Instruction, Instructions, OpID};
#[test]
fn instruction_parsing() {
    // imm[31:20] | rs1[19:15] | 000 | rd[11:7] | 1100111
    let op_code: u8 = 0b1100111;
    let funct3: u8 = 0b000;
    let imm_31: u8 = 0;
    let imm_27: u8 = 0;
    let imm_23: u8 = 0;
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0b1010;
    let imm_7: u8 = 0b1001;
    let imm_3: u8 = 0b1000;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let (rd_w, mem_w, pc_w) = OpID::JALR;

    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_immediate((imm_11 as u32) << 8 | (imm_7 as u32) << 4 | imm_3 as u32);
    instruction.encode_funct3(funct3);
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
