use crate::instructions::{decompose, reconstruct};

pub struct Sra();

impl Sra {
    pub fn apply(_imm: &[u32; 8], x_rs1: &[u32; 8], x_rs2: &[u32; 8]) -> [u32; 8] {
        let x_rs1_u32: i32 = reconstruct(x_rs1) as i32;
        let x_rs2_u32: i32 = reconstruct(x_rs2) as i32;
        decompose((x_rs1_u32 >> (x_rs2_u32 & 0x1F)) as u32)
    }
}
