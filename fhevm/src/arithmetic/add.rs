use super::{codegen, eval_circuit, BitCircuitInfo, Helper, OpType};
use core::{
    backend::{Module, FFT64},
    GGLWECiphertext, GGSWCiphertext, GLWECiphertext, GLWEOps, GLWEPlaintext, Infos, Scratch,
    SetMetaData,
};
use itertools::{izip, Itertools};

pub(crate) fn arith_scratch_space(
    module: &Module<FFT64>,
    basek: usize,
    k_out: usize,
    k_in: usize,
    k_ggsw: usize,
    digits_ggsw: usize,
    rank: usize,
) -> usize {
    GLWECiphertext::external_product_scratch_space(
        module,
        basek,
        k_out,
        k_in,
        k_ggsw,
        digits_ggsw,
        rank,
    )
}

fn calc_tmp_glwe_bounds<C: BitCircuitInfo>(circuit: &[C]) -> (usize, Vec<usize>) {
    let mut bounds = vec![];
    let mut curr_idx = 0;
    circuit.iter().for_each(|c| {
        let (nodes, _) = c.info();
        // skip 0,1 terminal nodes since they are fixed to plaintext constants 0,1
        // skip last node since its output is written to the actual output bit
        nodes
            .iter()
            .take(nodes.len() - 1)
            .skip(2)
            .for_each(|_| curr_idx += 1);
        bounds.push(curr_idx);
    });
    (curr_idx, bounds)
}

pub(crate) fn airth_tmp_glwe_bounds(op_type: OpType) -> (usize, Vec<usize>) {
    // only used for circuit direction
    match op_type {
        OpType::Add => calc_tmp_glwe_bounds(&codegen::add_codegen::OUTPUT_CIRCUITS),
        OpType::Sub => calc_tmp_glwe_bounds(&codegen::sub_codegen::OUTPUT_CIRCUITS),
        OpType::SLL => calc_tmp_glwe_bounds(&codegen::sll_codegen::OUTPUT_CIRCUITS),
        OpType::SRL => calc_tmp_glwe_bounds(&codegen::srl_codegen::OUTPUT_CIRCUITS),
        OpType::SRA => calc_tmp_glwe_bounds(&codegen::sra_codegen::OUTPUT_CIRCUITS),
        _ => calc_tmp_glwe_bounds(&codegen::sub_codegen::OUTPUT_CIRCUITS),
    }
}

macro_rules! define_inner_eval_fn {
    ($fn_name:ident, $codegen:path) => {
        pub(crate) fn $fn_name(
            inputs: &[&GGSWCiphertext<Vec<u8>, FFT64>],
            out: &mut [GLWECiphertext<Vec<u8>>],
            tmp_outs: &mut [&mut [GLWECiphertext<Vec<u8>>]],
            terminal_nodes: &[GLWEPlaintext<Vec<u8>>; 2],
            scratch: &mut Scratch,
            module: &Module<FFT64>,
        ) {
            izip!($codegen.iter(), out.iter_mut(), tmp_outs)
                .enumerate()
                .for_each(|(index, (output_bit_circuit, out_ct, tmp_outs))| {
                    let (nodes_lvld, lvl_bounds) = output_bit_circuit.info();
                    eval_circuit(
                        nodes_lvld,
                        lvl_bounds,
                        module,
                        inputs,
                        out_ct,
                        terminal_nodes,
                        tmp_outs,
                        scratch,
                    );
                });
        }
    };
}

define_inner_eval_fn!(add, codegen::add_codegen::OUTPUT_CIRCUITS);
define_inner_eval_fn!(sub, codegen::sub_codegen::OUTPUT_CIRCUITS);
define_inner_eval_fn!(sll, codegen::sll_codegen::OUTPUT_CIRCUITS);
define_inner_eval_fn!(srl, codegen::srl_codegen::OUTPUT_CIRCUITS);
define_inner_eval_fn!(sra, codegen::sra_codegen::OUTPUT_CIRCUITS);

#[cfg(test)]
pub(crate) mod tests {
    use core::{
        backend::{Decoding, Encoding, Module, ScalarZnx, ZnxViewMut, FFT64},
        FourierGLWESecret, GGSWCiphertext, GLWECiphertext, GLWESecret, ScratchOwned,
    };

    use itertools::Itertools;
    use num_bigfloat::BigFloat;
    use rand::{rng, RngCore};
    use sampling::source::Source;

    use crate::utils::tests::*;

    use super::*;

    const SIGMA: f64 = 3.2;

    fn ggsw_encrypt(
        module: &Module<FFT64>,
        sk: &FourierGLWESecret<Vec<u8>, FFT64>,
        data: u64,
        basek: usize,
        k_ggsw: usize,
        rows: usize,
        digits_ggsw: usize,
        rank: usize,
        source: &mut Source,
    ) -> GGSWCiphertext<Vec<u8>, FFT64> {
        let mut scratch_owned = ScratchOwned::new(GGSWCiphertext::encrypt_sk_scratch_space(
            module, basek, k_ggsw, rank,
        ));

        let mut pt = ScalarZnx::<Vec<u8>>::new(sk.n(), 1);
        pt.raw_mut()[0] = data as i64;

        let mut ggsw = GGSWCiphertext::alloc(module, basek, k_ggsw, rows, digits_ggsw, rank);
        ggsw.encrypt_sk(
            module,
            &pt,
            sk,
            &mut source.branch(),
            &mut source.branch(),
            SIGMA,
            scratch_owned.borrow(),
        );

        ggsw
    }

    fn glwe_decrypt(
        module: &Module<FFT64>,
        glwe_ct: &GLWECiphertext<Vec<u8>>,
        sk: &FourierGLWESecret<Vec<u8>, FFT64>,
        k_pt: usize,
    ) -> u64 {
        let mut pt = GLWEPlaintext::alloc(module, glwe_ct.basek(), k_pt);
        let mut scratch_owned = ScratchOwned::new(GLWECiphertext::decrypt_scratch_space(
            module,
            glwe_ct.basek(),
            glwe_ct.k(),
        ));
        glwe_ct.decrypt(module, &mut pt, sk, scratch_owned.borrow());

        (pt.data.decode_coeff_i64(0, pt.basek(), pt.k(), 0) as u64) % (1 << k_pt)
    }

    fn glwe_noise(
        module: &Module<FFT64>,
        glwe_ct: &GLWECiphertext<Vec<u8>>,
        sk: &FourierGLWESecret<Vec<u8>, FFT64>,
        want: &[i64],
        k_pt: usize,
    ) -> BigFloat {
        let mut pt = GLWEPlaintext::alloc(module, glwe_ct.basek(), glwe_ct.k());
        let mut scratch_owned = ScratchOwned::new(GLWECiphertext::decrypt_scratch_space(
            module,
            glwe_ct.basek(),
            glwe_ct.k(),
        ));
        glwe_ct.decrypt(module, &mut pt, sk, scratch_owned.borrow());

        let mut datai64 = vec![0; module.n()];
        pt.data.decode_vec_i64(0, pt.basek(), pt.k(), &mut datai64);

        let scale = BigFloat::from(2).pow(&BigFloat::from((pt.k() - k_pt) as u64));
        let mut diff_sum = BigFloat::from(0);
        izip!(want.iter(), datai64.iter()).for_each(|(w, h)| {
            let h_scaled = BigFloat::from(*h) / &scale;
            diff_sum += BigFloat::from(*w) - h_scaled;
        });
        let avg = diff_sum / &BigFloat::from(module.n() as u64);
        avg.abs().log2()
    }
}
