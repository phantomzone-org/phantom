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

use crate::instructions::{decompose, reconstruct};

pub struct Auipc();

impl Auipc {
    pub fn apply(imm: &[u32; 8], _x_rs1: &[u32; 8], _x_rs2: &[u32; 8], pc: &[u32; 8]) -> [u32; 8] {
        let imm_u32: u32 = reconstruct(imm);
        let pc_u32: u32 = reconstruct(pc);
        decompose(pc_u32.wrapping_add(imm_u32 << 12))
    }
}
