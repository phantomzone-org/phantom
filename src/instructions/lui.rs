//! LUI: load upper immediate.
//!
//! # Format
//!
//! lui rd,imm
//!
//! # Description
//!
//! Build 32-bit constants and uses the U-type format.
//! LUI places the U-immediate value in the top 20 bits
//! of the destination register rd, filling in the lowest
//! 12 bits with zeros.
//!
//! # Implementation
//!
//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(imm[19:0] << 12)
//!

use super::{decomp, reconstruct, sext, Arithmetic};
use crate::parameters::{U20DECOMP, U32DECOMP};

pub struct Lui();

impl Arithmetic for Lui {
    fn apply(&self, imm: &[u32], _x_rs1: &[u32], _x_rs2: &[u32]) -> Vec<u32> {
        decomp(sext(reconstruct(imm, &U20DECOMP), 20), &U32DECOMP)
    }
}

#[cfg(test)]
use crate::instructions::{encode_0110111, Instructions};
#[test]
fn instruction_parsing() {
    // imm[31:12] | rd[11:7] | 01101 | 11
    let imm_19: u8 = 0b1100;
    let imm_15: u8 = 0b1011;
    let imm_11: u8 = 0b1010;
    let imm_7: u8 = 0b1001;
    let imm_3: u8 = 0b1000;
    let rs2: u8 = 0;
    let rs1: u8 = 0;
    let rd: u8 = 0b11011;
    let rd_w: u8 = 1;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let rv32: u32 = encode_0110111(imm_19, imm_15, imm_11, imm_7, imm_3, rd);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
