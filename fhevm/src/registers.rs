use core::{
    backend::{Decoding, Module, FFT64},
    glwe, FourierGLWESecret, GLWECiphertext, GLWEPlaintext, GLWESecret, Infos, ScratchOwned,
};

use fhe_ram::{self, address::Address, keys::gen_keys, parameters::Parameters, ram::Ram};
use itertools::izip;
use rand_core::RngCore;
use sampling::source::Source;

fn decrypt_glwe(
    module: &Module<FFT64>,
    glwe_ct: &GLWECiphertext<Vec<u8>>,
    sk: &FourierGLWESecret<Vec<u8>, FFT64>,
    k_pt: usize,
) -> u64 {
    let mut scratch_owned = ScratchOwned::new(GLWECiphertext::decrypt_scratch_space(
        module,
        glwe_ct.basek(),
        glwe_ct.k(),
    ));
    let scratch = scratch_owned.borrow();
    let mut pt = GLWEPlaintext::alloc(module, glwe_ct.basek(), k_pt);
    glwe_ct.decrypt(module, &mut pt, sk, scratch);

    let havei64 = pt.data.decode_coeff_i64(0, glwe_ct.basek(), k_pt, 0);
    if havei64.is_negative() {
        ((1 << k_pt) as i64 + havei64) as u64
    } else {
        havei64 as u64
    }
}

fn decrypt_glwe_noise(
    module: &Module<FFT64>,
    glwe_ct: &GLWECiphertext<Vec<u8>>,
    sk: &FourierGLWESecret<Vec<u8>, FFT64>,
    k_pt: usize,
    want: u8,
) -> f64 {
    let mut scratch_owned = ScratchOwned::new(GLWECiphertext::decrypt_scratch_space(
        module,
        glwe_ct.basek(),
        glwe_ct.k(),
    ));
    let scratch = scratch_owned.borrow();
    let mut pt = GLWEPlaintext::alloc(module, glwe_ct.basek(), glwe_ct.k());
    glwe_ct.decrypt(module, &mut pt, sk, scratch);

    let havei64 = pt.data.decode_coeff_i64(0, pt.basek(), pt.k(), 0);

    let scale = (1u64 << (pt.k() - k_pt)) as f64;
    let want_i64 = uint_to_i64(want.into(), 1 << k_pt);
    let diff = (want_i64 as f64) - ((havei64 as f64) / scale);
    diff.log2()
}

fn uint_to_i64(v: u64, modulus: u64) -> i64 {
    if v >= modulus / 2 {
        return -((modulus - v) as i64);
    } else {
        return v as i64;
    }
}

struct ArithCircuitsParams {
    basek: usize,
    k_glwe: usize,
    rank: usize,
    k_ggsw: usize,
    rank_ggsw: usize,
    digits_ggsw: usize,
    sigma: f64,
    log_n: usize,
}

struct ArithmeticHandler {}

impl ArithCircuitsParams {
    fn new(
        basek: usize,
        k_glwe: usize,
        rank: usize,
        k_ggsw: usize,
        rank_ggsw: usize,
        digits_ggsw: usize,
        sigma: f64,
        log_n: usize,
    ) -> ArithCircuitsParams {
        ArithCircuitsParams {
            basek,
            k_glwe,
            rank,
            k_ggsw,
            rank_ggsw,
            digits_ggsw,
            sigma,
            log_n,
        }
    }
}

struct Interpreter {
    imm_rom: Ram,
    rd_w_rom: Ram,
    pc_w_rom: Ram,
    mem_w_rom: Ram,
    registers: Ram,
    arith_params: ArithCircuitsParams,
}

static MAX_ROM: usize = 1 << 7; // in bytes
static MAX_INSTRUCTIONS: usize = MAX_ROM >> 2;
static SIGMA: f64 = 3.2;

impl Interpreter {
    fn new(logn: usize, basek: usize, rank: usize) -> Self {
        let registers = Ram::new(&Parameters::new(
            logn,
            basek,
            1,
            1,
            1,
            basek * 2,
            basek * 4,
            basek * 4,
            0.5,
            SIGMA,
            32,
            vec![logn as u8],
            32,
        ));

        let imm_rom = Ram::new(&Parameters::new(
            logn,
            basek,
            1,
            1,
            1,
            basek * 2,
            basek * 4,
            basek * 4,
            0.5,
            3.2,
            32,
            vec![logn as u8],
            32,
        ));

        let rd_w_rom = Ram::new(&Parameters::new(
            logn,
            basek,
            1,
            1,
            5,
            basek * 2,
            basek * 4,
            basek * 4,
            0.5,
            3.2,
            32,
            vec![logn as u8],
            3,
        ));

        let pc_w_rom = Ram::new(&Parameters::new(
            logn,
            basek,
            1,
            1,
            4,
            basek * 2,
            basek * 4,
            basek * 4,
            0.5,
            3.2,
            32,
            vec![logn as u8],
            1,
        ));

        let mem_w_rom = Ram::new(&Parameters::new(
            logn,
            basek,
            1,
            1,
            2,
            basek * 2,
            basek * 4,
            basek * 4,
            0.5,
            3.2,
            32,
            vec![logn as u8],
            1,
        ));

        // let pc = 4;

        Self {
            imm_rom,
            rd_w_rom,
            mem_w_rom,
            pc_w_rom,
            registers,
            arith_params: ArithCircuitsParams::new(
                basek,
                basek * 3,
                rank,
                basek * 4,
                rank,
                1,
                SIGMA,
                logn,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::{
        backend::{
            Encoding, Module, ScalarZnx, ScalarZnxAlloc, Stats, VecZnxScratch, ZnxView, ZnxViewMut,
        },
        GGLWECiphertext, GGSWCiphertext, GLWEAutomorphismKey, GLWEOps, GLWEPacker, Scratch,
    };
    use std::{collections::HashMap, fs::read};

    use fhe_ram::keys::{gen_eval_keys, EvaluationKeys};
    use itertools::Itertools;

    use crate::{
        arithmetic::{self},
        instructions::{r_type::add::Add, Instruction, InstructionsParser},
        utils::{split_by_bounds_mut, tests::u32_to_bits},
    };

    use super::*;

    // #[test]
    // fn inst() {
    //     // rd = 6, rs2 = 2, rs1 = 1, rd = rs2 + rs1
    //     // let instruction = Instruction::new(0b0000000_00010_00001_000_00110_0110011);
    //     // let mut inst_parser = InstructionsParser::new();
    //     // inst_parser.add(instruction);
    //
    //     let logn = 12;
    //     let basek = 20;
    //     let rank = 1;
    //
    //     let arith_params =
    //         ArithCircuitsParams::new(basek, basek * 3, rank, basek * 4, rank, 1, SIGMA, logn);
    //
    //     let mut source = Source::new([123; 32]);
    //
    //     // Setup keys
    //
    //     let module = Module::new(1 << logn);
    //     let mut sk = GLWESecret::alloc(&module, rank);
    //     sk.fill_binary_prob(0.5, &mut source);
    //     let sk_fourier = FourierGLWESecret::from(&module, &sk);
    //
    //     let mut scratch_owned = ScratchOwned::new(
    //         GLWEPacker::scratch_space(
    //             &module,
    //             basek,
    //             arith_params.k_glwe,
    //             arith_params.k_ggsw,
    //             arith_params.digits_ggsw,
    //             rank,
    //         ) | GLWEAutomorphismKey::encrypt_sk_scratch_space(
    //             &module,
    //             basek,
    //             arith_params.k_glwe,
    //             rank,
    //         ) | arithmetic::add::add_scratch_space(
    //             &module,
    //             arith_params.basek,
    //             arith_params.k_glwe,
    //             arith_params.k_glwe,
    //             arith_params.k_ggsw,
    //             arith_params.digits_ggsw,
    //             rank,
    //         ) | arithmetic::sub::sub_scratch_space(
    //             &module,
    //             arith_params.basek,
    //             arith_params.k_glwe,
    //             arith_params.k_glwe,
    //             arith_params.k_ggsw,
    //             arith_params.digits_ggsw,
    //             rank,
    //         ) | GGSWCiphertext::encrypt_sk_scratch_space(
    //             &module,
    //             arith_params.basek,
    //             arith_params.k_ggsw,
    //             rank,
    //         ),
    //     );
    //
    //     // evaluate arithmetic instructions
    //
    //     let terminal_nodes_pt = {
    //         let mut pt0 = GLWEPlaintext::alloc(&module, arith_params.basek, arith_params.k_glwe);
    //         pt0.data.encode_coeff_i64(0, pt0.basek(), pt0.k(), 0, 0, 1);
    //
    //         let mut pt1 = GLWEPlaintext::alloc(&module, arith_params.basek, arith_params.k_glwe);
    //         pt1.data.encode_coeff_i64(0, pt1.basek(), pt1.k(), 0, 1, 1);
    //
    //         [pt0, pt1]
    //     };
    //
    //     let a = 3;
    //     let b = 0;
    //     let ggsw_a = u32_to_bits(a)
    //         .iter()
    //         .map(|b| {
    //             let f = ggsw_encrypt_constant(
    //                 &module,
    //                 &sk_fourier,
    //                 *b as i64,
    //                 arith_params.basek,
    //                 arith_params.k_ggsw,
    //                 arith_params.k_glwe.div_ceil(arith_params.basek),
    //                 arith_params.digits_ggsw,
    //                 rank,
    //                 &mut scratch_owned.borrow(),
    //                 &mut source,
    //             );
    //             f
    //         })
    //         .collect_vec();
    //
    //     let ggsw_b = u32_to_bits(b)
    //         .iter()
    //         .map(|b| {
    //             ggsw_encrypt_constant(
    //                 &module,
    //                 &sk_fourier,
    //                 *b as i64,
    //                 arith_params.basek,
    //                 arith_params.k_ggsw,
    //                 arith_params.k_glwe.div_ceil(arith_params.basek),
    //                 arith_params.digits_ggsw,
    //                 rank,
    //                 &mut scratch_owned.borrow(),
    //                 &mut source,
    //             )
    //         })
    //         .collect_vec();
    //
    //     let input_ggsw_refs = ggsw_a
    //         .iter()
    //         .chain(ggsw_b.iter())
    //         .map(|ct| ct)
    //         .collect_vec();
    //
    //     {
    //         let mut glwe = GLWECiphertext::alloc(&module, basek, arith_params.k_glwe, rank);
    //         let mut pt = GLWEPlaintext::alloc(&module, basek, 1);
    //         pt.data.encode_coeff_i64(0, basek, 1, 0, 1, 1);
    //         glwe.encrypt_sk(
    //             &module,
    //             &pt,
    //             &sk_fourier,
    //             &mut source.branch(),
    //             &mut source.branch(),
    //             SIGMA,
    //             scratch_owned.borrow(),
    //         );
    //         glwe.external_product_inplace(&module, &input_ggsw_refs[0], scratch_owned.borrow());
    //
    //         let mut pt_back = GLWEPlaintext::alloc(&module, basek, glwe.k());
    //         glwe.decrypt(&module, &mut pt_back, &sk_fourier, scratch_owned.borrow());
    //
    //         println!("QWERT = {}", pt_back.data);
    //     }
    //
    //     let mut add_helper = AddHelper::new(
    //         &module,
    //         arith_params.basek,
    //         arith_params.k_glwe,
    //         arith_params.rank,
    //     );
    //
    //     let mut add_outputs = (0..WORD_SIZE)
    //         .map(|_| {
    //             GLWECiphertext::alloc(
    //                 &module,
    //                 arith_params.basek,
    //                 arith_params.k_glwe,
    //                 arith_params.rank,
    //             )
    //         })
    //         .collect_vec();
    //
    //     let mut s2 = ScratchOwned::new(add_scratch_space(
    //         &module,
    //         basek,
    //         arith_params.k_glwe,
    //         arith_params.k_glwe,
    //         arith_params.k_ggsw,
    //         arith_params.digits_ggsw,
    //         rank,
    //     ));
    //     arithmetic::add::add(
    //         &input_ggsw_refs,
    //         &mut add_outputs,
    //         &mut add_helper.tmp_glwes_mut(),
    //         &terminal_nodes_pt,
    //         s2.borrow(),
    //         &module,
    //     );
    //
    //     {
    //         let mut s2 = ScratchOwned::new(add_scratch_space(
    //             &module,
    //             basek,
    //             arith_params.k_glwe,
    //             arith_params.k_glwe,
    //             arith_params.k_ggsw,
    //             arith_params.digits_ggsw,
    //             rank,
    //         ));
    //         for i in 0..5 {
    //             let ct = &add_outputs[i];
    //             let mut pt = GLWEPlaintext::alloc(&module, basek, ct.k());
    //             ct.decrypt(&module, &mut pt, &sk_fourier, s2.borrow());
    //             println!("add_outputs[{i}] : {}", pt.data);
    //         }
    //     }
    //
    //     let mut sub_helper = SubHelper::new(
    //         &module,
    //         arith_params.basek,
    //         arith_params.k_glwe,
    //         arith_params.rank,
    //     );
    //
    //     let mut sub_outputs = (0..WORD_SIZE)
    //         .map(|_| {
    //             GLWECiphertext::alloc(
    //                 &module,
    //                 arith_params.basek,
    //                 arith_params.k_glwe,
    //                 arith_params.rank,
    //             )
    //         })
    //         .collect_vec();
    //     let mut packed_outputs: Vec<GLWECiphertext<Vec<u8>>> = (0..WORD_SIZE)
    //         .map(|_| {
    //             GLWECiphertext::alloc(
    //                 &module,
    //                 arith_params.basek,
    //                 arith_params.k_glwe,
    //                 arith_params.rank,
    //             )
    //         })
    //         .collect_vec();
    //
    //     arithmetic::sub::sub(
    //         &input_ggsw_refs,
    //         &mut sub_outputs,
    //         &mut sub_helper.tmp_glwes_mut(),
    //         &terminal_nodes_pt,
    //         scratch_owned.borrow(),
    //         &module,
    //     );
    //
    //     let add_id = 1;
    //     let sub_id = 2;
    //     let log_max_id = 5;
    //     let mut glwe_zero = GLWECiphertext::alloc(
    //         &module,
    //         arith_params.basek,
    //         arith_params.k_glwe,
    //         arith_params.rank,
    //     );
    //     glwe_zero.encrypt_zero_sk(
    //         &module,
    //         &sk_fourier,
    //         &mut source.branch(),
    //         &mut source.branch(),
    //         SIGMA,
    //         scratch_owned.borrow(),
    //     );
    //     let mut pack = [[&glwe_zero; 32]; WORD_SIZE];
    //
    //     [(add_id, &add_outputs), (sub_id, &sub_outputs)]
    //         .into_iter()
    //         .for_each(|(id, outputs)| {
    //             (0..32).for_each(|bit_index| {
    //                 pack[bit_index][reverse_bits(id, log_max_id)] = &outputs[bit_index];
    //             });
    //         });
    //
    //     // Instantiate packer and generate packing keys
    //     let mut glwe_packer = GLWEPacker::new(
    //         &module,
    //         arith_params.log_n - 5,
    //         arith_params.basek,
    //         arith_params.k_glwe,
    //         arith_params.rank,
    //     );
    //     let mut pack_auto_keys: HashMap<i64, GLWEAutomorphismKey<Vec<u8>, FFT64>> = HashMap::new();
    //     let gal_els: Vec<i64> = GLWEPacker::galois_elements(&module);
    //     gal_els.iter().for_each(|gal_el| {
    //         let mut key = GLWEAutomorphismKey::alloc(
    //             &module,
    //             arith_params.basek,
    //             arith_params.k_ggsw,
    //             arith_params.k_glwe / arith_params.basek,
    //             arith_params.digits_ggsw,
    //             rank,
    //         );
    //         key.encrypt_sk(
    //             &module,
    //             *gal_el,
    //             &sk,
    //             &mut source.branch(),
    //             &mut source.branch(),
    //             SIGMA,
    //             scratch_owned.borrow(),
    //         );
    //
    //         pack_auto_keys.insert(*gal_el, key);
    //     });
    //
    //     izip!(packed_outputs.iter_mut(), pack.iter()).for_each(|(res, to_pack)| {
    //         pack32(
    //             &module,
    //             to_pack,
    //             res,
    //             &mut glwe_packer,
    //             &pack_auto_keys,
    //             scratch_owned.borrow(),
    //         );
    //     });
    //
    //     // {
    //     //     packed_outputs
    //     //         .iter()
    //     //         .take(3)
    //     //         .enumerate()
    //     //         .for_each(|(index, glwe_packed)| {
    //     //             let mut pt = GLWEPlaintext::alloc(&module, basek, glwe_packed.k());
    //     //             glwe_packed.decrypt(&module, &mut pt, &sk_fourier, scratch_owned.borrow());
    //     //             println!("Packed PT {index} : {}", pt.data);
    //     //         });
    //     // }
    //
    //     // GGSW(X^{-sub_id})
    //     let ggsw_selector = {
    //         let mut scalar = ScalarZnx::<Vec<u8>>::new(module.n(), 1);
    //         scalar.raw_mut()[module.n() - add_id] = -1;
    //         let mut ggsw = GGSWCiphertext::alloc(
    //             &module,
    //             basek,
    //             arith_params.k_ggsw,
    //             packed_outputs[0].size(),
    //             arith_params.digits_ggsw,
    //             rank,
    //         );
    //         ggsw.encrypt_sk(
    //             &module,
    //             &scalar,
    //             &sk_fourier,
    //             &mut source.branch(),
    //             &mut source.branch(),
    //             SIGMA,
    //             scratch_owned.borrow(),
    //         );
    //         ggsw
    //     };
    //
    //     packed_outputs.iter_mut().for_each(|ct| {
    //         ct.external_product_inplace(&module, &ggsw_selector, scratch_owned.borrow());
    //     });
    //
    //     let want_out = u32_to_bits(a.wrapping_add(b));
    //
    //     izip!(want_out.iter(), packed_outputs.iter()).for_each(|(want_bit, ct)| {
    //         let mut pt_have = GLWEPlaintext::alloc(&module, basek, ct.k());
    //         ct.decrypt(&module, &mut pt_have, &sk_fourier, scratch_owned.borrow());
    //
    //         let mut pt_want = GLWEPlaintext::alloc(&module, basek, 1);
    //         pt_want
    //             .data
    //             .encode_coeff_i64(0, basek, pt_want.k(), 0, *want_bit as i64, 1);
    //
    //         pt_have.sub_inplace_ab(&module, &pt_want);
    //
    //         // println!("Noise: {}", pt_have.data.std(0, basek).log2());
    //     });
    //
    //     // Write final outputs back to register
    // }

    fn pack32(
        module: &Module<FFT64>,
        ct_list: &[&GLWECiphertext<Vec<u8>>],
        res: &mut GLWECiphertext<Vec<u8>>,
        glwe_packer: &mut GLWEPacker,
        auto_keys: &HashMap<i64, GLWEAutomorphismKey<Vec<u8>, FFT64>>,
        scratch: &mut Scratch,
    ) {
        assert!(ct_list.len() == 32);
        ct_list
            .iter()
            .for_each(|c| glwe_packer.add(module, Some(c), auto_keys, scratch));
        glwe_packer.flush(module, res);
    }

    fn reverse_bits(v: usize, bits: usize) -> usize {
        v.reverse_bits() >> (usize::BITS - bits as u32)
    }

    // fn circuit_bootstrap_monomial(
    //     module: &Module<FFT64>,
    //     glwe_ct: &GLWECiphertext<Vec<u8>>,
    //     k_pt_glwe_ct: usize,
    //     sk: &FourierGLWESecret<Vec<u8>, FFT64>,
    //     k_ggsw: usize,
    //     digit_ggsw: usize,
    //     rank_ggsw: usize,
    //     source: &mut Source,
    // ) -> GGLWECiphertext<Vec<u8>, FFT64> {

    // }
    fn ggsw_encrypt_constant(
        module: &Module<FFT64>,
        sk: &FourierGLWESecret<Vec<u8>, FFT64>,
        value: i64,
        basek: usize,
        k_ggsw: usize,
        rows: usize,
        digit_ggsw: usize,
        rank_ggsw: usize,
        scratch: &mut Scratch,
        source: &mut Source,
    ) -> GGSWCiphertext<Vec<u8>, FFT64> {
        let mut pt = ScalarZnx::<Vec<u8>>::new(module.n(), 1);
        pt.raw_mut()[0] = value;
        // dbg!(pt.raw_mut()[0]);
        let mut ggsw_ct =
            GGSWCiphertext::alloc(&module, basek, k_ggsw, rows, digit_ggsw, rank_ggsw);
        ggsw_ct.encrypt_sk(
            &module,
            &pt,
            &sk,
            &mut source.branch(),
            &mut source.branch(),
            SIGMA,
            scratch,
        );
        ggsw_ct
    }

    fn circuit_bootstrap(
        module: &Module<FFT64>,
        glwe_ct: &GLWECiphertext<Vec<u8>>,
        k_pt_glwe_ct: usize,
        sk: &FourierGLWESecret<Vec<u8>, FFT64>,
        k_ggsw: usize,
        digit_ggsw: usize,
        rank_ggsw: usize,
        source: &mut Source,
    ) -> GGSWCiphertext<Vec<u8>, FFT64> {
        let v = decrypt_glwe(module, glwe_ct, &sk, k_pt_glwe_ct);
        assert!(v == 0 || v == 1, "invalid v={v}");
        let mut ggsw_ct = GGSWCiphertext::alloc(
            module,
            glwe_ct.basek(),
            k_ggsw,
            glwe_ct.rows(),
            digit_ggsw,
            rank_ggsw,
        );
        let mut source1 = source.branch();
        let mut source2 = source.branch();

        let mut scratch_owned = ScratchOwned::new(
            module.bytes_of_scalar_znx(1)
                + GGSWCiphertext::encrypt_sk_scratch_space(
                    module,
                    glwe_ct.basek(),
                    k_ggsw,
                    rank_ggsw,
                ),
        );

        let (mut pt, scratch) = scratch_owned.borrow().tmp_scalar_znx(module, 1);
        pt.raw_mut()[0] = v as i64;
        ggsw_ct.encrypt_sk(module, &pt, sk, &mut source1, &mut source2, SIGMA, scratch);
        ggsw_ct
    }

    fn read_register(
        module: &Module<FFT64>,
        registers: &mut Ram,
        idx_glwe: &GLWECiphertext<Vec<u8>>,
        k_pt_idx_glwe: usize,
        sk: &FourierGLWESecret<Vec<u8>, FFT64>,
        eval_keys: &EvaluationKeys,
        source: &mut Source,
    ) -> Vec<GLWECiphertext<Vec<u8>>> {
        let idx = decrypt_glwe(module, idx_glwe, &sk, k_pt_idx_glwe);
        let mut idx_add = Address::alloc(registers.params());
        idx_add.encrypt_sk(registers.params(), idx as u32, &sk, source);
        registers.read(&idx_add, eval_keys)
    }
}

// {
//     // Note: re-using eval keys for all ROMs for now
//     let rd_w_eval_keys = gen_eval_keys(interpreter.rd_w_rom.params(), &sk, &mut source);

//     // Setup ROMs
//     {
//         // rd_w ROM
//         let rd_w_rom_data = (0..inst_parser.max_count())
//             .flat_map(|idx| {
//                 let op_reg = inst_parser.get_opregisters(idx);
//                 [op_reg.rs2(), op_reg.rs1(), op_reg.rsd()]
//             })
//             .collect_vec();

//         interpreter
//             .rd_w_rom
//             .encrypt_sk(&rd_w_rom_data, &sk_fourier, &mut source);
//     }
//     {
//         // registers
//         let reg_data = vec![
//             0u8;
//             interpreter.registers.params().word_size()
//                 * interpreter.registers.params().max_addr()
//         ];

//         interpreter
//             .registers
//             .encrypt_sk(&reg_data, &sk_fourier, &mut source);
//     }

//     // Read ROMs at pc //
//     let pc = 0;
//     let mut rd_w_add = {
//         let mut address = Address::alloc(interpreter.rd_w_rom.params());
//         address.encrypt_sk(interpreter.rd_w_rom.params(), pc, &sk_fourier, &mut source);
//         address
//     };

//     let rd_w_out = interpreter.rd_w_rom.read(&rd_w_add, &rd_w_eval_keys);
//     let rs2_glwe = &rd_w_out[0];
//     let rs1_glwe = &rd_w_out[1];
//     let rd_glwe = &rd_w_out[2];

//     // Read registers rs2, rs1, (prepwrite) rd

//     let value_rs2_glwe = read_register(
//         &module,
//         &mut interpreter.registers,
//         rs2_glwe,
//         interpreter.rd_w_rom.params().k_pt().clone(),
//         &sk_fourier,
//         &rd_w_eval_keys,
//         &mut source,
//     );
//     let value_rs1_glwe = read_register(
//         &module,
//         &mut interpreter.registers,
//         rs1_glwe,
//         interpreter.rd_w_rom.params().k_pt(),
//         &sk_fourier,
//         &rd_w_eval_keys,
//         &mut source,
//     );

//     let value_rd_glwe = {
//         let rd = decrypt_glwe(
//             &module,
//             rd_glwe,
//             &sk_fourier,
//             interpreter.rd_w_rom.params().k_pt(),
//         );
//         let mut rd_add = Address::alloc(interpreter.registers.params());
//         rd_add.encrypt_sk(
//             interpreter.registers.params(),
//             rd as u32,
//             &sk_fourier,
//             &mut source,
//         );
//         interpreter
//             .registers
//             .read_prepare_write(&rd_add, &rd_w_eval_keys)
//     };
// }
