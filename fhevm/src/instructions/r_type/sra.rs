use crate::instructions::{decompose, reconstruct};

pub struct Sra();

impl Sra {
    pub fn apply(_imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8] {
        let x_rs1_u32: i32 = reconstruct(x_rs1) as i32;
        let x_rs2_u32: i32 = reconstruct(x_rs2) as i32;
        decompose((x_rs1_u32 >> (x_rs2_u32 & 0x1F)) as u32)
    }
}
