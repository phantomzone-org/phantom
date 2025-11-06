use crate::instructions::{decompose, reconstruct};

pub struct Rem();

impl Rem {
    pub fn apply(_imm: &[u32; 8], x_rs1: &[u32; 8], x_rs2: &[u32; 8]) -> [u32; 8] {
        let x_rs1_i32: i32 = reconstruct(x_rs1) as i32;
        let x_rs2_i32: i32 = reconstruct(x_rs2) as i32;
        if x_rs2_i32 == 0 {
            decompose(x_rs1_i32 as u32)
        } else if x_rs1_i32 == i32::MIN && x_rs2_i32 == -1 {
            decompose(0)
        } else {
            decompose((x_rs1_i32 % x_rs2_i32) as u32)
        }
    }
}
