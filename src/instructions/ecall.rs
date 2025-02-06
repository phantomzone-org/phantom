//! slti: set less than immediate
//! 
//! # Format
//! 
//! slti rd,rs1,imm
//! 
//! # Description
//! 
//! Place the value 1 in register rd if register rs1 is 
//! less than the signextended immediate when both are 
//! treated as signed numbers, else 0 is written to rd.
//! 
//! # Implementation
//! 
//! x[rd] = x[rs1] <s sext(immediate)