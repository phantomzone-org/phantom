use crate::{
    keys::{RAMKeys, RAMKeysPrepared},
    parameters::{CryptographicParameters, DECOMP_N},
    Instruction, InstructionsParser, Interpreter,
};
use poulpy_backend::FFT64Ref;
use poulpy_core::layouts::{GLWEInfos, GLWESecret, GLWESecretPrepared};
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{Module, ScratchOwned},
    source::Source,
};

#[test]
pub fn test_interpreter_init_one_instruction() {
    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();

    let module: &Module<FFT64Ref> = params.module();

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 22);

    let rom_size = 1 << 10;
    let ram_size = 1 << 10;
    let mut interpreter: Interpreter<FFT64Ref> =
        Interpreter::new(&params, rom_size, ram_size, DECOMP_N.into());

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
    let (rs1, rs2, rd) = instruction.get_registers();
    let correct_rs1: u32 = rs1 as u32;
    let correct_rs2: u32 = rs2 as u32;
    let correct_rd: u32 = rd as u32;

    let (rdu, mu, pcu) = instruction.get_opid();
    let correct_rdu: u32 = rdu as u32;
    let correct_mu: u32 = mu as u32;
    let correct_pcu: u32 = pcu as u32;

    interpreter.pc_fhe_uint_prepared.encrypt_sk(
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

#[test]
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

    let mut interpreter: Interpreter<FFT64Ref> =
        Interpreter::new(&params, 1 << 10, 1 << 10, DECOMP_N.into());

    let instructions_u32 = vec![
        258455, 33653139, 512279, 4286644499, 66579, 10507363, 3221229683, 8388847, 3221229683,
        791, 8585319, 259383,
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

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 24);

    let key: RAMKeys<Vec<u8>> =
        RAMKeys::encrypt_sk(&params, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: RAMKeysPrepared<Vec<u8>, FFT64Ref> = RAMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    for idx in 0..instructions_u32.len() {
        let instruction = instructions.get_raw(idx);
        let correct_imm = instruction.get_immediate();
        let (rs1, rs2, rd) = instruction.get_registers();
        let correct_rs1: u32 = rs1 as u32;
        let correct_rs2: u32 = rs2 as u32;
        let correct_rd: u32 = rd as u32;
        let (rdu, mu, pcu) = instruction.get_opid();
        let correct_rdu: u32 = rdu as u32;
        let correct_mu: u32 = mu as u32;
        let correct_pcu: u32 = pcu as u32;

        interpreter.pc_fhe_uint_prepared.encrypt_sk(
            module,
            idx as u32,
            &sk_glwe_prepared,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );
        interpreter.read_instruction_components(module, &key_prepared, scratch.borrow());

        let dec_rs1: u32 =
            interpreter
                .rs1_addr_fhe_uint
                .decrypt(module, &sk_glwe_prepared, scratch.borrow());
        let dec_rs2: u32 =
            interpreter
                .rs2_addr_fhe_uint
                .decrypt(module, &sk_glwe_prepared, scratch.borrow());
        let dec_rd: u32 =
            interpreter
                .rd_addr_fhe_uint
                .decrypt(module, &sk_glwe_prepared, scratch.borrow());
        let dec_imm: u32 =
            interpreter
                .imm_val_fhe_uint
                .decrypt(module, &sk_glwe_prepared, scratch.borrow());

        let dec_rdu: u32 = interpreter.rdu_val_fhe_uint.decrypt(
            params.module(),
            &sk_glwe_prepared,
            scratch.borrow(),
        );
        let dec_mu: u32 = interpreter.mu_val_fhe_uint.decrypt(
            params.module(),
            &sk_glwe_prepared,
            scratch.borrow(),
        );
        let dec_pcu: u32 = interpreter.pcu_val_fhe_uint.decrypt(
            params.module(),
            &sk_glwe_prepared,
            scratch.borrow(),
        );

        // println!(
        //     "-- {} {} {} {} {} {} {}",
        //     correct_imm, correct_rs1, correct_rs2, correct_rd, correct_rdu, correct_mu, correct_pcu
        // );
        // println!(
        //     "-- {} {} {} {} {} {} {}",
        //     dec_imm, dec_rs1, dec_rs2, dec_rd, dec_rdu, dec_mu, dec_pcu
        // );

        assert_eq!(correct_imm, dec_imm);
        assert_eq!(correct_rs1, dec_rs1);
        assert_eq!(correct_rs2, dec_rs2);
        assert_eq!(correct_rd, dec_rd);
        assert_eq!(correct_rdu, dec_rdu);
        assert_eq!(correct_mu, dec_mu);
        assert_eq!(correct_pcu, dec_pcu);
    }
}
