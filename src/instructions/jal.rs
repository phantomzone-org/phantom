//! JAL
//! 
//! # Format
//! 
//! jal rd,offset
//! 
//! # Description
//! 
//! Jump to address and place return address in rd.
//! 
//! Implementation
//! 
//! jal    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + 4; pc += sext(imm[19:0])

use super::{Arithmetic, sext};

pub struct Jal();

impl Arithmetic for Jal{
    fn apply(&self, imm_19: u32, imm_15: u32, imm_11: u32, imm_7: u32, imm_3: u32, x_rs1: u32, x_rs2: u32, pc: u32) -> (u32, u32){
        (pc + 4, sext(imm_19<<16 | imm_11<<12 | imm_11<<8 | imm_7<<4 | imm_3, 12))
    }
}