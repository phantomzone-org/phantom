use poulpy_backend::FFT64Ref;
use poulpy_core::{GGLWEToGGSWKeyEncryptSk, GGSWAutomorphism, GLWEAutomorphismKeyEncryptSk, GLWEDecrypt, GLWEEncryptSk, GLWEExternalProduct, GLWEPackerOps, GLWEPacking, GLWETrace, ScratchTakeCore, layouts::{GGLWEToGGSWKeyPreparedFactory, GGSWPreparedFactory, GLWEAutomorphismKeyPreparedFactory, GLWEInfos, GLWESecret, GLWESecretPrepared, GLWESecretPreparedFactory, LWESecret}};
use poulpy_hal::{api::{ModuleN, ModuleNew, ScratchOwnedAlloc, ScratchOwnedBorrow}, layouts::{Backend, Module, Scratch, ScratchOwned}, source::Source};
use poulpy_schemes::tfhe::{bdd_arithmetic::{BDDKeyEncryptSk, BDDKeyPreparedFactory, FheUint, FheUintPrepare, FheUintPrepared, FheUintPreparedEncryptSk, FheUintPreparedFactory, GGSWBlindRotation}, blind_rotation::{BlindRotationAlgo, BlindRotationKey, BlindRotationKeyFactory, CGGI}};

use crate::{keys::{VMKeys, VMKeysPrepared}, parameters::CryptographicParameters, update_pc};



#[test]
fn test_pc_update_fft64_ref(){
    test_pc_update::<CGGI, FFT64Ref>()
}

fn test_pc_update<BRA: BlindRotationAlgo, BE: Backend>() where
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
        + FheUintPrepare<BRA, u32, BE>
        + GGSWBlindRotation<u32, BE>
        + GGSWPreparedFactory<BE>
        + GLWEDecrypt<BE>
        + GLWEAutomorphismKeyPreparedFactory<BE>
        + GGLWEToGGSWKeyPreparedFactory<BE>
        + BDDKeyPreparedFactory<BRA, BE>
        + GGSWAutomorphism<BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
    BlindRotationKey<Vec<u8>, BRA>: BlindRotationKeyFactory<BRA>,{

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

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BE> =
        GLWESecretPrepared::alloc(module, sk_glwe.rank());
    sk_glwe_prepared.prepare(module, &sk_glwe);

    let ggsw_infos: &poulpy_core::layouts::GGSWLayout = &params.ggsw_infos();
    let glwe_infos: &poulpy_core::layouts::GLWELayout = &params.glwe_ct_infos();

    
    let mut rs1_prep: FheUintPrepared<Vec<u8>, u32, BE> = FheUintPrepared::alloc_from_infos(module, ggsw_infos);
    let mut rs2_prep: FheUintPrepared<Vec<u8>, u32, BE> = FheUintPrepared::alloc_from_infos(module, ggsw_infos);
    let mut imm_prep: FheUintPrepared<Vec<u8>, u32, BE> = FheUintPrepared::alloc_from_infos(module, ggsw_infos);
    let mut pc_prep: FheUintPrepared<Vec<u8>, u32, BE> = FheUintPrepared::alloc_from_infos(module, ggsw_infos);
    let mut pc_id: FheUintPrepared<Vec<u8>, u32, BE> = FheUintPrepared::alloc_from_infos(module, ggsw_infos); 
    let mut pc: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(glwe_infos);

    rs1_prep.encrypt_sk(module, 0, &sk_glwe_prepared, &mut source_xa, &mut source_xe, scratch.borrow());
    rs2_prep.encrypt_sk(module, 0, &sk_glwe_prepared, &mut source_xa, &mut source_xe, scratch.borrow());
    imm_prep.encrypt_sk(module, 0, &sk_glwe_prepared, &mut source_xa, &mut source_xe, scratch.borrow());
    pc_prep.encrypt_sk(module, 0, &sk_glwe_prepared, &mut source_xa, &mut source_xe, scratch.borrow());
    pc_id.encrypt_sk(module, 1, &sk_glwe_prepared, &mut source_xa, &mut source_xe, scratch.borrow());

    let keys: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);
    let mut keys_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    keys_prepared.prepare(module, &keys, scratch.borrow());

    update_pc(module, &mut pc, &rs1_prep, &rs2_prep, &pc_prep, &imm_prep, &pc_id, &keys_prepared, scratch.borrow());

    println!("pc: {}", pc.decrypt(module, &sk_glwe_prepared, scratch.borrow()));
    
}