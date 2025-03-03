#[cfg(test)]
use crate::instructions::{test_s_type, OpID};
#[test]
fn instruction_parsing() {
    test_s_type(0b010, 0b0100011, OpID::SW)
}
