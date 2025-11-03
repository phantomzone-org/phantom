use fhevm::{Instruction, InstructionsParser, Interpreter, parameters::CryptographicParameters};
use poulpy_backend::FFT64Ref;
use poulpy_core::layouts::{GLWESecret, GLWESecretPrepared, LWESecret};
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::ScratchOwned,
    source::Source,
};

#[test]
pub fn test_interpreter_init_pc() {
    // use poulpy_schemes::tfhe::bdd_arithmetic::{FheUintPreparedDebug, FheUintBlockDebugPrepare};

    let params = CryptographicParameters::<FFT64Ref>::new();

    let seed_xs: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);

    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.module().n().into());
    sk_lwe.fill_binary_block(8, &mut source_xs);

    let mut interpreter = Interpreter::new(&sk_lwe, &sk_glwe);

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc_from_infos(params.module(), &params.glwe_ct_infos());
    sk_glwe_prepared.prepare(params.module(), &sk_glwe);
    interpreter.init_pc(&sk_glwe_prepared);

    // TODO: decrypt the prepared program counter
    // fheuint_debug can only be initialized from
}

#[test]
pub fn test_interpreter_init_one_instruction() {
    let params = CryptographicParameters::<FFT64Ref>::new();

    let seed_xs: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    // sk_glwe.fill_zero();

    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.module().n().into());
    sk_lwe.fill_binary_block(8, &mut source_xs);
    // sk_lwe.fill_zero();

    let mut interpreter = Interpreter::new(&sk_lwe, &sk_glwe);

    let instruction_u32 = 258455;
    let mut parser = InstructionsParser::new();
    parser.add(Instruction::new(instruction_u32));

    interpreter.init_instructions(&sk_glwe, &parser);
    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc_from_infos(params.module(), &params.glwe_ct_infos());
    sk_glwe_prepared.prepare(params.module(), &sk_glwe);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 22);

    let instruction = parser.get_raw(0);
    let correct_imm = instruction.get_immediate();
    let correct_rs1 = instruction.get_rs1() as u32;
    let correct_rs2 = instruction.get_rs2() as u32;
    let correct_rd = instruction.get_rd() as u32;

    interpreter.init_pc(&sk_glwe_prepared);
    let (imm_fheuint, rs1_fheuint, rs2_fheuint, rd_fheuint) =
        interpreter.read_instruction_components();

    let dec_rs1 = rs1_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    let dec_rs2 = rs2_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    let dec_rd = rd_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    let dec_imm = imm_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());

    println!(
        "{} {} {} {}",
        correct_imm, correct_rs1, correct_rs2, correct_rd
    );
    println!("{} {} {} {}", dec_imm, dec_rs1, dec_rs2, dec_rd);

    assert_eq!(correct_imm, dec_imm);
    assert_eq!(correct_rs1, dec_rs1);
    assert_eq!(correct_rs2, dec_rs2);
    assert_eq!(correct_rd, dec_rd);
}

#[test]
pub fn test_interpreter_init_many_instructions() {
    use crate::Instruction;

    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();

    let seed_xs: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);

    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.module().n().into());
    sk_lwe.fill_binary_block(8, &mut source_xs);

    let mut interpreter = Interpreter::new(&sk_lwe, &sk_glwe);

    let instructions_u32 = vec![
        258455, 33653139, 512279, 4286644499, 66579, 10507363, 3221229683, 8388847, 3221229683,
        791, 8585319, 259383,
    ];

    let mut parser = InstructionsParser::new();
    for inst in instructions_u32.clone() {
        parser.add(Instruction::new(inst));
    }

    interpreter.init_instructions(&sk_glwe, &parser);
    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc_from_infos(params.module(), &params.glwe_ct_infos());
    sk_glwe_prepared.prepare(params.module(), &sk_glwe);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 22);

    for idx in 0..instructions_u32.len() {
        let instruction = parser.get_raw(idx);
        let correct_imm = instruction.get_immediate();
        let correct_rs1 = instruction.get_rs1() as u32;
        let correct_rs2 = instruction.get_rs2() as u32;
        let correct_rd = instruction.get_rd() as u32;

        interpreter.set_pc_to(idx as u32, &sk_glwe_prepared);
        let (imm_fheuint, rs1_fheuint, rs2_fheuint, rd_fheuint) =
            interpreter.read_instruction_components();

        let dec_rs1 = rs1_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
        let dec_rs2 = rs2_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
        let dec_rd = rd_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
        let dec_imm = imm_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());

        assert_eq!(correct_imm, dec_imm);
        assert_eq!(correct_rs1, dec_rs1);
        assert_eq!(correct_rs2, dec_rs2);
        assert_eq!(correct_rd, dec_rd);
    }
}
