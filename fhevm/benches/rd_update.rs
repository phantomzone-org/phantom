use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fhevm::{
    instructions::RD_UPDATE_RV32I_OP_LIST,
    interpreter::Interpreter,
    keys::{VMKeys, VMKeysPrepared},
    parameters::{CryptographicParameters, DECOMP_N},
    rd_update::Evaluate,
};
use std::hint::black_box;

use poulpy_backend::FFT64Avx;
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

fn benc_rd_update_fft64_avx(c: &mut Criterion) {
    benc_rd_update::<CGGI, FFT64Avx>(c, "fft64_ref");
}

pub fn benc_rd_update<BRA: BlindRotationAlgo, BE: Backend>(c: &mut Criterion, label: &str)
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
        + FheUintPrepare<BRA, u32, BE>
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
    let group_name: String = format!("rd_update::{label}");

    let mut group = c.benchmark_group(group_name);

    let params: CryptographicParameters<BE> = CryptographicParameters::<BE>::new();

    let module: &Module<BE> = params.module();

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    let rom_size = 1 << 10;
    let ram_size = 1 << 10;
    let mut interpreter: Interpreter<BE> =
        Interpreter::new(&params, rom_size, ram_size, DECOMP_N.into());

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

    let key: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    let mut runner = move || {
        interpreter.update_registers(
            module,
            &RD_UPDATE_RV32I_OP_LIST,
            &key_prepared,
            None::<&GLWESecretPrepared<Vec<u8>, BE>>,
            scratch.borrow(),
        );
        black_box(());
    };

    let id: BenchmarkId = BenchmarkId::from_parameter(format!("n_glwe: {} n_lwe: {}", params.n_glwe(), params.n_lwe()));
    group.bench_with_input(id, &(), |b, _| b.iter(&mut runner));
}

criterion_group!(benches, benc_rd_update_fft64_avx,);

criterion_main!(benches);
