//! Backend and module bounds shared by the whole VM.
//!
//! `poulpy` 0.8 keys every operation on the backend `BE` and on the
//! `Module<BE>` that implements it. Rather than repeating the full list of
//! operation traits on every function, the VM names the set once here:
//!
//! - [`PhantomBackend`]: a host-resident backend storing `i64` coefficients in
//!   `Vec<u8>` buffers (every CPU backend of `poulpy`).
//! - [`PhantomModule`]: every backend-independent operation the VM uses.
//! - [`PhantomBootstrapModule`]: the operations that additionally depend on the
//!   blind-rotation algorithm `BRA` (circuit bootstrapping, BDD key material).
//!
//! It also provides [`for_each_with_scratch`], the parallel loop used by the
//! VM's own multi-threaded paths.

use poulpy_bin_fhe::bdd_arithmetic::{BitSize, GetBitCircuitInfo, GetGGSWBit, Node};
use poulpy_bin_fhe::{
    bdd_arithmetic::{
        BDDKeyEncryptSk, BDDKeyPreparedFactory, Cmux, Cswap, ExecuteBDDCircuit,
        ExecuteBDDCircuit1WTo1W, ExecuteBDDCircuit2WTo1W, FheUintPrepare, FheUintPreparedEncryptSk,
        FheUintPreparedFactory, GGSWBlindRotation, GLWEBlindRetrieval, GLWEBlindRotation,
        GLWEBlindSelection,
    },
    blind_rotation::BlindRotationAlgo,
    circuit_bootstrapping::CircuitBootstrappingExecute,
};
use poulpy_core::{
    layouts::{
        GGLWEToGGSWKeyPreparedFactory, GGSWPreparedFactory, GLWEAutomorphismKeyPreparedFactory,
        GLWEInfos, GLWESecretPreparedFactory, GLWESecretSampling, GLWEToBackendMut,
        LWESecretSampling, ModuleCoreAlloc,
    },
    GGLWEToGGSWKeyEncryptSk, GGSWEncryptSk, GLWEAdd, GLWEAutomorphismKeyEncryptSk, GLWEBytesOf,
    GLWECopy, GLWEDecrypt, GLWEEncryptSk, GLWEExternalProduct, GLWEExternalProductInternal,
    GLWEKeyswitch, GLWENoise, GLWENormalize, GLWEPacking, GLWERotate, GLWESub, GLWETrace, GLWEZero,
    LWEFromGLWE, DEFAULT_BOUND_XE, DEFAULT_SIGMA_XE,
};
use poulpy_hal::{
    api::{ModuleLogN, ModuleN},
    execution::{worker_count, worker_scratch_bytes, TaskExecutor},
    layouts::{Backend, HostBackend, Module, NoiseInfos, ScratchArena},
};

/// A host-resident backend with `i64` coefficients stored in `Vec<u8>`.
pub trait PhantomBackend:
    for<'a> Backend<
        OwnedBuf = Vec<u8>,
        ZnxWord = i64,
        BufRef<'a> = &'a [u8],
        BufMut<'a> = &'a mut [u8],
    > + HostBackend
    + Sync
    + 'static
{
}

impl<BE> PhantomBackend for BE where
    BE: for<'a> Backend<
            OwnedBuf = Vec<u8>,
            ZnxWord = i64,
            BufRef<'a> = &'a [u8],
            BufMut<'a> = &'a mut [u8],
        > + HostBackend
        + Sync
        + 'static
{
}

/// Every backend-independent operation used by the VM.
pub trait PhantomModule<BE: PhantomBackend>:
    Sized
    + Sync
    + ModuleN
    + ModuleLogN
    + ModuleCoreAlloc<OwnedBuf = Vec<u8>, ZnxWord = i64>
    + GLWEBytesOf<BE>
    + GLWESecretSampling<BE>
    + LWESecretSampling<BE>
    + GLWESecretPreparedFactory<BE>
    + GLWEEncryptSk<BE>
    + GLWEDecrypt<BE>
    + GLWENoise<BE>
    + GLWEAdd<BE>
    + GLWESub<BE>
    + GLWECopy<BE>
    + GLWEZero<BE>
    + GLWERotate<BE>
    + GLWENormalize<BE>
    + GLWETrace<BE>
    + GLWEPacking<BE>
    + GLWEKeyswitch<BE>
    + GLWEExternalProduct<BE>
    + GLWEExternalProductInternal<BE>
    + LWEFromGLWE<BE>
    + GGSWEncryptSk<BE>
    + GGSWPreparedFactory<BE>
    + GLWEAutomorphismKeyEncryptSk<BE>
    + GLWEAutomorphismKeyPreparedFactory<BE>
    + GGLWEToGGSWKeyEncryptSk<BE>
    + GGLWEToGGSWKeyPreparedFactory<BE>
    + Cmux<BE>
    + Cswap<BE>
    + GLWEBlindRotation<BE>
    + GGSWBlindRotation<u32, BE>
    + GLWEBlindRetrieval<BE>
    + GLWEBlindSelection<u32, BE>
    + ExecuteBDDCircuit<BE>
    + ExecuteBDDCircuit1WTo1W<BE>
    + ExecuteBDDCircuit2WTo1W<BE>
    + FheUintPreparedFactory<u32, BE>
    + FheUintPreparedEncryptSk<u32, BE>
{
}

impl<BE, M> PhantomModule<BE> for M
where
    BE: PhantomBackend,
    M: Sized
        + Sync
        + ModuleN
        + ModuleLogN
        + ModuleCoreAlloc<OwnedBuf = Vec<u8>, ZnxWord = i64>
        + GLWEBytesOf<BE>
        + GLWESecretSampling<BE>
        + LWESecretSampling<BE>
        + GLWESecretPreparedFactory<BE>
        + GLWEEncryptSk<BE>
        + GLWEDecrypt<BE>
        + GLWENoise<BE>
        + GLWEAdd<BE>
        + GLWESub<BE>
        + GLWECopy<BE>
        + GLWEZero<BE>
        + GLWERotate<BE>
        + GLWENormalize<BE>
        + GLWETrace<BE>
        + GLWEPacking<BE>
        + GLWEKeyswitch<BE>
        + GLWEExternalProduct<BE>
        + GLWEExternalProductInternal<BE>
        + LWEFromGLWE<BE>
        + GGSWEncryptSk<BE>
        + GGSWPreparedFactory<BE>
        + GLWEAutomorphismKeyEncryptSk<BE>
        + GLWEAutomorphismKeyPreparedFactory<BE>
        + GGLWEToGGSWKeyEncryptSk<BE>
        + GGLWEToGGSWKeyPreparedFactory<BE>
        + Cmux<BE>
        + Cswap<BE>
        + GLWEBlindRotation<BE>
        + GGSWBlindRotation<u32, BE>
        + GLWEBlindRetrieval<BE>
        + GLWEBlindSelection<u32, BE>
        + ExecuteBDDCircuit<BE>
        + ExecuteBDDCircuit1WTo1W<BE>
        + ExecuteBDDCircuit2WTo1W<BE>
        + FheUintPreparedFactory<u32, BE>
        + FheUintPreparedEncryptSk<u32, BE>,
{
}

/// [`PhantomModule`] plus the operations that depend on the blind-rotation
/// algorithm `BRA`.
pub trait PhantomBootstrapModule<BRA: BlindRotationAlgo, BE: PhantomBackend>:
    PhantomModule<BE>
    + FheUintPrepare<BRA, BE>
    + BDDKeyEncryptSk<BRA, BE>
    + BDDKeyPreparedFactory<BRA, BE>
    + CircuitBootstrappingExecute<BRA, BE>
{
}

impl<BRA, BE, M> PhantomBootstrapModule<BRA, BE> for M
where
    BRA: BlindRotationAlgo,
    BE: PhantomBackend,
    M: PhantomModule<BE>
        + FheUintPrepare<BRA, BE>
        + BDDKeyEncryptSk<BRA, BE>
        + BDDKeyPreparedFactory<BRA, BE>
        + CircuitBootstrappingExecute<BRA, BE>,
{
}

/// Default Gaussian encryption noise for a ciphertext described by `infos`.
pub(crate) fn default_noise_infos<A: GLWEInfos>(infos: &A) -> NoiseInfos {
    NoiseInfos::new(infos.k().as_usize(), DEFAULT_SIGMA_XE, DEFAULT_BOUND_XE)
        .expect("default encryption noise parameters are valid")
}

/// Per-worker scratch size, aligned for the backend, that a parallel loop must
/// reserve for each of its workers.
pub(crate) fn worker_bytes<BE: Backend>(bytes: usize) -> usize {
    worker_scratch_bytes::<BE>(bytes)
}

/// Applies `task` to every item of `items`, at most `threads` at a time, with
/// a dedicated scratch slice of `per_worker` bytes per worker.
///
/// On a backend whose task executor is parallel (the `*Rayon` backends) the
/// loop is scheduled by that executor, so nested library parallelism is
/// coordinated by the backend. On a serial backend the loop falls back to
/// `std::thread::scope`, so the VM stays multi-threaded on every CPU backend.
///
/// `task` receives the absolute index of the item (`base_index + offset`).
pub(crate) fn for_each_with_scratch<BE, T, F>(
    threads: usize,
    items: &mut [T],
    base_index: usize,
    per_worker: usize,
    scratch: &mut ScratchArena<'_, BE>,
    task: &F,
) where
    BE: PhantomBackend,
    T: Send,
    F: Fn(usize, &mut T, &mut ScratchArena<'_, BE>) + Send + Sync,
{
    if items.is_empty() {
        return;
    }
    let threads: usize = threads.max(1);
    let workers: usize = if BE::TaskExecutor::IS_PARALLEL {
        worker_count::<BE::TaskExecutor>(threads, items.len())
    } else {
        threads.min(items.len())
    };
    assert!(
        scratch.available() >= workers * per_worker,
        "scratch.available(): {} < workers: {workers} * per_worker: {per_worker}",
        scratch.available()
    );
    let (arenas, _) = scratch.borrow().split(workers, per_worker);

    if BE::TaskExecutor::IS_PARALLEL {
        poulpy_hal::execution::for_each_with_scratch::<BE::TaskExecutor, BE, _, _>(
            items, base_index, arenas, task,
        );
        return;
    }

    let chunk_size: usize = items.len().div_ceil(workers);
    std::thread::scope(|scope| {
        for (chunk_index, (chunk, mut arena)) in
            items.chunks_mut(chunk_size).zip(arenas).enumerate()
        {
            let start: usize = base_index + chunk_index * chunk_size;
            scope.spawn(move || {
                for (offset, item) in chunk.iter_mut().enumerate() {
                    task(start + offset, item, &mut arena);
                }
            });
        }
    });
}

/// One output bit of a multi-output BDD circuit, presented as a circuit of its
/// own so that a single worker can evaluate it.
struct BitCircuitWindow<'a, C: GetBitCircuitInfo> {
    circuit: &'a C,
    bit: usize,
}

impl<'a, C: GetBitCircuitInfo> GetBitCircuitInfo for BitCircuitWindow<'a, C> {
    fn input_size(&self) -> usize {
        self.circuit.input_size()
    }

    fn output_size(&self) -> usize {
        1
    }

    fn get_circuit(&self, bit: usize) -> (&[Node], usize) {
        debug_assert_eq!(bit, 0);
        self.circuit.get_circuit(self.bit)
    }
}

/// Evaluates `circuit` on `inputs` over at most `threads` workers.
///
/// On a backend with a parallel task executor (the `*Rayon` backends) this is
/// exactly poulpy's `execute_bdd_circuit_multi_thread`. On the serial CPU
/// backends that call runs on one thread, so the output bits are spread over
/// `std::thread::scope` workers instead, each evaluating one bit through a
/// [`BitCircuitWindow`] (see [`for_each_with_scratch`]).
pub(crate) fn execute_bdd_circuit_parallel<BE, G, C, O>(
    threads: usize,
    module: &Module<BE>,
    out: &mut [O],
    inputs: &G,
    circuit: &C,
    scratch: &mut ScratchArena<'_, BE>,
) where
    BE: PhantomBackend,
    Module<BE>: PhantomModule<BE>,
    G: GetGGSWBit<BE> + BitSize,
    C: GetBitCircuitInfo,
    O: GLWEToBackendMut<BE> + GLWEInfos + Send,
{
    if BE::TaskExecutor::IS_PARALLEL {
        module.execute_bdd_circuit_multi_thread(threads, out, inputs, circuit, scratch);
        return;
    }
    let output_size: usize = circuit.output_size();
    assert!(out.len() >= output_size);
    for out_i in out.iter_mut().skip(output_size) {
        module.glwe_zero(out_i);
    }
    if output_size == 0 {
        return;
    }
    let per_worker: usize = worker_bytes::<BE>(module.execute_bdd_circuit_tmp_bytes(
        &out[0],
        circuit.max_state_size(),
        inputs.get_bit(0),
    ));
    for_each_with_scratch(
        threads,
        &mut out[..output_size],
        0,
        per_worker,
        scratch,
        &|bit, out_i, scratch| {
            let window = BitCircuitWindow { circuit, bit };
            module.execute_bdd_circuit(
                std::slice::from_mut(out_i),
                inputs,
                &window,
                &mut scratch.borrow(),
            );
        },
    );
}

/// Every trait whose methods the VM calls on a module, a ciphertext or a key.
///
/// Supertraits of [`PhantomModule`] do not bring their methods into scope, so
/// modules import this prelude.
pub mod prelude {
    pub use poulpy_bin_fhe::{
        bdd_arithmetic::{
            BDDKeyEncryptSk, BDDKeyHelper, BDDKeyInfos, BDDKeyPreparedFactory, Cmux, Cswap,
            ExecuteBDDCircuit, ExecuteBDDCircuit1WTo1W, ExecuteBDDCircuit2WTo1W, FheUintPrepare,
            FheUintPreparedEncryptSk, FheUintPreparedFactory, GGSWBlindRotation,
            GLWEBlindRetrieval, GLWEBlindRotation, GLWEBlindSelection, GetGGSWBit, GetGGSWBitMut,
        },
        blind_rotation::BlindRotationKeyInfos,
        circuit_bootstrapping::{CircuitBootstrappingExecute, CircuitBootstrappingKeyInfos},
    };
    pub use poulpy_core::{
        layouts::{
            prepared::{
                GGLWEPreparedToBackendRef, GGSWPreparedToBackendMut, GGSWPreparedToBackendRef,
            },
            GGLWEInfos, GGLWEToGGSWKeyPreparedFactory, GGSWInfos, GGSWPreparedFactory,
            GGSWToBackendMut, GGSWToBackendRef, GLWEAutomorphismKeyPreparedFactory, GLWEInfos,
            GLWESecretPreparedFactory, GLWESecretSampling, GLWEToBackendMut, GLWEToBackendRef,
            GetAutomorphismKey, GetGaloisElement, LWEInfos, LWESecretSampling, ModuleCoreAlloc,
            SetGaloisElement,
        },
        GGLWEToGGSWKeyEncryptSk, GGSWEncryptSk, GLWEAdd, GLWEAutomorphismKeyEncryptSk, GLWEBytesOf,
        GLWECopy, GLWEDecrypt, GLWEEncryptSk, GLWEExternalProduct, GLWEKeyswitch, GLWENoise,
        GLWENormalize, GLWEPacking, GLWERotate, GLWESub, GLWETrace, GLWEZero, GetDistribution,
        LWEFromGLWE, ScratchArenaTakeCore,
    };
    pub use poulpy_hal::api::{
        ModuleLogN, ModuleN, ModuleNew, ScratchOwnedAlloc, ScratchOwnedBorrow,
    };
}
