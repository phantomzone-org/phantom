//! ori
//! 
//! # Format
//! 
//! ori rd,rs1,imm
//! 
//! # Description
//! 
//! Performs bitwise OR on register rs1 and the 
//! sign-extended 12-bit immediate and place the result in rd
//! 
//! # Implementation
//! 
//! x[rd] = x[rs1] | sext(immediate)