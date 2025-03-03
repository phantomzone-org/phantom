//! slli
//!
//! # Format
//!
//! slli rd,rs1,shamt
//!
//! # Description
//!
//! Performs logical left shift on the value in register
//! rs1 by the shift amount held in the lower 5 bits of
//! the immediate In RV64, bit-25 is used to shamt[5].
//!
//! # Implementation
//!
//! x[rd] = x[rs1] << shamt

#[cfg(test)]
use crate::instructions::{test_i_shamt_type, OpID};
#[test]
fn instruction_parsing() {
    test_i_shamt_type(0b000000011111, 0b001, OpID::SLLI)
}
