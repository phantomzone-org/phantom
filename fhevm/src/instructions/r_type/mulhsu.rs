use crate::instructions::{decompose, reconstruct};

pub struct Mulhsu();

impl Mulhsu {
    pub fn apply(_imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8] {
        let x_rs1_i64: i64 = reconstruct(x_rs1) as i32 as i64;
        let x_rs2_i64: i64 = reconstruct(x_rs2) as u32 as i64;
        decompose(((x_rs1_i64*x_rs2_i64 )>>32) as u32)
    }
}
