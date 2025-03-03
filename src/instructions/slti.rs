#[cfg(test)]
use crate::instructions::{test_i_type, OpID};
#[test]
fn instruction_parsing() {
    test_i_type(0b010, 0b0010011, OpID::SLTI)
}
