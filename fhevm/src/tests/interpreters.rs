use crate::{
    keys::{VMKeys, VMKeysPrepared},
    parameters::CryptographicParameters,
    rd_update::Evaluate,
    Instruction, InstructionsParser, Interpreter, RD_UPDATE_RV32I_OP_LIST,
};
use poulpy_backend::FFT64Ref;
use poulpy_core::{
    layouts::{
        GGLWEToGGSWKeyPreparedFactory, GGSWPreparedFactory, GLWEAutomorphismKeyPreparedFactory,
        GLWEInfos, GLWESecret, GLWESecretPrepared, GLWESecretPreparedFactory, LWESecret,
    },
    GGLWEToGGSWKeyEncryptSk, GGSWAutomorphism, GLWEAutomorphismKeyEncryptSk, GLWEDecrypt,
    GLWEEncryptSk, GLWEExternalProduct, GLWEPackerOps, GLWEPacking, GLWETrace, ScratchTakeCore,
};
use poulpy_hal::{
    api::{ModuleN, ModuleNew, ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{Backend, Module, Scratch, ScratchOwned},
    source::Source,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{
        BDDKeyEncryptSk, BDDKeyPreparedFactory, FheUint, FheUintPrepare, FheUintPreparedEncryptSk,
        FheUintPreparedFactory, GGSWBlindRotation,
    },
    blind_rotation::{BlindRotationAlgo, BlindRotationKey, BlindRotationKeyFactory, CGGI},
};
use rand_core::RngCore;

#[test]
fn test_interpreter_init_one_instruction_fft64_ref() {
    test_interpreter_init_one_instruction::<CGGI, FFT64Ref>()
}

pub fn test_interpreter_init_one_instruction<BRA: BlindRotationAlgo, BE: Backend>()
where
    Module<BE>: ModuleNew<BE>
        + GLWESecretPreparedFactory<BE>
        + FheUintPreparedFactory<u32, BE>
        + ModuleN
        + GLWEEncryptSk<BE>
        + FheUintPreparedEncryptSk<u32, BE>
        + GLWEAutomorphismKeyEncryptSk<BE>
        + GGLWEToGGSWKeyEncryptSk<BE>
        + GLWETrace<BE>
        + BDDKeyEncryptSk<BRA, BE>
        + GGSWPreparedFactory<BE>
        + GLWEExternalProduct<BE>
        + GLWEPackerOps<BE>
        + GLWEPacking<BE>
        + FheUintPrepare<BRA, BE>
        + GGSWBlindRotation<u32, BE>
        + GGSWPreparedFactory<BE>
        + GLWEDecrypt<BE>
        + GLWEAutomorphismKeyPreparedFactory<BE>
        + GGLWEToGGSWKeyPreparedFactory<BE>
        + BDDKeyPreparedFactory<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
    BlindRotationKey<Vec<u8>, BRA>: BlindRotationKeyFactory<BRA>,
{
    let params: CryptographicParameters<BE> = CryptographicParameters::<BE>::new();

    let module: &Module<BE> = params.module();

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 22);

    let rom_size = 1 << 10;
    let ram_size = 1 << 10;
    let mut interpreter: Interpreter<BE> = Interpreter::new(&params, rom_size, ram_size);

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([0u8; 32]);
    let mut source_xe: Source = Source::new([0u8; 32]);

    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BE> =
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
    let correct_imm: u32 = instruction.get_imm();
    let (rs2, rs1, rd) = instruction.get_registers();
    let correct_rs1: u32 = rs1 as u32;
    let correct_rs2: u32 = rs2 as u32;
    let correct_rd: u32 = rd as u32;

    let (rdu, mu, pcu) = instruction.get_opid();
    let correct_rdu: u32 = rdu as u32;
    let correct_mu: u32 = mu as u32;
    let correct_pcu: u32 = pcu as u32;

    interpreter.pc_fhe_uint.encrypt_sk(
        module,
        idx as u32,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let key: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    let mut this_cycle_measurement = crate::PerCycleMeasurements::new();
    interpreter.read_instruction_components(
        1,
        module,
        &key_prepared,
        Some(&sk_glwe_prepared),
        scratch.borrow(),
        &mut this_cycle_measurement,
    );

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
fn test_interpreter_init_one_op_fft64_ref() {
    test_interpreter_init_one_op::<CGGI, FFT64Ref>()
}

pub fn test_interpreter_init_one_op<BRA: BlindRotationAlgo, BE: Backend>()
where
    Module<BE>: ModuleNew<BE>
        + GLWESecretPreparedFactory<BE>
        + FheUintPreparedFactory<u32, BE>
        + ModuleN
        + GLWEEncryptSk<BE>
        + FheUintPreparedEncryptSk<u32, BE>
        + GLWEAutomorphismKeyEncryptSk<BE>
        + GGLWEToGGSWKeyEncryptSk<BE>
        + GLWETrace<BE>
        + BDDKeyEncryptSk<BRA, BE>
        + GGSWPreparedFactory<BE>
        + GLWEExternalProduct<BE>
        + GLWEPackerOps<BE>
        + GLWEPacking<BE>
        + FheUintPrepare<BRA, BE>
        + GGSWBlindRotation<u32, BE>
        + GGSWPreparedFactory<BE>
        + GLWEDecrypt<BE>
        + GLWEAutomorphismKeyPreparedFactory<BE>
        + GGLWEToGGSWKeyPreparedFactory<BE>
        + BDDKeyPreparedFactory<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
    BlindRotationKey<Vec<u8>, BRA>: BlindRotationKeyFactory<BRA>,
{
    let params: CryptographicParameters<BE> = CryptographicParameters::<BE>::new();

    let threads = 4;

    let module: &Module<BE> = params.module();

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 22);

    let rom_size = 1 << 10;
    let ram_size = 1 << 10;
    let mut interpreter: Interpreter<BE> = Interpreter::new(&params, rom_size, ram_size);

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([0u8; 32]);
    let mut source_xe: Source = Source::new([0u8; 32]);

    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BE> =
        GLWESecretPrepared::alloc(module, sk_glwe.rank());
    sk_glwe_prepared.prepare(module, &sk_glwe);

    let imm_value: u32 = source_xa.next_u32();
    let rs1_value: u32 = source_xa.next_u32();
    let rs2_value: u32 = source_xa.next_u32();
    let pc_value: u32 = source_xa.next_u32();
    let ram_value: u32 = source_xa.next_u32();

    let key: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    let mut this_cycle_measurement = crate::PerCycleMeasurements::new();
    interpreter.read_instruction_components(
        1,
        module,
        &key_prepared,
        Some(&sk_glwe_prepared),
        scratch.borrow(),
        &mut this_cycle_measurement,
    );

    interpreter.rs1_val_fhe_uint_prepared.encrypt_sk(
        module,
        rs1_value,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );
    interpreter.rs2_val_fhe_uint_prepared.encrypt_sk(
        module,
        rs2_value,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );
    interpreter.imm_val_fhe_uint_prepared.encrypt_sk(
        module,
        imm_value,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );
    interpreter.pc_fhe_uint_prepared.encrypt_sk(
        module,
        pc_value,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    interpreter.ram_val_fhe_uint.encrypt_sk(
        module,
        ram_value,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let mut res = FheUint::alloc_from_infos(&params.glwe_ct_infos());

    for op in RD_UPDATE_RV32I_OP_LIST {
        op.eval_enc(
            threads,
            module,
            &mut res,
            &interpreter.rs1_val_fhe_uint_prepared,
            &interpreter.rs2_val_fhe_uint_prepared,
            &interpreter.imm_val_fhe_uint_prepared,
            &interpreter.pc_fhe_uint_prepared,
            &interpreter.ram_val_fhe_uint,
            &key_prepared,
            scratch.borrow(),
        );

        let have: u32 = res.decrypt(module, &sk_glwe_prepared, scratch.borrow());
        let want: u32 = op.eval_plain(imm_value, rs1_value, rs2_value, pc_value, ram_value);

        assert_eq!(
            have, want,
            "{:#?}\n
            rs1: {rs1_value}\n
            rs2: {rs2_value}\n
            imm: {imm_value}\n
            pc:  {pc_value}\n
            ram: {ram_value}\n
            -> have: {} != want: {}",
            op, have, want
        );
    }
}

#[test]
fn test_interpreter_init_many_instructions_fft64_ref() {
    test_interpreter_init_many_instructions::<CGGI, FFT64Ref>()
}

fn test_interpreter_init_many_instructions<BRA: BlindRotationAlgo, BE: Backend>()
where
    Module<BE>: ModuleNew<BE>
        + GLWESecretPreparedFactory<BE>
        + FheUintPreparedFactory<u32, BE>
        + ModuleN
        + GLWEEncryptSk<BE>
        + FheUintPreparedEncryptSk<u32, BE>
        + GLWEAutomorphismKeyEncryptSk<BE>
        + GGLWEToGGSWKeyEncryptSk<BE>
        + GLWETrace<BE>
        + BDDKeyEncryptSk<BRA, BE>
        + GGSWPreparedFactory<BE>
        + GLWEExternalProduct<BE>
        + GLWEPackerOps<BE>
        + GLWEPacking<BE>
        + FheUintPrepare<BRA, BE>
        + GGSWBlindRotation<u32, BE>
        + GGSWPreparedFactory<BE>
        + GLWEDecrypt<BE>
        + GLWEAutomorphismKeyPreparedFactory<BE>
        + GGLWEToGGSWKeyPreparedFactory<BE>
        + BDDKeyPreparedFactory<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
    BlindRotationKey<Vec<u8>, BRA>: BlindRotationKeyFactory<BRA>,
{
    use crate::Instruction;

    let params: CryptographicParameters<BE> = CryptographicParameters::<BE>::new();
    let module: &Module<BE> = params.module();

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([0u8; 32]);
    let mut source_xe: Source = Source::new([0u8; 32]);

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

    let rom_size = 1 << 10;
    let ram_size = 1 << 10;
    let mut interpreter: Interpreter<BE> = Interpreter::new(&params, rom_size, ram_size);

    let instructions_u32 = vec![
        258455, 33653139, 512279, 4286644499, 66579, 10507363, 3221229683, 8388847, 3221229683,
        791, 8585319, 259383,
    ];

    let mut instructions = InstructionsParser::new();
    for inst in instructions_u32.clone() {
        instructions.add(Instruction::new(inst));
    }

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BE> =
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

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    let key: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    for idx in 0..instructions_u32.len() {
        let instruction = instructions.get_raw(idx);
        let correct_imm = instruction.get_imm();
        let (rs2, rs1, rd) = instruction.get_registers();
        let correct_rs1: u32 = rs1 as u32;
        let correct_rs2: u32 = rs2 as u32;
        let correct_rd: u32 = rd as u32;
        let (rdu, mu, pcu) = instruction.get_opid();
        let correct_rdu: u32 = rdu as u32;
        let correct_mu: u32 = mu as u32;
        let correct_pcu: u32 = pcu as u32;

        interpreter.pc_fhe_uint.encrypt_sk(
            module,
            (idx << 2) as u32,
            &sk_glwe_prepared,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );
        let mut this_cycle_measurement = crate::PerCycleMeasurements::new();
        interpreter.read_instruction_components(
            1,
            module,
            &key_prepared,
            Some(&sk_glwe_prepared),
            scratch.borrow(),
            &mut this_cycle_measurement,
        );

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

#[test]
fn test_interpreter_cycle_single_instruction_noop_fft64_ref() {
    test_interpreter_cycle_single_instruction_noop::<CGGI, FFT64Ref>();
}

fn test_interpreter_cycle_single_instruction_noop<BRA: BlindRotationAlgo, BE: Backend>()
where
    Module<BE>: ModuleNew<BE>
        + GLWESecretPreparedFactory<BE>
        + FheUintPreparedFactory<u32, BE>
        + ModuleN
        + GLWEEncryptSk<BE>
        + FheUintPreparedEncryptSk<u32, BE>
        + GLWEAutomorphismKeyEncryptSk<BE>
        + GGLWEToGGSWKeyEncryptSk<BE>
        + GLWETrace<BE>
        + BDDKeyEncryptSk<BRA, BE>
        + GGSWPreparedFactory<BE>
        + GLWEExternalProduct<BE>
        + GLWEPackerOps<BE>
        + GLWEPacking<BE>
        + FheUintPrepare<BRA, BE>
        + GGSWBlindRotation<u32, BE>
        + GGSWPreparedFactory<BE>
        + GLWEDecrypt<BE>
        + GLWEAutomorphismKeyPreparedFactory<BE>
        + GGLWEToGGSWKeyPreparedFactory<BE>
        + BDDKeyPreparedFactory<BRA, BE>
        + GGSWAutomorphism<BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
    BlindRotationKey<Vec<u8>, BRA>: BlindRotationKeyFactory<BRA>,
{
    use crate::Instruction;

    let params: CryptographicParameters<BE> = CryptographicParameters::<BE>::new();
    let module: &Module<BE> = params.module();

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([0u8; 32]);
    let mut source_xe: Source = Source::new([0u8; 32]);

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

    let rom_size = 1 << 10;
    let ram_size = 1 << 10;
    let mut interpreter: Interpreter<BE> = Interpreter::new(&params, rom_size, ram_size);

    let instructions_u32 = vec![
        // 258455
        0b00000000_00000000_00000000_1110011,
        0b00000000_00000000_00000000_1110011,
        // 258455, 33653139, 512279, 4286644499, 66579, 10507363, 3221229683, 8388847, 3221229683,
        // 791, 8585319, 259383,
    ];

    let mut instructions = InstructionsParser::new();
    for inst in instructions_u32.clone() {
        instructions.add(Instruction::new(inst));
    }

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BE> =
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

    interpreter.pc_fhe_uint.encrypt_sk(
        module,
        0,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    let key: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    println!("Cycle");
    interpreter.cycle(1, module, &key_prepared, scratch.borrow());
    println!("Cycle done");

    let pc = interpreter
        .pc_fhe_uint
        .decrypt(module, &sk_glwe_prepared, scratch.borrow());
    println!("PC: {}", pc);
    assert_eq!(pc, 4);
}
