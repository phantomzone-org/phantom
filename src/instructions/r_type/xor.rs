use crate::instructions::{decompose, reconstruct, Arithmetic};

pub struct Xor();

impl Arithmetic for Xor {
    fn apply(&self, _imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8] {
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        let x_rs2_u32: u32 = reconstruct(x_rs2);
        decompose(x_rs1_u32 ^ x_rs2_u32)
    }
}
