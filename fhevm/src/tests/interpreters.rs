use crate::{
    Base1D, Instruction, InstructionsParser, Interpreter, keys::{RAMKeys, RAMKeysPrepared}, parameters::{CryptographicParameters, DECOMP_N}
};
use poulpy_backend::FFT64Ref;
use poulpy_core::{GLWEDecrypt, SIGMA, layouts::{GGSWLayout, GLWE, GLWEInfos, GLWEPlaintext, GLWESecret, GLWESecretPrepared, LWEInfos, LWESecret}};
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow, ScratchTakeBasic, VecZnxRotateInplace},
    layouts::{Backend, Module, ScalarZnx, Scratch, ScratchOwned, ZnxView, ZnxViewMut},
    source::Source,
};
use poulpy_schemes::tfhe::bdd_arithmetic::FheUint;

#[test]
pub fn test_interpreter_prepared_pc() {
    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();

    let module: &Module<FFT64Ref> = params.module();

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 24);

    let mut interpreter: Interpreter<FFT64Ref> =
        Interpreter::new(&params, 1 << 10, 1 << 10, DECOMP_N.into());

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([0u8; 32]);
    let mut source_xe: Source = Source::new([0u8; 32]);

    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    // sk_glwe.fill_zero();

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc(module, sk_glwe.rank());
    sk_glwe_prepared.prepare(module, &sk_glwe);

    // let instruction_u32 = 258455;
    let mut instructions: InstructionsParser = InstructionsParser::new();
    for i in 0..1<<10 {
        instructions.add(Instruction::new(258455));
    }
    // instructions.add(Instruction::new(33653139));
    // instructions.add(Instruction::new(512279));
    // instructions.add(Instruction::new(512279));
    // instructions.add(Instruction::new(512279));
    // instructions.add(Instruction::new(512279));

    interpreter.instructions_encrypt_sk(
        module,
        &instructions,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let idx = 1;

    let instruction: Instruction = instructions.get_raw(idx);
    let correct_imm: u32 = instruction.get_immediate();

    interpreter.pc_encrypt_sk(
        module,
        idx as u32,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let key: RAMKeys<Vec<u8>> =
        RAMKeys::encrypt_sk(&params, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: RAMKeysPrepared<Vec<u8>, FFT64Ref> = RAMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    interpreter.pc_addr
        .set_from_fheuint_prepared(module, &interpreter.pc_fhe_uint_prepared, scratch.borrow());

    let ggsw_res_infos: GGSWLayout = params.ggsw_infos();
    let max_noise = |col_i: usize| {
        let mut noise: f64 =
            -(ggsw_res_infos.size() as f64 * params.basek().as_usize() as f64) + SIGMA.log2() + 3.0;
        noise += 0.5 * ggsw_res_infos.log_n() as f64;
        if col_i != 0 {
            noise += 0.5 * ggsw_res_infos.log_n() as f64
        }
        noise
    };    
    let mut bit_rsh: usize = 0;
    for coordinate in interpreter.pc_addr.coordinates.iter_mut() {
        let mut bit_lsh: usize = 0;
        let base1d: Base1D = coordinate.base1d.clone();

        for (ggsw, bit_mask) in coordinate.value.iter_mut().zip(base1d.0) {
            let mask: u32 = (1 << bit_mask) - 1;

            let mut pt: ScalarZnx<Vec<u8>> = ScalarZnx::alloc(module.n(), 1);
            pt.raw_mut()[0] = 1;

            let rot: i64 = (((idx as u32 >> bit_rsh) & mask) << bit_lsh) as i64;

            module.vec_znx_rotate_inplace(rot, &mut pt.as_vec_znx_mut(), 0, scratch.borrow());

            ggsw.assert_noise(module, &sk_glwe_prepared, &pt, &max_noise);
            bit_lsh += bit_mask as usize;
            bit_rsh += bit_mask as usize;
        }
    }

    // normal way of doing it
    interpreter.read_instruction_components(module, &key_prepared, scratch.borrow());
    let dec_imm: u32 =
        interpreter
            .imm_val_fhe_uint
            .decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());

    println!("dec_imm: {}", dec_imm);
    println!("correct_imm: {}", correct_imm);

    assert_eq!(correct_imm, dec_imm);
}

#[test]
pub fn test_interpreter_init_one_instruction() {
    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();

    let module: &Module<FFT64Ref> = params.module();

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 22);

    let mut interpreter: Interpreter<FFT64Ref> =
        Interpreter::new(&params, 1 << 10, 1 << 10, DECOMP_N.into());

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([0u8; 32]);
    let mut source_xe: Source = Source::new([0u8; 32]);

    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc(module, sk_glwe.rank());
    sk_glwe_prepared.prepare(module, &sk_glwe);

    let instruction_u32 = 258455;
    let mut instructions: InstructionsParser = InstructionsParser::new();
    instructions.add(Instruction::new(instruction_u32));

    interpreter.instructions_encrypt_sk(
        module,
        &instructions,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let idx = 0;

    let instruction: Instruction = instructions.get_raw(idx);
    let correct_imm: u32 = instruction.get_immediate();
    let correct_rs1: u32 = instruction.get_rs1_or_zero() as u32;
    let correct_rs2: u32 = instruction.get_rs2_or_zero() as u32;
    let correct_rd: u32 = instruction.get_rd_or_zero() as u32;
    let (rdu, mu, pcu) = instruction.get_opid();
    let correct_rdu: u32 = rdu as u32;
    let correct_mu: u32 = mu as u32;
    let correct_pcu: u32 = pcu as u32;

    interpreter.pc_encrypt_sk(
        module,
        idx as u32,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let key: RAMKeys<Vec<u8>> =
        RAMKeys::encrypt_sk(&params, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: RAMKeysPrepared<Vec<u8>, FFT64Ref> = RAMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    interpreter.read_instruction_components(module, &key_prepared, scratch.borrow());

    let dec_rs1: u32 =
        interpreter
            .rs1_addr_fhe_uint
            .decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    let dec_rs2: u32 =
        interpreter
            .rs2_addr_fhe_uint
            .decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    let dec_rd: u32 =
        interpreter
            .rd_addr_fhe_uint
            .decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    let dec_imm: u32 =
        interpreter
            .imm_val_fhe_uint
            .decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());

    let dec_rdu: u32 =
        interpreter
            .rdu_val_fhe_uint
            .decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    let dec_mu: u32 =
        interpreter
            .mu_val_fhe_uint
            .decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    let dec_pcu: u32 =
        interpreter
            .pcu_val_fhe_uint
            .decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());

    println!(
        "{} {} {} {} {} {} {}",
        correct_imm, correct_rs1, correct_rs2, correct_rd, correct_rdu, correct_mu, correct_pcu
    );
    println!(
        "{} {} {} {} {} {} {}",
        dec_imm, dec_rs1, dec_rs2, dec_rd, dec_rdu, dec_mu, dec_pcu
    );

    assert_eq!(correct_imm, dec_imm);
    assert_eq!(correct_rs1, dec_rs1);
    assert_eq!(correct_rs2, dec_rs2);
    assert_eq!(correct_rd, dec_rd);
    assert_eq!(correct_rdu, dec_rdu);
    assert_eq!(correct_mu, dec_mu);
    assert_eq!(correct_pcu, dec_pcu);
}

#[test] //wip, not working for pc > 0
pub fn test_interpreter_init_many_instructions() {
    use crate::Instruction;

    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();
    let module: &Module<FFT64Ref> = params.module();

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([0u8; 32]);
    let mut source_xe: Source = Source::new([0u8; 32]);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 24);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);

    let mut interpreter: Interpreter<FFT64Ref> = Interpreter::new(&params, 1 << 10, 1 << 10, DECOMP_N.into());

    let instructions_u32 = vec![
        258455,
        33653139,
        // 512279, 4286644499, 66579, 10507363, 3221229683, 8388847, 3221229683,
        // 791, 8585319, 259383,
    ];

    let mut instructions = InstructionsParser::new();
    for inst in instructions_u32.clone() {
        instructions.add(Instruction::new(inst));
    }

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc(module, sk_glwe.rank());
    sk_glwe_prepared.prepare(module, &sk_glwe);

    interpreter.instructions_encrypt_sk(
        module,
        &instructions,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );
    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc_from_infos(module, &params.glwe_ct_infos());
    sk_glwe_prepared.prepare(module, &sk_glwe);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 24);

    let key: RAMKeys<Vec<u8>> =
        RAMKeys::encrypt_sk(&params, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: RAMKeysPrepared<Vec<u8>, FFT64Ref> = RAMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    for idx in 0..instructions_u32.len() {
        let instruction = instructions.get_raw(idx);
        let correct_imm = instruction.get_immediate();
        let correct_rs1 = instruction.get_rs1_or_zero() as u32;
        let correct_rs2 = instruction.get_rs2_or_zero() as u32;

        interpreter.pc_encrypt_sk(
            module,
            idx as u32,
            &sk_glwe_prepared,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );
        interpreter.read_instruction_components(module, &key_prepared, scratch.borrow());

        let dec_rs1: u32 = interpreter.rs1_val_fhe_uint.decrypt(
            module,
            &sk_glwe_prepared,
            scratch.borrow(),
        );
        let dec_rs2: u32 = interpreter.rs2_val_fhe_uint.decrypt(
            module,
            &sk_glwe_prepared,
            scratch.borrow(),
        );
        let dec_imm: u32 = interpreter.imm_val_fhe_uint.decrypt(
            module,
            &sk_glwe_prepared,
            scratch.borrow(),
        );

        assert_eq!(correct_imm, dec_imm);
        assert_eq!(correct_rs1, dec_rs1);
        assert_eq!(correct_rs2, dec_rs2);
    }
}


fn decrypt_glwe<B: Backend>(
    params: &CryptographicParameters<B>,
    ct: &GLWE<Vec<u8>>,
    want: i64,
    sk: &GLWESecretPrepared<Vec<u8>, B>,
  ) -> (i64, f64)
  where
    ScratchOwned<B>: ScratchOwnedAlloc<B> + ScratchOwnedBorrow<B>,
    Module<B>: GLWEDecrypt<B>,
    Scratch<B>: ScratchTakeBasic,
  {
    let module: &Module<B> = params.module();
  
    let mut pt: GLWEPlaintext<Vec<u8>> = GLWEPlaintext::alloc_from_infos(ct);
    let mut scratch: ScratchOwned<B> = ScratchOwned::alloc(GLWE::decrypt_tmp_bytes(module, ct));
  
    ct.decrypt(module, &mut pt, sk, scratch.borrow());
  
    let log_scale: usize = pt.k().as_usize() - params.k_glwe_pt().as_usize();
    let decrypted_value_before_scale: i64 = pt.decode_coeff_i64(pt.k(), 0);
    let diff: i64 = decrypted_value_before_scale - (want << log_scale);
    let noise: f64 = (diff.abs() as f64).log2() - pt.k().as_usize() as f64;
    let decrypted_value: i64 =
        (decrypted_value_before_scale as f64 / f64::exp2(log_scale as f64)).round() as i64;
    (decrypted_value, noise)
  }


