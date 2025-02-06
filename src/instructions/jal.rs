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
//! x[rd] = pc+4; pc += sext(offset)