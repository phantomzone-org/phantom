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

use crate::instructions::{decompose, reconstruct};

pub struct Jalr();

impl Jalr {
    pub fn apply_pc(
        imm: &[u32; 8],
        x_rs1: &[u32; 8],
        _x_rs2: &[u32; 8],
        _pc: &[u32; 8],
    ) -> [u32; 8] {
        decompose(reconstruct(x_rs1).wrapping_add(reconstruct(imm)) & 0xFFFF_FFFE)
    }

    pub fn apply_rd(
        _imm: &[u32; 8],
        _x_rs1: &[u32; 8],
        _x_rs2: &[u32; 8],
        pc: &[u32; 8],
    ) -> [u32; 8] {
        decompose(reconstruct(pc).wrapping_add(4))
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
        let pc: u32 = 4;
        let rd_w_decomp: [u32; 8] = Jalr::apply_rd(
            &decompose(imm),
            &decompose(x_rs1),
            &[0u32; 8],
            &decompose(pc),
        );
        let pc_w_decomp: [u32; 8] = Jalr::apply_pc(
            &decompose(imm),
            &decompose(x_rs1),
            &[0u32; 8],
            &decompose(pc),
        );
        assert_eq!(reconstruct(&rd_w_decomp), pc.wrapping_add(4));
        assert_eq!(
            reconstruct(&pc_w_decomp),
            x_rs1.wrapping_add(imm) & 0xFFFF_FFFE
        );
    }
}
