use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fhevm::{
    instructions::Instruction,
    instructions::InstructionsParser,
    interpreter::Interpreter,
    keys::{VMKeys, VMKeysPrepared},
    parameters::CryptographicParameters,
    prepare::PrepareMultiple,
};

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "fma"
))]
use poulpy_cpu_avx::FFT64Avx as BackendImpl;
#[cfg(not(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "fma"
)))]
use poulpy_cpu_ref::FFT64Ref as BackendImpl;

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
use poulpy_schemes::bin_fhe::{
    bdd_arithmetic::{
        BDDKeyEncryptSk, BDDKeyPreparedFactory, FheUintPrepare, FheUintPreparedEncryptSk,
        FheUintPreparedFactory, GGSWBlindRotation, GLWEBlindRetrieval,
    },
    blind_rotation::{BlindRotationAlgo, BlindRotationKey, BlindRotationKeyFactory, CGGI},
};
use std::hint::black_box;

fn benc_cycle_with_backend(c: &mut Criterion) {
    if cfg!(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "fma"
    )) {
        println!("Running benchmark with FFT64Avx backend");
        benc_cycle::<CGGI, BackendImpl>(c, "fft64_avx");
    } else {
        println!("Running benchmark with FFT64Ref backend");
        benc_cycle::<CGGI, BackendImpl>(c, "fft64_ref");
    }
}

pub fn benc_cycle<BRA: BlindRotationAlgo, BE: Backend>(c: &mut Criterion, label: &str)
where
    Module<BE>: ModuleNew<BE>
        + GLWESecretPreparedFactory<BE>
        + FheUintPreparedFactory<u32, BE>
        + ModuleN
        + GLWEEncryptSk<BE>
        + FheUintPreparedEncryptSk<u32, BE>
        + PrepareMultiple<BE, BRA>
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
        + GLWEBlindRetrieval<BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
    BlindRotationKey<Vec<u8>, BRA>: BlindRotationKeyFactory<BRA>,
{
    let group_name: String = format!("cycle::{label}");

    let mut group = c.benchmark_group(group_name);
    let mut rom: Vec<Instruction> = vec![];
    for _ in 0..1024 {
        rom.push(Instruction::new(0b00000000_00000000_00000000_1110011));
    }

    let ram: Vec<u32> = vec![0u32; 64];

    let params: CryptographicParameters<BE> = CryptographicParameters::<BE>::new();
    let module: &Module<BE> = params.module();

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([0u8; 32]);
    let mut source_xe: Source = Source::new([0u8; 32]);

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc(params.n_glwe(), params.rank());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

    let mut interpreter: Interpreter<BE> =
        Interpreter::new_with_debug(&params, rom.len(), ram.len());

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

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 28);

    let key: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    interpreter.set_verbose_timings(true);
    interpreter.set_threads(32);

    let mut runner = move || {
        interpreter.cycle(module, &key_prepared, scratch.borrow());
        black_box(());
    };

    let id: BenchmarkId = BenchmarkId::from_parameter(format!(
        "n_glwe: {} n_lwe: {}",
        params.n_glwe(),
        params.n_lwe()
    ));
    group.bench_with_input(id, &(), |b, _| b.iter(&mut runner));
}

criterion_group!(benches, benc_cycle_with_backend);

criterion_main!(benches);
