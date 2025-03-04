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

use crate::instructions::{decompose, reconstruct, Arithmetic};

pub struct Sltiu();

impl Arithmetic for Sltiu {
    fn apply(imm: &[u8; 8], x_rs1: &[u8; 8], _x_rs2: &[u8; 8]) -> [u8; 8] {
        if reconstruct(x_rs1) < reconstruct(imm) {
            return decompose(1);
        }
        decompose(0)
    }
}
