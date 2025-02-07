//! JALR
//! 
//! # Format
//! 
//! jalr rd,rs1,offset
//! 
//! # Description
//! 
//! Jump to address and place return address in rd.
//! 
//! Implementation
//! 
//! jalr   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | t = pc + 4; pc = (x[rs1] + sext(imm[11:0])) & ~1; x[rd] = t

use super::{Arithmetic, sext};

pub struct Jalr();

impl Arithmetic for Jalr{
    fn apply(&self, imm_19: u32, imm_15: u32, imm_11: u32, imm_7: u32, imm_3: u32, x_rs1: u32, x_rs2: u32, pc: u32) -> (u32, u32){
        (pc + 4, x_rs1.wrapping_add(sext(imm_19<<16 | imm_11<<12 | imm_11<<8 | imm_7<<4 | imm_3, 12)) & !1)
    }
}