//! slli
//! 
//! # Format
//! 
//! srai rd,rs1,shamt
//! 
//! # Description
//! 
//! Performs arithmetic right shift on the value in register 
//! rs1 by the shift amount held in the lower 5 bits of the 
//! immediate In RV64, bit-25 is used to shamt[5].
//! 
//! # Implementation
//! 
//! x[rd] = x[rs1] >>s shamt