use crate::instructions::{decompose, reconstruct};
pub struct Slt();

impl Slt {
    pub fn apply(_imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8] {
        if (reconstruct(x_rs1) as i32) < (reconstruct(x_rs2) as i32) {
            return decompose(1);
        }
        decompose(0)
    }
}
