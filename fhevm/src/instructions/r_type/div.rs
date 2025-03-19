use crate::instructions::{decompose, reconstruct};

pub struct Div();

impl Div {
    pub fn apply(_imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8] {
        let x_rs1_i32: i32 = reconstruct(x_rs1) as i32;
        let x_rs2_i32: i32 = reconstruct(x_rs2) as i32;
        if x_rs2_i32 == 0{
            return decompose(u32::MAX as u32)
        }else if x_rs1_i32 == i32::MIN && x_rs2_i32 == -1{
            return decompose(i32::MIN as u32)
        }
        decompose((x_rs1_i32/x_rs2_i32) as u32)
    }
}
