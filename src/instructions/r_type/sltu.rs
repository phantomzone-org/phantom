use crate::instructions::{decompose, reconstruct, Arithmetic};
pub struct Sltu();

impl Arithmetic for Sltu {
    fn apply(_imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8] {
        if reconstruct(x_rs1) < reconstruct(x_rs2) {
            return decompose(1);
        }
        decompose(0)
    }
}
