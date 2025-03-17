//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] & x[rs2]

use crate::instructions::{decompose, reconstruct};

pub struct And();

impl And {
    pub fn apply(_imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8] {
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        let x_rs2_u32: u32 = reconstruct(x_rs2);
        decompose(x_rs1_u32 & x_rs2_u32)
    }
}
