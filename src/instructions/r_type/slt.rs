use crate::instructions::{decompose, reconstruct, Arithmetic};
pub struct Slt();

impl Arithmetic for Slt {
    fn apply(_imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8] {
        if (reconstruct(x_rs1) as i32) < (reconstruct(x_rs2) as i32) {
            return decompose(1);
        }
        decompose(0)
    }
}
