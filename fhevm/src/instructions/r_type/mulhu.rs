use crate::instructions::{decompose, reconstruct};

pub struct Mulhu();

impl Mulhu {
    pub fn apply(_imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8] {
        let x_rs1_u64: u64 = reconstruct(x_rs1) as u64;
        let x_rs2_u64: u64 = reconstruct(x_rs2) as u64;
        decompose(((x_rs1_u64*x_rs2_u64)>>32) as u32)
    }
}
