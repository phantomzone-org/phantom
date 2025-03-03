//! sltiu: set less than immediate
//!
//! # Format
//!
//! sltiu rd,rs1,imm
//!
//! # Description
//!
//! Place the value 1 in register rd if register rs1 is
//! less than the immediate when both are treated as
//! unsigned numbers, else 0 is written to rd.
//!
//! # Implementation
//!
//! x[rd] = x[rs1] <u sext(immediate)

#[cfg(test)]
use crate::instructions::{test_i_type, OpID};
#[test]
fn instruction_parsing() {
    test_i_type(0b011, 0b0010011, OpID::SLTIU)
}
