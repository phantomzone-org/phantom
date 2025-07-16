use core::{
    backend::{Module, FFT64},
    GGSWCiphertext, GLWECiphertext, GLWEPlaintext, Scratch,
};

use itertools::{izip, Itertools};

use super::{codegen, eval_circuit, BitCircuitInfo, OpType, WORD_SIZE};

#[inline]
fn bitwise_eval(
    circuit: &impl BitCircuitInfo,
    inputs: &[&GGSWCiphertext<Vec<u8>, FFT64>],
    out: &mut [GLWECiphertext<Vec<u8>>],
    tmp_outs: &mut [&mut [GLWECiphertext<Vec<u8>>]],
    // structure: [GlwePt(0), GlwePt(1)]
    terminal_nodes: &[GLWEPlaintext<Vec<u8>>; 2],
    scratch: &mut Scratch,
    module: &Module<FFT64>,
) {
    let (input_a, input_b) = inputs.split_at(inputs.len() / 2);
    izip!(
        input_a.iter(),
        input_b.iter(),
        out.iter_mut(),
        tmp_outs.iter_mut()
    )
    // .take(2)
    .for_each(|(ct_a, ct_b, output, tmpouts)| {
        let (nodes, lvl_bounds) = circuit.info();
        let inputs = [*ct_a, *ct_b];
        eval_circuit(
            nodes,
            lvl_bounds,
            module,
            &inputs,
            output,
            terminal_nodes,
            tmpouts,
            scratch,
        );
    });
}

pub(crate) fn and(
    inputs: &[&GGSWCiphertext<Vec<u8>, FFT64>],
    out: &mut [GLWECiphertext<Vec<u8>>],
    tmp_outs: &mut [&mut [GLWECiphertext<Vec<u8>>]],
    // structure: [GlwePt(0), GlwePt(1)]
    terminal_nodes: &[GLWEPlaintext<Vec<u8>>; 2],
    scratch: &mut Scratch,
    module: &Module<FFT64>,
) {
    bitwise_eval(
        &codegen::and_codegen::OUTPUT_CIRCUIT,
        inputs,
        out,
        tmp_outs,
        terminal_nodes,
        scratch,
        module,
    );
}

pub(crate) fn or(
    inputs: &[&GGSWCiphertext<Vec<u8>, FFT64>],
    out: &mut [GLWECiphertext<Vec<u8>>],
    tmp_outs: &mut [&mut [GLWECiphertext<Vec<u8>>]],
    // structure: [GlwePt(0), GlwePt(1)]
    terminal_nodes: &[GLWEPlaintext<Vec<u8>>; 2],
    scratch: &mut Scratch,
    module: &Module<FFT64>,
) {
    bitwise_eval(
        &codegen::or_codegen::OUTPUT_CIRCUIT,
        inputs,
        out,
        tmp_outs,
        terminal_nodes,
        scratch,
        module,
    );
}

pub(crate) fn xor(
    inputs: &[&GGSWCiphertext<Vec<u8>, FFT64>],
    out: &mut [GLWECiphertext<Vec<u8>>],
    tmp_outs: &mut [&mut [GLWECiphertext<Vec<u8>>]],
    // structure: [GlwePt(0), GlwePt(1)]
    terminal_nodes: &[GLWEPlaintext<Vec<u8>>; 2],
    scratch: &mut Scratch,
    module: &Module<FFT64>,
) {
    bitwise_eval(
        &codegen::xor_codegen::OUTPUT_CIRCUIT,
        inputs,
        out,
        tmp_outs,
        terminal_nodes,
        scratch,
        module,
    );
}

pub(crate) fn bitwise_tmp_glwe_bounds(op_type: OpType) -> (usize, Vec<usize>) {
    match op_type {
        OpType::OR | OpType::AND => {
            // AND and OR circuit have same structure
            let glwe_bounds = (0..WORD_SIZE).map(|i| i + 1).collect_vec();
            (WORD_SIZE, glwe_bounds)
        }
        OpType::XOR => {
            let glwe_bounds = (0..WORD_SIZE).map(|i| (i + 1) * 2).collect_vec();
            (WORD_SIZE * 2, glwe_bounds)
        }
        _ => {
            panic!("op_type not a bitwise operation");
        }
    }
}
