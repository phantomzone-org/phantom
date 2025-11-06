use crate::instructions::{decompose, reconstruct};

pub struct Mulh();

impl Mulh {
    pub fn apply(_imm: &[u32; 8], x_rs1: &[u32; 8], x_rs2: &[u32; 8]) -> [u32; 8] {
        let x_rs1_i64: i64 = reconstruct(x_rs1) as i32 as i64;
        let x_rs2_i64: i64 = reconstruct(x_rs2) as i32 as i64;
        decompose(((x_rs1_i64 * x_rs2_i64) >> 32) as u32)
    }
}
