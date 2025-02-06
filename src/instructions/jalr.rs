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
//! t =pc+4; pc=(x[rs1]+sext(offset))&∼1; x[rd]=t