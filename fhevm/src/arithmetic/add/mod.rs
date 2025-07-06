use super::cmux;
use core::{
    backend::{Backend, Module, FFT64},
    glwe::ciphertext::{GLWECiphertextToMut, GLWECiphertextToRef},
    GGSWCiphertext, GLWECiphertext, GLWEOps, GLWEPlaintext, Infos, Scratch, SetMetaData,
};
use itertools::izip;

mod codegen;

pub(crate) fn add_scratch_space(
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

pub(crate) fn tmp_glwe_bounds() -> (usize, Vec<usize>) {
    let total: usize = codegen::OUTPUT_CIRCUITS
        .iter()
        .map(|circuit| circuit.inner().0.len() - 3)
        .sum();
    let mut bounds = vec![];
    let mut curr_idx = 0;
    codegen::OUTPUT_CIRCUITS.iter().for_each(|circ| {
        let nodes = circ.inner().0;
        // skip 0,1 terminal nodes since they are fixed to plaintext constants 0,1
        // skip last node since its output is written to the actual output bit
        nodes
            .iter()
            .take(nodes.len() - 1)
            .skip(2)
            .for_each(|_| curr_idx += 1);
        bounds.push(curr_idx);
    });
    (total, bounds)
}

pub(crate) fn add(
    inputs: &[&GGSWCiphertext<Vec<u8>, FFT64>],
    out: &mut [GLWECiphertext<Vec<u8>>],
    tmp_outs: &mut [&mut [GLWECiphertext<Vec<u8>>]],
    // structure: [GlwePt(0), GlwePt(1)]
    terminal_nodes_pt: &[GLWEPlaintext<Vec<u8>>; 2],
    scratch: &mut Scratch,
    module: &Module<FFT64>,
) {
    izip!(codegen::OUTPUT_CIRCUITS.iter(), out.iter_mut(), tmp_outs)
        .enumerate()
        .for_each(|(index, (output_bit_circuit, out_ct, tmp_outs))| {
            let (nodes_lvld, lvl_bounds) = output_bit_circuit.inner();

            // Note: tmp_outs does not store the expected first two terminal nodes. Thus, j^th node's tmp output
            // is stored at tmp_outs[j-2]

            // Note: We assume that
            // - only nodes at level 1 have terminal nodes as children.
            // - there are at-least 2 levels

            // Process level 1 separately because nodes at level 1 only have terminal nodes as their children
            {
                let start = lvl_bounds[0];
                let end = lvl_bounds[1];
                for j in start..end {
                    let node = &nodes_lvld[j];
                    cmux(
                        &terminal_nodes_pt[node.high_index],
                        &terminal_nodes_pt[node.low_index],
                        &inputs[node.input_index],
                        &mut tmp_outs[j - 2],
                        module,
                        scratch,
                    );
                }
            }

            // start at level 2
            for i in 1..lvl_bounds.len() - 2 {
                let start = lvl_bounds[i];
                let end = lvl_bounds[i + 1];
                let (tmpouts_ref, tmpouts_mut) = tmp_outs.split_at_mut(start - 2);
                for j in start..end {
                    let node = &nodes_lvld[j];
                    cmux(
                        &tmpouts_ref[node.high_index - 2],
                        &tmpouts_ref[node.low_index - 2],
                        &inputs[node.input_index],
                        &mut tmpouts_mut[j - start],
                        module,
                        scratch,
                    );
                }
            }

            // handle last output
            // there's always only 1 node at last level
            let node = nodes_lvld.last().unwrap();
            cmux(
                &tmp_outs[node.high_index - 2],
                &tmp_outs[node.low_index - 2],
                &inputs[node.input_index],
                out_ct,
                module,
                scratch,
            );
        });
}

#[cfg(test)]
mod tests {
    use core::{
        backend::{Decoding, Encoding, Module, ScalarZnx, ZnxViewMut, FFT64},
        FourierGLWESecret, GGSWCiphertext, GLWECiphertext, GLWESecret, ScratchOwned,
    };

    use itertools::Itertools;
    use sampling::source::Source;

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

    fn u32_to_bits(v: u32) -> Vec<u64> {
        (0..u32::BITS)
            .into_iter()
            .map(|i| ((v >> i) & 1) as u64)
            .collect()
    }

    fn bits_to_u32(bits: &[u64]) -> u32 {
        bits.iter().enumerate().fold(0u32, |acc, (i, b)| {
            assert!(*b == 0 || *b == 1);
            acc + (*b << i) as u32
        })
    }

    fn split_by_bounds_mut<'a, T>(mut slice: &'a mut [T], bounds: &[usize]) -> Vec<&'a mut [T]> {
        let mut chunks = Vec::with_capacity(bounds.len() - 1);
        let mut last_end = 0;
        for end in bounds.iter() {
            let (left, right) = slice.split_at_mut(*end - last_end);
            chunks.push(left);
            slice = right;
            last_end = *end;
        }

        chunks
    }

    #[test]
    fn fff() {
        let logn = 12;
        let rank = 1;
        let basek = 20;
        let k_glwe = basek * 2;
        let k_ggsw = basek * 4;
        let rank_ggsw = rank;
        let digit_ggsw = 1;
        let k_pt = 1;

        let mut source = Source::new([112; 32]);

        let module = Module::new(1 << logn);
        let mut sk = GLWESecret::alloc(&module, rank);
        sk.fill_binary_prob(0.5, &mut source);
        let sk_fourier = FourierGLWESecret::from(&module, &sk);

        let a = 23121u32;
        let b = 213412u32;

        // assert!(bits_to_u32(&u32_to_bits(a)) == a);

        let ggsw_a = u32_to_bits(a)
            .iter()
            .map(|b| {
                ggsw_encrypt(
                    &module,
                    &sk_fourier,
                    *b,
                    basek,
                    k_ggsw,
                    k_glwe.div_ceil(basek),
                    digit_ggsw,
                    rank,
                    &mut source,
                )
            })
            .collect_vec();
        let ggsw_b = u32_to_bits(b)
            .iter()
            .map(|b| {
                ggsw_encrypt(
                    &module,
                    &sk_fourier,
                    *b,
                    basek,
                    k_ggsw,
                    k_glwe.div_ceil(basek),
                    digit_ggsw,
                    rank,
                    &mut source,
                )
            })
            .collect_vec();

        let inputs = ggsw_a
            .iter()
            .chain(ggsw_b.iter())
            .map(|ct| ct)
            .collect_vec();
        let mut outputs = (0..32)
            .map(|_| GLWECiphertext::alloc(&module, basek, k_glwe, rank))
            .collect_vec();

        // Prepare scratch space
        let (tmp_glwes_count, tmp_glwes_bounds) = tmp_glwe_bounds();
        let mut tmp_glwes = (0..tmp_glwes_count)
            .map(|_| GLWECiphertext::alloc(&module, basek, k_glwe, rank))
            .collect_vec();
        let mut tmp_glwes_sharded = split_by_bounds_mut(&mut tmp_glwes, &tmp_glwes_bounds);
        let mut scratch_owned = ScratchOwned::new(add_scratch_space(
            &module, basek, k_glwe, k_glwe, k_ggsw, digit_ggsw, rank,
        ));

        // fixed terminal nodes (i.e. GlwePt of 0,1)
        let terminal_nodes_pt = {
            let mut pt0 = GLWEPlaintext::alloc(&module, basek, k_glwe);
            pt0.data.encode_coeff_i64(0, basek, k_pt, 0, 0, 1);

            let mut pt1 = GLWEPlaintext::alloc(&module, basek, k_glwe);
            pt1.data.encode_coeff_i64(0, basek, k_pt, 0, 1, 1);

            [pt0, pt1]
        };

        // Compute addition
        add(
            &inputs,
            &mut outputs,
            &mut tmp_glwes_sharded,
            &terminal_nodes_pt,
            scratch_owned.borrow(),
            &module,
        );

        let out_bits = outputs
            .iter()
            .map(|out| glwe_decrypt(&module, out, &sk_fourier, k_pt))
            .collect_vec();
        let out = bits_to_u32(&out_bits);

        assert!(out == a + b);
    }
}
