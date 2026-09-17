use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fhevm::{
    backend::{prelude::*, PhantomBackend, PhantomBootstrapModule},
    instructions::Instruction,
    instructions::InstructionsParser,
    interpreter::Interpreter,
    keys::{VMKeys, VMKeysPrepared},
    parameters::CryptographicParameters,
};
use std::env;

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "fma"
))]
use poulpy_cpu_avx::{FFT64Avx, FFT64AvxRayon};
use poulpy_cpu_ref::FFT64Ref;

use poulpy_bin_fhe::blind_rotation::{BlindRotationAlgo, CGGI};
use poulpy_core::layouts::{GLWESecret, GLWESecretPrepared, LWESecret};
use poulpy_hal::{
    layouts::{Module, ScratchOwned},
    source::Source,
};
use std::hint::black_box;

fn benc_cycle_with_backend(c: &mut Criterion) {
    // Parse thread count from environment variable
    // Usage: PHANTOM_THREADS=64 cargo bench --package fhevm --bench cycle
    let threads = env::var("PHANTOM_THREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(32);

    println!("Using {} threads", threads);

    // PHANTOM_BACKEND selects the backend: fft64-ref, fft64-avx (default when
    // the build has AVX2/FMA) or fft64-avx-rayon.
    let has_avx = cfg!(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "fma"
    ));
    let backend = env::var("PHANTOM_BACKEND")
        .map(|s| s.trim().to_ascii_lowercase().replace('_', "-"))
        .unwrap_or_else(|_| {
            if has_avx {
                "fft64-avx".into()
            } else {
                "fft64-ref".into()
            }
        });
    println!("Running benchmark with {backend} backend");

    match backend.as_str() {
        "fft64-ref" => benc_cycle::<CGGI, FFT64Ref>(c, "fft64_ref", threads),
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "fma"
        ))]
        "fft64-avx" => benc_cycle::<CGGI, FFT64Avx>(c, "fft64_avx", threads),
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "fma"
        ))]
        "fft64-avx-rayon" => {
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global();
            benc_cycle::<CGGI, FFT64AvxRayon>(c, "fft64_avx_rayon", threads)
        }
        other => panic!("PHANTOM_BACKEND={other}: unknown or unavailable in this build"),
    }
}

pub fn benc_cycle<BRA: BlindRotationAlgo, BE: PhantomBackend>(
    c: &mut Criterion,
    label: &str,
    threads: usize,
) where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
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
    let mut sk_glwe: GLWESecret<Vec<u8>, i64> = module.glwe_secret_alloc(params.rank());
    module.glwe_secret_fill_ternary_prob(&mut sk_glwe, 0.5, &mut source_xs);
    let mut sk_lwe: LWESecret<Vec<u8>, i64> = module.lwe_secret_alloc(params.n_lwe());
    module.lwe_secret_fill_binary_block(&mut sk_lwe, params.lwe_block_size(), &mut source_xs);

    let mut interpreter: Interpreter<BE> =
        Interpreter::new_with_debug(&params, rom.len(), ram.len());

    let mut instructions = InstructionsParser::new();
    for inst in &rom {
        instructions.add(*inst);
    }

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BE> =
        module.glwe_secret_prepared_alloc(params.rank());
    module.glwe_secret_prepare(&mut sk_glwe_prepared, &sk_glwe);

    interpreter.instructions_encrypt_sk(
        module,
        &instructions,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        &mut scratch.borrow(),
    );

    interpreter.ram_encrypt_sk(
        module,
        &ram,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        &mut scratch.borrow(),
    );

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 28);

    let key: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, &mut scratch.borrow());

    interpreter.set_verbose_timings(false);
    interpreter.set_threads(threads);

    let mut runner = move || {
        interpreter.cycle(module, &key_prepared, &mut scratch.borrow());
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
