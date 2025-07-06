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

struct ArithGGSWParams {
    basek: usize,
    k_ct: usize,
    rank: usize,
    digits: usize,
    sigma: f64,
    log_n: usize,
}

struct Interpreter {
    imm_rom: Ram,
    rd_w_rom: Ram,
    pc_w_rom: Ram,
    mem_w_rom: Ram,
    registers: Ram,
    arith_ggsw_params: ArithGGSWParams,
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
            arith_ggsw_params: ArithGGSWParams {
                basek,
                k_ct: basek * 2,
                rank,
                sigma: SIGMA,
                digits: 1,
                log_n: logn,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use core::{
        backend::{Encoding, Module, ScalarZnx, ScalarZnxAlloc, ZnxViewMut},
        GGLWECiphertext, GGSWCiphertext, Scratch,
    };
    use std::fs::read;

    use fhe_ram::keys::{gen_eval_keys, EvaluationKeys};
    use itertools::Itertools;

    use crate::{
        arithmetic::{
            self,
            add::{self, tmp_glwe_bounds},
        },
        instructions::{r_type::add::Add, Instruction, InstructionsParser},
    };

    use super::*;

    #[test]
    fn inst() {
        // rd = 6, rs2 = 2, rs1 = 1, rd = rs2 + rs1
        let instruction = Instruction::new(0b0000000_00010_00001_000_00110_0110011);
        let mut inst_parser = InstructionsParser::new();
        inst_parser.add(instruction);

        let logn = 12;
        let basek = 20;
        let rank = 1;

        let mut interpreter = Interpreter::new(logn, basek, rank);
        let mut source = Source::new([12; 32]);

        // Setup keys

        let module = interpreter.registers.params().module().clone();
        let mut sk = GLWESecret::alloc(&module, interpreter.registers.params().rank());
        sk.fill_binary_prob(0.5, &mut source);
        let sk_fourier = FourierGLWESecret::from(&module, &sk);

        // Note: re-using eval keys for all ROMs for now
        let rd_w_eval_keys = gen_eval_keys(interpreter.rd_w_rom.params(), &sk, &mut source);

        // Setup ROMs
        {
            // rd_w ROM
            let rd_w_rom_data = (0..inst_parser.max_count())
                .flat_map(|idx| {
                    let op_reg = inst_parser.get_opregisters(idx);
                    [op_reg.rs2(), op_reg.rs1(), op_reg.rsd()]
                })
                .collect_vec();

            interpreter
                .rd_w_rom
                .encrypt_sk(&rd_w_rom_data, &sk_fourier, &mut source);
        }
        {
            // registers
            let reg_data = vec![
                0u8;
                interpreter.registers.params().word_size()
                    * interpreter.registers.params().max_addr()
            ];

            interpreter
                .registers
                .encrypt_sk(&reg_data, &sk_fourier, &mut source);
        }

        // Read ROMs at pc //
        let pc = 0;
        let mut rd_w_add = {
            let mut address = Address::alloc(interpreter.rd_w_rom.params());
            address.encrypt_sk(interpreter.rd_w_rom.params(), pc, &sk_fourier, &mut source);
            address
        };

        let rd_w_out = interpreter.rd_w_rom.read(&rd_w_add, &rd_w_eval_keys);
        let rs2_glwe = &rd_w_out[0];
        let rs1_glwe = &rd_w_out[1];
        let rd_glwe = &rd_w_out[2];

        // Read registers rs2, rs1, (prepwrite) rd

        let value_rs2_glwe = read_register(
            &module,
            &mut interpreter.registers,
            rs2_glwe,
            interpreter.rd_w_rom.params().k_pt().clone(),
            &sk_fourier,
            &rd_w_eval_keys,
            &mut source,
        );
        let value_rs1_glwe = read_register(
            &module,
            &mut interpreter.registers,
            rs1_glwe,
            interpreter.rd_w_rom.params().k_pt(),
            &sk_fourier,
            &rd_w_eval_keys,
            &mut source,
        );

        let value_rd_glwe = {
            let rd = decrypt_glwe(
                &module,
                rd_glwe,
                &sk_fourier,
                interpreter.rd_w_rom.params().k_pt(),
            );
            let mut rd_add = Address::alloc(interpreter.registers.params());
            rd_add.encrypt_sk(
                interpreter.registers.params(),
                rd as u32,
                &sk_fourier,
                &mut source,
            );
            interpreter
                .registers
                .read_prepare_write(&rd_add, &rd_w_eval_keys)
        };

        // evaluate arithmetic instructions

        let k_ggsw = basek * 4;
        let rank_ggsw = 1;
        let digit_ggsw = 1;
        let k_pt = 1;
        let input_ggsws = value_rs1_glwe
            .iter()
            .chain(value_rs2_glwe.iter())
            .map(|glwe| {
                circuit_bootstrap(
                    &module,
                    glwe,
                    interpreter.registers.params().k_pt(),
                    &sk_fourier,
                    k_ggsw,
                    digit_ggsw,
                    rank_ggsw,
                    &mut source,
                )
            })
            .collect_vec();
        let input_ggsw_refs = input_ggsws.iter().map(|v| v).collect_vec();
        {
            let mut add_outputs = (0..32)
                .map(|_| GLWECiphertext::alloc(&module, basek, basek * 2, rank))
                .collect_vec();

            let (tmp_glwes_count, tmp_glwes_bounds) = arithmetic::add::tmp_glwe_bounds();
            let mut tmp_glwes = (0..tmp_glwes_count)
                .map(|_| GLWECiphertext::alloc(&module, basek, basek * 2, rank))
                .collect_vec();
            let mut tmp_glwes_sharded = split_by_bounds_mut(&mut tmp_glwes, &tmp_glwes_bounds);

            let mut scratch_owned = ScratchOwned::new(arithmetic::add::add_scratch_space(
                &module,
                basek,
                add_outputs[0].k(),
                add_outputs[0].k(),
                k_ggsw,
                digit_ggsw,
                rank,
            ));

            let terminal_nodes_pt = {
                let mut pt0 = GLWEPlaintext::alloc(&module, basek, basek * 2);
                pt0.data.encode_coeff_i64(0, basek, k_pt, 0, 0, 1);

                let mut pt1 = GLWEPlaintext::alloc(&module, basek, basek * 2);
                pt1.data.encode_coeff_i64(0, basek, k_pt, 0, 1, 1);

                [pt0, pt1]
            };

            arithmetic::add::add(
                &input_ggsw_refs,
                &mut add_outputs,
                tmp_glwes_sharded.as_mut_slice(),
                &terminal_nodes_pt,
                scratch_owned.borrow(),
                &module,
            );
        }
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

    #[test]
    fn dd() {
        // let log_n = 12;
        // let basek = 20;
        // let register_params = Parameters::new(
        //     log_n,
        //     basek,
        //     1,
        //     1,
        //     1,
        //     basek * 2,
        //     basek * 4,
        //     basek * 4,
        //     0.5,
        //     3.2,
        //     32,
        //     vec![log_n as u8],
        //     1,
        // );

        // let mut source = Source::new([53u8; 32]);

        // let (sk, eval_keys) = gen_keys(&register_params, &mut source);

        // let sk_fourier = FourierGLWESecret::from(register_params.module(), &sk);

        // let mut data = vec![0u8; register_params.max_addr()];
        // data.iter_mut()
        //     .for_each(|v| *v = (source.next_u32() as u8) & 1);

        // let mut registers = Ram::new(&register_params);
        // registers.encrypt_sk(&data, &sk_fourier, &mut source);

        // // read
        // let mut address = Address::alloc(&register_params);

        // address.encrypt_sk(&register_params, 6, &sk_fourier, &mut source);

        // let res_glwe = registers.read(&address, &eval_keys);
        // assert!(res_glwe.len() == 1);
        // let noise = decrypt_glwe_noise(&res_glwe[0], register_params, &sk_fourier, data[3]);
        // dbg!(noise);
    }
}
