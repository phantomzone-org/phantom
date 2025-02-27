//! addi: add immediate
//!
//! # Description
//!
//! Adds the sign-extended 12-bit immediate to register rs1.
//! Arithmetic overflow is ignored and the result is simply the
//! low XLEN bits of the result. ADDI rd, rs1, 0 is used to
//! implement the MV rd, rs1 assembler pseudo-instruction.
//!
//! # Implementation
//!
//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] + sext(imm[11:0])

use super::{decomp, reconstruct, sext, Arithmetic};
use crate::parameters::{U12DECOMP, U32DECOMP};

pub struct Addi();

impl Arithmetic for Addi {
    fn apply(&self, imm: &[u32], x_rs1: &[u32], _x_rs2: &[u32]) -> Vec<u32> {
        let imm_u32: u32 = sext(reconstruct(imm, &U12DECOMP), 12);
        let x_rs1_u32: u32 = reconstruct(x_rs1, &U32DECOMP);
        decomp(x_rs1_u32.wrapping_add(imm_u32), &U32DECOMP)
    }
}

#[cfg(test)]
use crate::instructions::{encode_0010011, Instructions};
#[test]
fn instruction_parsing() {
    // imm[31:20] | rs1[19:15] | 000 | rd[11:7] | 00100 | 11
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0b1010;
    let imm_7: u8 = 0b1001;
    let imm_3: u8 = 0b1000;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let rd_w: u8 = 3;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let rv32: u32 = encode_0010011(imm_11, imm_7, imm_3, rs1, 0b000, rd);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
