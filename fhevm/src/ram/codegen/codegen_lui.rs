use poulpy_schemes::tfhe::bdd_arithmetic::{
    BitCircuit, BitCircuitFamily, BitCircuitInfo, Circuit, Node,
};
pub(crate) enum AnyBitCircuit {
    B0(BitCircuit<0>),
    B1(BitCircuit<0>),
    B2(BitCircuit<0>),
    B3(BitCircuit<0>),
    B4(BitCircuit<0>),
    B5(BitCircuit<0>),
    B6(BitCircuit<0>),
    B7(BitCircuit<0>),
    B8(BitCircuit<0>),
    B9(BitCircuit<0>),
    B10(BitCircuit<0>),
    B11(BitCircuit<0>),
    B12(BitCircuit<2>),
    B13(BitCircuit<2>),
    B14(BitCircuit<2>),
    B15(BitCircuit<2>),
    B16(BitCircuit<2>),
    B17(BitCircuit<2>),
    B18(BitCircuit<2>),
    B19(BitCircuit<2>),
    B20(BitCircuit<2>),
    B21(BitCircuit<2>),
    B22(BitCircuit<2>),
    B23(BitCircuit<2>),
    B24(BitCircuit<2>),
    B25(BitCircuit<2>),
    B26(BitCircuit<2>),
    B27(BitCircuit<2>),
    B28(BitCircuit<2>),
    B29(BitCircuit<2>),
    B30(BitCircuit<2>),
    B31(BitCircuit<2>),
}
impl BitCircuitInfo for AnyBitCircuit {
    fn info(&self) -> (&[Node], usize) {
        match self {
            AnyBitCircuit::B0(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B1(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B2(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B3(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B4(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B5(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B6(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B7(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B8(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B9(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B10(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B11(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B12(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B13(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B14(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B15(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B16(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B17(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B18(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B19(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B20(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B21(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B22(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B23(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B24(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B25(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B26(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B27(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B28(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B29(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B30(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
            AnyBitCircuit::B31(bit_circuit) => {
                (bit_circuit.nodes.as_ref(), bit_circuit.max_inter_state)
            }
        }
    }
}

impl BitCircuitFamily for AnyBitCircuit {
    const INPUT_BITS: usize = 20;
    const OUTPUT_BITS: usize = 32;
}

pub(crate) static OUTPUT_CIRCUITS: Circuit<AnyBitCircuit, 32usize> = Circuit([
    AnyBitCircuit::B0(BitCircuit::new([], 0)),
    AnyBitCircuit::B1(BitCircuit::new([], 0)),
    AnyBitCircuit::B2(BitCircuit::new([], 0)),
    AnyBitCircuit::B3(BitCircuit::new([], 0)),
    AnyBitCircuit::B4(BitCircuit::new([], 0)),
    AnyBitCircuit::B5(BitCircuit::new([], 0)),
    AnyBitCircuit::B6(BitCircuit::new([], 0)),
    AnyBitCircuit::B7(BitCircuit::new([], 0)),
    AnyBitCircuit::B8(BitCircuit::new([], 0)),
    AnyBitCircuit::B9(BitCircuit::new([], 0)),
    AnyBitCircuit::B10(BitCircuit::new([], 0)),
    AnyBitCircuit::B11(BitCircuit::new([], 0)),
    AnyBitCircuit::B12(BitCircuit::new([Node::Cmux(0, 1, 0), Node::None], 2)),
    AnyBitCircuit::B13(BitCircuit::new([Node::Cmux(1, 1, 0), Node::None], 2)),
    AnyBitCircuit::B14(BitCircuit::new([Node::Cmux(2, 1, 0), Node::None], 2)),
    AnyBitCircuit::B15(BitCircuit::new([Node::Cmux(3, 1, 0), Node::None], 2)),
    AnyBitCircuit::B16(BitCircuit::new([Node::Cmux(4, 1, 0), Node::None], 2)),
    AnyBitCircuit::B17(BitCircuit::new([Node::Cmux(5, 1, 0), Node::None], 2)),
    AnyBitCircuit::B18(BitCircuit::new([Node::Cmux(6, 1, 0), Node::None], 2)),
    AnyBitCircuit::B19(BitCircuit::new([Node::Cmux(7, 1, 0), Node::None], 2)),
    AnyBitCircuit::B20(BitCircuit::new([Node::Cmux(8, 1, 0), Node::None], 2)),
    AnyBitCircuit::B21(BitCircuit::new([Node::Cmux(9, 1, 0), Node::None], 2)),
    AnyBitCircuit::B22(BitCircuit::new([Node::Cmux(10, 1, 0), Node::None], 2)),
    AnyBitCircuit::B23(BitCircuit::new([Node::Cmux(11, 1, 0), Node::None], 2)),
    AnyBitCircuit::B24(BitCircuit::new([Node::Cmux(12, 1, 0), Node::None], 2)),
    AnyBitCircuit::B25(BitCircuit::new([Node::Cmux(13, 1, 0), Node::None], 2)),
    AnyBitCircuit::B26(BitCircuit::new([Node::Cmux(14, 1, 0), Node::None], 2)),
    AnyBitCircuit::B27(BitCircuit::new([Node::Cmux(15, 1, 0), Node::None], 2)),
    AnyBitCircuit::B28(BitCircuit::new([Node::Cmux(16, 1, 0), Node::None], 2)),
    AnyBitCircuit::B29(BitCircuit::new([Node::Cmux(17, 1, 0), Node::None], 2)),
    AnyBitCircuit::B30(BitCircuit::new([Node::Cmux(18, 1, 0), Node::None], 2)),
    AnyBitCircuit::B31(BitCircuit::new([Node::Cmux(19, 1, 0), Node::None], 2)),
]);
