use crate::instructions::{decompose, reconstruct};

pub struct Slti();

impl Slti {
    pub fn apply(imm: &[u32; 8], x_rs1: &[u32; 8], _x_rs2: &[u32; 8]) -> [u32; 8] {
        if (reconstruct(x_rs1) as i32) < (reconstruct(imm) as i32) {
            return decompose(1);
        }
        decompose(0)
    }
}
