//! # Description
//! 
//! Adds the registers rs1 and rs2 and stores the result in rd.
//! Arithmetic overflow is ignored and the result is simply the low XLEN bits of the result.
//! 
//! # Implementation
//! |     4      |      4     |     4     |     4    |     4    |  5  |  5  |  5 |
//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] + x[rs2]

use super::Arithmetic;

pub struct Add();

impl Arithmetic for Add{
    fn apply(&self, imm_19: u32, imm_15: u32, imm_11: u32, imm_7: u32, imm_3: u32, x_rs1: u32, x_rs2: u32) -> (u32, u32){
        (rs1.wrapping_add(rs2), 0)
    }
}



