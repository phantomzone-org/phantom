use crate::instructions::{decompose, reconstruct};

pub struct Remu();

impl Remu {
    pub fn apply(_imm: &[u32; 8], x_rs1: &[u32; 8], x_rs2: &[u32; 8]) -> [u32; 8] {
        let x_rs1_u32: u32 = reconstruct(x_rs1) as u32;
        let x_rs2_u32: u32 = reconstruct(x_rs2) as u32;
        if x_rs2_u32 == 0 {
            return decompose(x_rs1_u32);
        }
        decompose((x_rs1_u32 % x_rs2_u32) as u32)
    }
}
