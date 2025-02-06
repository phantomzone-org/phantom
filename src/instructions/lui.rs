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
//! x[rd] = sext(immediate[31:12] << 12)