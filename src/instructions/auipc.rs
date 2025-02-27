//! AUIPC: add upper immediate to pc
//!
//! #Format
//!
//! auipc rd,imm
//!
//! # Description
//! Build pc-relative addresses and uses the U-type format.
//! AUIPC forms a 32-bit offset from the 20-bit U-immediate, filling in
//! the lowest 12 bits with zeros, adds this offset to the pc, then places
//! the result in register rd.
//!
//! # Implementation
//!
//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + sext(imm[19:0] << 12)

use super::{decomp, reconstruct, sext, PcUpdates};
use crate::parameters::{U12DECOMP, U32DECOMP};

pub struct Auipc();

impl PcUpdates for Auipc {
    fn apply(
        &self,
        imm: &[u32],
        _x_rs1: &[u32],
        _x_rs2: &[u32],
        pc: &[u32],
    ) -> (Vec<u32>, Vec<u32>) {
        let imm_u32: u32 = sext(reconstruct(imm, &U12DECOMP), 12);
        let pc_u32: u32 = reconstruct(pc, &U32DECOMP);
        (decomp(pc_u32 + (imm_u32 << 12), &U32DECOMP), pc.into())
    }
}

#[cfg(test)]
use crate::instructions::{encode_0010111, Instructions};
#[test]
fn instruction_parsing() {
    // imm[31:12] | rd[11:7] | 00101 | 11
    let imm_19: u8 = 0b1100;
    let imm_15: u8 = 0b1011;
    let imm_11: u8 = 0b1010;
    let imm_7: u8 = 0b1001;
    let imm_3: u8 = 0b1000;
    let rs2: u8 = 0;
    let rs1: u8 = 0;
    let rd: u8 = 0b11011;
    let rd_w: u8 = 2;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let rv32: u32 = encode_0010111(imm_19, imm_15, imm_11, imm_7, imm_3, rd);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
