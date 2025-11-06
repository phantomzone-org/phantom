use crate::instructions::{decompose, reconstruct};
pub struct Slt();

impl Slt {
    pub fn apply(_imm: &[u32; 8], x_rs1: &[u32; 8], x_rs2: &[u32; 8]) -> [u32; 8] {
        if (reconstruct(x_rs1) as i32) < (reconstruct(x_rs2) as i32) {
            return decompose(1);
        }
        decompose(0)
    }
}
