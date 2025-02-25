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
