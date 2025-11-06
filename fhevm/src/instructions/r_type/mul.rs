use crate::instructions::{decompose, reconstruct};

pub struct Mul();

impl Mul {
    pub fn apply(_imm: &[u32; 8], x_rs1: &[u32; 8], x_rs2: &[u32; 8]) -> [u32; 8] {
        let x_rs1_i32: i32 = reconstruct(x_rs1) as i32;
        let x_rs2_i32: i32 = reconstruct(x_rs2) as i32;
        decompose(x_rs1_i32.wrapping_mul(x_rs2_i32) as u32)
    }
}
