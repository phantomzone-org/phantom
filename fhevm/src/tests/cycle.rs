use crate::{
    keys::{VMKeys, VMKeysPrepared},
    parameters::{CryptographicParameters, DECOMP_N},
    Instruction, InstructionsParser, Interpreter, RV32I,
};
use poulpy_backend::FFT64Ref;
use poulpy_core::{
    layouts::{
        GGLWEToGGSWKeyPreparedFactory, GGSWPreparedFactory, GLWEAutomorphismKeyPreparedFactory,
        GLWEInfos, GLWESecret, GLWESecretPrepared, GLWESecretPreparedFactory, LWESecret,
    },
    GGLWEToGGSWKeyEncryptSk, GLWEAutomorphismKeyEncryptSk, GLWEDecrypt, GLWEEncryptSk,
    GLWEExternalProduct, GLWEPackerOps, GLWEPacking, GLWETrace, ScratchTakeCore,
};
use poulpy_hal::{
    api::{ModuleN, ModuleNew, ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{Backend, Module, Scratch, ScratchOwned},
    source::Source,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{
        BDDKeyEncryptSk, BDDKeyPreparedFactory, FheUintPrepare, FheUintPreparedEncryptSk,
        FheUintPreparedFactory, GGSWBlindRotation,
    },
    blind_rotation::{BlindRotationAlgo, BlindRotationKey, BlindRotationKeyFactory, CGGI},
};

#[test]
fn test_interpreter_cycles_fft64_ref() {
    test_interpreter_cycles::<CGGI, FFT64Ref>()
}

fn test_interpreter_cycles<BRA: BlindRotationAlgo, BE: Backend>()
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
    let rom: Vec<Instruction> = vec![
        // RD[31] <- 1<<18
        RV32I::LUI.new().set_imm(1 << 6).set_rd(31),
        // RD[1] <- 0xABCD<<12
        RV32I::LUI.new().set_imm(0xABCD).set_rd(1),
        // RD[2] <- 0xEF10<<12
        RV32I::LUI.new().set_imm(0xEF10).set_rd(2),
        // RAM[RD[31] - 1<<18] <- RD[1] + 1<<12
        RV32I::ADDI.new().set_imm(0x1).set_rs1(1).set_rd(3),
        RV32I::SW.new().set_imm(0).set_rs1(31).set_rs2(3),
        RV32I::ADDI.new().set_imm(4).set_rs1(31).set_rd(31),
        // RAM[RD[31] - 1<<18] <- RD[1] < 0xEF10<<12
        RV32I::SLTI.new().set_imm(0xEF10).set_rs1(1).set_rd(3),
        RV32I::SW.new().set_imm(0).set_rs1(31).set_rs2(3),
        RV32I::ADDI.new().set_imm(4).set_rs1(31).set_rd(31),
    ];

    let ram: Vec<u32> = vec![0u32; 64];

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

    let mut interpreter: Interpreter<BE> =
        Interpreter::new_with_debug(&params, rom.len(), ram.len(), DECOMP_N.into());

    let mut instructions = InstructionsParser::new();
    for inst in &rom {
        instructions.add(*inst);
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

    interpreter.ram_encrypt_sk(
        module,
        &ram,
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

    for _ in 0..rom.len() {
        interpreter.cycle_debug(1, module, &key_prepared, &sk_glwe_prepared, scratch.borrow());
    }
}
