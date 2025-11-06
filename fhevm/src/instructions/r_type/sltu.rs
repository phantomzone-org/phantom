use crate::instructions::{decompose, reconstruct};
pub struct Sltu();

impl Sltu {
    pub fn apply(_imm: &[u32; 8], x_rs1: &[u32; 8], x_rs2: &[u32; 8]) -> [u32; 8] {
        if reconstruct(x_rs1) < reconstruct(x_rs2) {
            return decompose(1);
        }
        decompose(0)
    }
}
