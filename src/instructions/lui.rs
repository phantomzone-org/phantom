//! LUI: load upper immediate.
//! 
//! # Format 
//! 
//! lui rd,imm
//! 
//! # Description
//! 
//! Build 32-bit constants and uses the U-type format. 
//! LUI places the U-immediate value in the top 20 bits 
//! of the destination register rd, filling in the lowest 
//! 12 bits with zeros.
//! 
//! # Implementation
//! 
//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(imm[19:0] << 12)
//! 
//! use super::{Counter, sext};

pub struct Bgeu();

impl Counter for Bgeu{
    fn apply(&self, imm_19: u32, imm_15: u32, imm_11: u32, imm_7: u32, imm_3: u32, x_rs1: u32, x_rs2: u32, pc: u32) -> (u32, u32){
        (sext(imm_19<<16 | imm_11<<12 | imm_11<<8 | imm_7<<4 | imm_3, 12)<<12, 0)
    }
}