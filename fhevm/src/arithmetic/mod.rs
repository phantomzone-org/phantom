use core::{
    backend::{Backend, Module, FFT64},
    glwe::ciphertext::{GLWECiphertextToMut, GLWECiphertextToRef},
    GGSWCiphertext, GLWECiphertext, GLWEOps, GLWEPlaintext, GetRow, Infos, Scratch, ScratchCore,
    SetMetaData,
};
use std::usize;

use itertools::{izip, Itertools};

use crate::{
    arithmetic::{add::airth_tmp_glwe_bounds, bitwise::bitwise_tmp_glwe_bounds},
    utils::split_by_bounds_mut,
};

pub mod add;
pub mod bitwise;
pub(crate) mod codegen;

static WORD_SIZE: usize = 32;

fn cmux<G1: GLWECiphertextToRef, G2: GLWECiphertextToRef>(
    t: &G1,
    f: &G2,
    s: &GGSWCiphertext<Vec<u8>, FFT64>,
    o: &mut GLWECiphertext<Vec<u8>>,
    module: &Module<FFT64>,
    scratch: &mut Scratch,
) {
    o.sub(module, t, f);
    o.external_product_inplace(module, s, scratch);
    o.add_inplace(module, f);
}

struct Node {
    input_index: usize,
    high_index: usize,
    low_index: usize,
}
impl Node {
    const fn new(input_index: usize, high_index: usize, low_index: usize) -> Self {
        Self {
            input_index,
            high_index,
            low_index,
        }
    }
}

pub(super) struct BitCircuit<const N: usize, const K: usize> {
    pub(crate) lvld_nodes: [Node; N],
    pub(crate) lvl_bounds: [usize; K],
}
impl<const N: usize, const K: usize> BitCircuit<N, K> {
    const fn new(lvld_nodes: [Node; N], lvl_bounds: [usize; K]) -> Self {
        Self {
            lvld_nodes,
            lvl_bounds,
        }
    }
}

trait BitCircuitInfo {
    fn info(&self) -> (&[Node], &[usize]);
}

enum OpType {
    Add,
    Sub,
    Slt,
    SltU,
    SLL,
    SRL,
    SRA,
    AND,
    OR,
    XOR,
}

struct Helper {
    tmp_glwes: Vec<GLWECiphertext<Vec<u8>>>,
    tmp_glwes_bounds: Vec<usize>,
    single_bit_output: bool,
}

impl Helper {
    fn new(op: OpType, module: &Module<FFT64>, basek: usize, k_glwe: usize, rank: usize) -> Self {
        let ((total_tmp_glwes, tmp_glwes_bounds), single_bit_output) = match op {
            OpType::Add | OpType::Sub => (airth_tmp_glwe_bounds(op), false),
            OpType::AND | OpType::OR | OpType::XOR => (bitwise_tmp_glwe_bounds(op), false),
            _ => (airth_tmp_glwe_bounds(op), false),
        };

        let tmp_glwes = (0..total_tmp_glwes)
            .map(|_| GLWECiphertext::alloc(module, basek, k_glwe, rank))
            .collect_vec();
        Self {
            tmp_glwes_bounds,
            tmp_glwes,
            single_bit_output,
        }
    }

    fn tmp_glwes_mut(&mut self) -> Vec<&mut [GLWECiphertext<Vec<u8>>]> {
        split_by_bounds_mut(self.tmp_glwes.as_mut(), &self.tmp_glwes_bounds)
    }
}

fn pointer_hurdle(
    node: &Node,
    output: &mut GLWECiphertext<Vec<u8>>,
    terminal_nodes: &[GLWEPlaintext<Vec<u8>>; 2],
    tmp_outs: &[GLWECiphertext<Vec<u8>>],
    inputs: &[&GGSWCiphertext<Vec<u8>, FFT64>],
    module: &Module<FFT64>,
    scratch: &mut Scratch,
) {
    let if_true = node.high_index;
    let if_false = node.low_index;
    if if_true < 2 && if_false < 2 {
        // both children nodes are terminal nodes
        cmux(
            &terminal_nodes[if_true],
            &terminal_nodes[if_false],
            &inputs[node.input_index],
            output,
            module,
            scratch,
        );
    } else if if_true < 2 {
        cmux(
            &terminal_nodes[if_true],
            &tmp_outs[if_false - 2],
            &inputs[node.input_index],
            output,
            module,
            scratch,
        );
    } else if if_false < 2 {
        cmux(
            &tmp_outs[if_true - 2],
            &terminal_nodes[if_false],
            inputs[node.input_index],
            output,
            module,
            scratch,
        );
    } else {
        cmux(
            &tmp_outs[if_true - 2],
            &tmp_outs[if_false - 2],
            inputs[node.input_index],
            output,
            module,
            scratch,
        );
    }
}

/// Circuit description is `nodes_lvld` and `lvl_bounds`
fn eval_circuit(
    nodes_lvld: &[Node],
    lvl_bounds: &[usize],
    module: &Module<FFT64>,
    inputs: &[&GGSWCiphertext<Vec<u8>, FFT64>],
    output: &mut GLWECiphertext<Vec<u8>>,
    terminal_nodes: &[GLWEPlaintext<Vec<u8>>; 2],
    tmp_outs: &mut [GLWECiphertext<Vec<u8>>],
    scratch: &mut Scratch,
) {
    // Note: tmp_outs does not store the expected first two terminal nodes. Thus, j^th node's tmp output
    // is stored at tmp_outs[j-2]

    // start at level 1
    for i in 0..lvl_bounds.len() - 2 {
        let start = lvl_bounds[i];
        let end = lvl_bounds[i + 1];
        let (tmpouts_ref, tmpouts_mut) = tmp_outs.split_at_mut(start - 2);
        for j in start..end {
            let node = &nodes_lvld[j];
            pointer_hurdle(
                node,
                &mut tmpouts_mut[j - start],
                terminal_nodes,
                &tmpouts_ref,
                inputs,
                module,
                scratch,
            );
        }
    }

    // handle last output
    // there's always only 1 node at last level
    let node = nodes_lvld.last().unwrap();
    pointer_hurdle(
        node,
        output,
        terminal_nodes,
        &tmp_outs,
        inputs,
        module,
        scratch,
    );
}

#[cfg(test)]
mod tests {
    use core::{
        backend::{
            Decoding, Encoding, Module, ScalarZnx, ScalarZnxAlloc, VecZnxBigOps, VecZnxDftAlloc,
            VecZnxDftOps, ZnxInfos, ZnxViewMut,
        },
        gglwe, FourierGLWECiphertext, FourierGLWESecret, GGSWCiphertext, GLWESecret, ScratchOwned,
    };
    use std::ops::{BitAnd, BitOr, BitXor};

    use super::*;
    use itertools::Itertools;
    use rand::{rng, RngCore, TryRngCore};
    use sampling::source::Source;

    // macro_rules! eval_test {
    //     ($op_type:expr, $fn_name:expr, $inputs:ident, $output_len:ident, $assert: expr) => {
    //         let mut helper = Helper::new($op_type, &module, basek, k_glwe, rank);
    //         let mut outputs = (0..$output_len)
    //             .map(|_| GLWECiphertext::alloc(&module, basek, k_glwe, rank))
    //             .collect_vec();
    //         $fn_name(
    //             &$inputs,
    //             &mut outputs,
    //             &mut helper.tmp_glwes_mut(),
    //             &terminal_nodes,
    //             scratch_owned.borrow(),
    //             &module,
    //         );
    //         let have = outputs
    //             .iter()
    //             .map(|b| decrypt_bit(&module, b, &sk_fourier, scratch_owned.borrow()))
    //             .collect_vec();
    //
    //         $assert;
    //     };
    // }

    use crate::utils::tests::{bits_to_u32, i64_to_u64, u32_to_bits};
    #[test]
    fn rr() {
        let logn = 4;
        let rank = 1;
        let basek = 15;
        let k_glwe = basek * 3;
        let k_ggsw = basek * 4;
        let digit_ggsw = 1;
        let k_pt = 1;
        let sigma = 3.2;
        let word_size = WORD_SIZE;

        let mut seed = [0; 32];
        rng().fill_bytes(&mut seed);
        let mut source = Source::new(seed);

        let module = Module::new(1 << logn);
        let mut sk = GLWESecret::alloc(&module, rank);
        sk.fill_binary_prob(0.5, &mut source);
        let sk_fourier = FourierGLWESecret::from(&module, &sk);

        let mut scratch_owned = ScratchOwned::new(
            GGSWCiphertext::encrypt_sk_scratch_space(&module, basek, k_ggsw, digit_ggsw)
                | module.bytes_of_scalar_znx(1)
                | add::arith_scratch_space(
                    &module, basek, k_glwe, k_glwe, k_ggsw, digit_ggsw, rank,
                ),
        );

        let a = rng().try_next_u32().unwrap();
        let b = rng().try_next_u32().unwrap();
        // let a = 3;
        // let b = 3;
        let inputs = u32_to_bits(a)
            .iter()
            .chain(u32_to_bits(b).iter())
            .map(|bit| {
                let mut ggsw = GGSWCiphertext::alloc(
                    &module,
                    basek,
                    k_ggsw,
                    k_glwe.div_ceil(basek),
                    digit_ggsw,
                    rank,
                );
                let scratch1 = scratch_owned.borrow();
                let (mut pt, scratch1) = scratch1.tmp_scalar_znx(&module, 1);
                pt.raw_mut()[0] = *bit as i64;
                ggsw.encrypt_sk(
                    &module,
                    &pt,
                    &sk_fourier,
                    &mut source.branch(),
                    &mut source.branch(),
                    sigma,
                    scratch1,
                );
                ggsw
            })
            .collect_vec();
        let inputs_ref = inputs.iter().map(|c| c).collect_vec();
        // shamt: a[0:4]
        let inputs_shift_ref = inputs
            .iter()
            .take(5)
            .chain(inputs.iter().skip(word_size))
            .map(|c| c)
            .collect_vec();

        let terminal_nodes = [0, 1].map(|b| {
            let mut pt = GLWEPlaintext::alloc(&module, basek, k_pt);
            pt.data.encode_coeff_i64(0, basek, k_pt, 0, b, k_pt);
            pt
        });

        macro_rules! eval_test {
            ($op_type:expr, $fn_name:expr, $inputs:ident, $output_len:ident, $want: expr) => {
                let mut helper = Helper::new($op_type, &module, basek, k_glwe, rank);
                let mut outputs = (0..$output_len)
                    .map(|_| GLWECiphertext::alloc(&module, basek, k_glwe, rank))
                    .collect_vec();
                $fn_name(
                    &$inputs,
                    &mut outputs,
                    &mut helper.tmp_glwes_mut(),
                    &terminal_nodes,
                    scratch_owned.borrow(),
                    &module,
                );
                let have = outputs
                    .iter()
                    .map(|b| decrypt_bit(&module, b, &sk_fourier, scratch_owned.borrow()))
                    .collect_vec();

                assert_eq!(have, $want);
            };
        }

        eval_test!(
            OpType::Add,
            add::add,
            inputs_ref,
            word_size,
            u32_to_bits(a.wrapping_add(b))
        );

        eval_test!(
            OpType::Sub,
            add::sub,
            inputs_ref,
            word_size,
            u32_to_bits(a.wrapping_sub(b))
        );

        let a_shamt = a & ((1 << 5) - 1);
        eval_test!(
            OpType::SLL,
            add::sll,
            inputs_shift_ref,
            word_size,
            u32_to_bits(b.wrapping_shl(a_shamt))
        );

        eval_test!(
            OpType::SRL,
            add::srl,
            inputs_shift_ref,
            word_size,
            u32_to_bits(b.wrapping_shr(a_shamt))
        );

        eval_test!(
            OpType::SRA,
            add::sra,
            inputs_shift_ref,
            word_size,
            u32_to_bits(((b as i32).wrapping_shr(a_shamt)) as u32)
        );

        eval_test!(
            OpType::AND,
            bitwise::and,
            inputs_ref,
            word_size,
            u32_to_bits(a.bitand(b))
        );

        eval_test!(
            OpType::OR,
            bitwise::or,
            inputs_ref,
            word_size,
            u32_to_bits(a.bitor(b))
        );

        eval_test!(
            OpType::XOR,
            bitwise::xor,
            inputs_ref,
            word_size,
            u32_to_bits(a.bitxor(b))
        );

        // perform the operations
        // check operations are valid
        // then homomorphic selection
        // then check selection is valid
    }
    // TODO (Jay): Measure noise
    fn decrypt_bit(
        module: &Module<FFT64>,
        bit_ct: &GLWECiphertext<Vec<u8>>,
        sk: &FourierGLWESecret<Vec<u8>, FFT64>,
        scratch: &mut Scratch,
    ) -> u64 {
        let mut pt = GLWEPlaintext::alloc(module, bit_ct.basek(), 1);
        bit_ct.decrypt(module, &mut pt, sk, scratch);
        // println!("Bit PT: {}", pt.data);
        // println!(
        //     "Decoded PT: {}",
        //     pt.data.decode_coeff_i64(0, bit_ct.basek(), 1, 0)
        // );
        i64_to_u64(pt.data.decode_coeff_i64(0, bit_ct.basek(), 1, 0), 1)
    }

    #[test]
    fn trial() {
        let basek = 15;
        let logn = 12;
        let k_glwe = basek * 2;
        let k_ggsw = basek * 4;
        let digit_ggsw = 1;
        let rank = 1;
        let sigma = 3.2;

        let module = Module::<FFT64>::new(1 << logn);
        let mut source = Source::new([0u8; 32]);

        let mut sk = GLWESecret::alloc(&module, rank);
        sk.fill_ternary_prob(0.5, &mut source);
        let fourier_sk = FourierGLWESecret::from(&module, &sk);

        let mut scratch_o = ScratchOwned::new(
            GGSWCiphertext::encrypt_sk_scratch_space(&module, basek, k_ggsw, rank)
                | module.bytes_of_vec_znx_dft(rank + 1, k_ggsw.div_ceil(basek))
                | module.vec_znx_idft_tmp_bytes(),
        );
        // Encrypt GGSW
        let mut ggsw = GGSWCiphertext::alloc(
            &module,
            basek,
            k_ggsw,
            k_glwe.div_ceil(basek),
            digit_ggsw,
            rank,
        );
        let mut pt = ScalarZnx::<Vec<u8>>::new(module.n(), 1);
        pt.raw_mut()[0] = 1;
        ggsw.encrypt_sk(
            &module,
            &pt,
            &fourier_sk,
            &mut source.branch(),
            &mut source.branch(),
            sigma,
            scratch_o.borrow(),
        );

        // Extract Fourier GLWE
        let (mut f_glwe, scratch) = scratch_o
            .borrow()
            .tmp_fourier_glwe_ct(&module, basek, k_ggsw, rank);
        ggsw.get_row(&module, 0, 0, &mut f_glwe);

        let (mut vec_znx_big, scratch) =
            scratch.tmp_vec_znx_big(&module, 2, k_ggsw.div_ceil(basek));
        for i in 0..vec_znx_big.cols() {
            module.vec_znx_idft(&mut vec_znx_big, i, &f_glwe.data, i, scratch);
        }

        let mut glwe = GLWECiphertext::alloc(&module, basek, k_glwe, rank);
        for i in 0..glwe.cols() {
            module.vec_znx_big_normalize(basek, &mut glwe.data, i, &vec_znx_big, i, scratch);
        }

        let mut pt = GLWEPlaintext::alloc(&module, basek, k_glwe);
        glwe.decrypt(&module, &mut pt, &fourier_sk, scratch_o.borrow());
        println!("PT={}", pt.data);
    }
}
