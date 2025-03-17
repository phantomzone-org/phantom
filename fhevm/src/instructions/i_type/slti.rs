use crate::instructions::{decompose, reconstruct};

pub struct Slti();

impl Slti {
    pub fn apply(imm: &[u8; 8], x_rs1: &[u8; 8], _x_rs2: &[u8; 8]) -> [u8; 8] {
        if (reconstruct(x_rs1) as i32) < (reconstruct(imm) as i32) {
            return decompose(1);
        }
        decompose(0)
    }
}
