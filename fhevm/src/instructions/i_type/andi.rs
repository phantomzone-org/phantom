//! ori
//!
//! # Format
//!
//! andi rd,rs1,imm
//!
//! # Description
//!
//! Performs bitwise AND on register rs1 and the sign-
//! extended 12-bit immediate and place the result in rd.
//!
//! # Implementation
//!
//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] & sext(imm[11:0])
//!

use crate::instructions::{decompose, reconstruct};

pub struct Andi();

impl Andi {
    pub fn apply(imm: &[u8; 8], x_rs1: &[u8; 8], _x_rs2: &[u8; 8]) -> [u8; 8] {
        let imm_u32: u32 = reconstruct(imm);
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        decompose(x_rs1_u32 & imm_u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::{decompose, reconstruct, sext};
    #[test]
    pub fn apply() {
        let imm: u32 = sext(0xFFF, 11);
        let x_rs1: u32 = 0x0FFF_FFFF;
        let rd_w_decomp: [u8; 8] = Andi::apply(&decompose(imm), &decompose(x_rs1), &[0u8; 8]);
        assert_eq!(reconstruct(&rd_w_decomp), x_rs1 & imm)
    }
}
