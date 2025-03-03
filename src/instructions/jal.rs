//! JAL
//!
//! # Format
//!
//! jal rd,offset
//!
//! # Description
//!
//! Jump to address and place return address in rd.
//!
//! Implementation
//!
//! jal    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + 4; pc += sext(imm[19:0])

use super::{decomp, reconstruct, PcUpdates};

pub struct Jal();

impl PcUpdates for Jal {
    fn apply(
        &self,
        imm: &[u8; 8],
        _x_rs1: &[u8; 8],
        _x_rs2: &[u8; 8],
        pc: &[u8; 8],
    ) -> ([u8; 8], [u8; 8]) {
        (
            decomp(reconstruct(pc) + 4),
            decomp(reconstruct(pc).wrapping_add(reconstruct(imm))),
        )
    }
}

#[cfg(test)]
use crate::instructions::{Instruction, Instructions, OpID};
#[test]
fn instruction_parsing() {
    // imm[20|10:1|11|19:12] | rd[11:7] | 1101111
    let op_code: u8 = 0b1101111;
    let imm_31: u8 = 0;
    let imm_27: u8 = 0;
    let imm_23: u8 = 0b0001;
    let imm_19: u8 = 0b1001;
    let imm_15: u8 = 0b1010;
    let imm_11: u8 = 0b1010;
    let imm_7: u8 = 0b1001;
    let imm_3: u8 = 0b1000;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let (rd_w, mem_w, pc_w) = OpID::JAL;

    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_immediate(
        (imm_31 as u32) << 28
            | (imm_27 as u32) << 24
            | (imm_23 as u32) << 20
            | (imm_19 as u32) << 16
            | (imm_15 as u32) << 12
            | (imm_11 as u32) << 8
            | (imm_7 as u32) << 4
            | imm_3 as u32,
    );
    instruction.encode_rd(rd);

    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_31, imm_27, imm_23, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w,
        pc_w,
    );
}
