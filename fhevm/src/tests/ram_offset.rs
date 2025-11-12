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
        BDDKeyEncryptSk, BDDKeyPreparedFactory, FheUint, FheUintPrepare, FheUintPrepared,
        FheUintPreparedEncryptSk, FheUintPreparedFactory, GGSWBlindRotation,
    },
    blind_rotation::{BlindRotationAlgo, BlindRotationKey, BlindRotationKeyFactory, CGGI},
};
use rand_core::RngCore;

use crate::{
    keys::{VMKeys, VMKeysPrepared},
    parameters::CryptographicParameters,
    ram_offset::ram_offset,
};

#[test]
fn test_ram_offset_fft64_ref() {
    test_ram_offset::<CGGI, FFT64Ref>()
}

fn test_ram_offset<BRA: BlindRotationAlgo, BE: Backend>()
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

    let mut rs1_prep: FheUintPrepared<Vec<u8>, u32, BE> =
        FheUintPrepared::alloc_from_infos(module, ggsw_infos);
    let mut imm_prep: FheUintPrepared<Vec<u8>, u32, BE> =
        FheUintPrepared::alloc_from_infos(module, ggsw_infos);
    let mut ram: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(glwe_infos);

    let rs1: u32 = source_xa.next_u32();
    let imm: u32 = source_xa.next_u32();
    let offset: u32 = 1 << 18;

    rs1_prep.encrypt_sk(
        module,
        rs1,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );
    imm_prep.encrypt_sk(
        module,
        imm,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let keys: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);
    let mut keys_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    keys_prepared.prepare(module, &keys, scratch.borrow());

    ram_offset(
        module,
        &mut ram,
        &rs1_prep,
        &imm_prep,
        &keys_prepared,
        scratch.borrow(),
    );

    //println!("ram: {}", ram.decrypt(module, &sk_glwe_prepared, scratch.borrow()));
    assert_eq!(
        rs1.wrapping_add(imm).wrapping_sub(offset),
        ram.decrypt(module, &sk_glwe_prepared, scratch.borrow())
    )
}
