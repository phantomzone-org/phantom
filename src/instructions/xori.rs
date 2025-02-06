//! xori
//! 
//! # Format
//! 
//! xori rd,rs1,imm
//! 
//! # Description
//! 
//! Performs bitwise XOR on register rs1 and the sign-extended 12-bit 
//! immediate and place the result in rd. 
//! Note, “XORI rd, rs1, -1” performs a bitwise logical inversion of 
//! register rs1(assembler pseudo-instruction NOT rd, rs)
//! 
//! # Implementation
//! 
//! x[rd] = x[rs1] ^ sext(immediate)