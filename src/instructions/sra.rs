#[cfg(test)]
use crate::instructions::{test_r_type, OpID};
#[test]
fn instruction_parsing() {
    test_r_type(0b010000, 0b101, 0b0110011, OpID::SRA)
}
