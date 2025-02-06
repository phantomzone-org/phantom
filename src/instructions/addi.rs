//! addi: add immediate
//! 
//! # Format
//! 
//! addi rd,rs1,imm
//! 
//! 
//! # Description
//! 
//! Adds the sign-extended 12-bit immediate to register rs1. 
//! Arithmetic overflow is ignored and the result is simply the 
//! low XLEN bits of the result. ADDI rd, rs1, 0 is used to 
//! implement the MV rd, rs1 assembler pseudo-instruction.
//! 
//! # Implementation
//! 
//! x[rd] = x[rs1] + sext(immediate)