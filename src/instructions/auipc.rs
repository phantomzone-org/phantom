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
