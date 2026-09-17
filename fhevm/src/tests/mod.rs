mod cycle;
mod interpreters;
mod memory;
mod pc_update;
mod ram_offset;
mod timing;

use poulpy_bin_fhe::blind_rotation::BlindRotationAlgo;
use poulpy_core::layouts::{GLWESecret, GLWESecretPrepared, LWESecret};
use poulpy_hal::{
    layouts::{Module, NoiseInfos, ScratchArena, ScratchOwned},
    source::Source,
};

use crate::{
    backend::{default_noise_infos, prelude::*, PhantomBackend, PhantomBootstrapModule},
    keys::{VMKeys, VMKeysPrepared},
    parameters::CryptographicParameters,
};

/// Parameters, secret keys and prepared evaluation keys shared by the tests.
pub(crate) struct TestContext<BRA: BlindRotationAlgo, BE: PhantomBackend> {
    pub(crate) params: CryptographicParameters<BE>,
    pub(crate) sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BE>,
    pub(crate) keys_prepared: VMKeysPrepared<Vec<u8>, BRA, BE>,
    pub(crate) source_xa: Source,
    pub(crate) source_xe: Source,
    /// Encryption noise of a `FheUint` at the VM's `fhe_uint_infos`.
    pub(crate) fhe_uint_noise: NoiseInfos,
    /// Encryption noise of a `FheUintPrepared` at the VM's
    /// `fhe_uint_prepared_infos` (the GGSW's own precision, not the GLWE's).
    pub(crate) fhe_uint_prepared_noise: NoiseInfos,
}

impl<BRA: BlindRotationAlgo, BE: PhantomBackend> TestContext<BRA, BE>
where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
{
    /// Generates the secret keys and the prepared evaluation keys.
    pub(crate) fn new() -> Self {
        let params: CryptographicParameters<BE> = CryptographicParameters::<BE>::new();
        let mut source_xs: Source = Source::new([0u8; 32]);
        let mut source_xa: Source = Source::new([0u8; 32]);
        let mut source_xe: Source = Source::new([0u8; 32]);

        let (sk_glwe, sk_lwe, sk_glwe_prepared) = {
            let module: &Module<BE> = params.module();

            let mut sk_glwe: GLWESecret<Vec<u8>, i64> = module.glwe_secret_alloc(params.rank());
            module.glwe_secret_fill_ternary_prob(&mut sk_glwe, 0.5, &mut source_xs);

            let mut sk_lwe: LWESecret<Vec<u8>, i64> = module.lwe_secret_alloc(params.n_lwe());
            module.lwe_secret_fill_binary_block(
                &mut sk_lwe,
                params.lwe_block_size(),
                &mut source_xs,
            );

            let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BE> =
                module.glwe_secret_prepared_alloc(params.rank());
            module.glwe_secret_prepare(&mut sk_glwe_prepared, &sk_glwe);
            (sk_glwe, sk_lwe, sk_glwe_prepared)
        };

        let keys: VMKeys<Vec<u8>, BRA> =
            VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

        let keys_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = {
            let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);
            let mut keys_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> =
                VMKeysPrepared::alloc(&params);
            keys_prepared.prepare(params.module(), &keys, &mut scratch.borrow());
            keys_prepared
        };

        let fhe_uint_noise: NoiseInfos = default_noise_infos(&params.fhe_uint_infos());
        let fhe_uint_prepared_noise: NoiseInfos =
            default_noise_infos(&params.fhe_uint_prepared_infos());

        Self {
            params,
            sk_glwe_prepared,
            keys_prepared,
            source_xa,
            source_xe,
            fhe_uint_noise,
            fhe_uint_prepared_noise,
        }
    }

    /// Encrypts `value` into a fresh `FheUint`.
    pub(crate) fn encrypt_fhe_uint(
        &mut self,
        value: u32,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> poulpy_bin_fhe::bdd_arithmetic::FheUint<Vec<u8>, u32, i64> {
        let mut ct = poulpy_bin_fhe::bdd_arithmetic::FheUint::alloc_from_infos(
            self.params.module(),
            &self.params.fhe_uint_infos(),
        );
        ct.encrypt_sk(
            self.params.module(),
            value,
            &self.sk_glwe_prepared,
            &self.fhe_uint_noise,
            &mut self.source_xe,
            &mut self.source_xa,
            scratch,
        );
        ct
    }

    /// Encrypts `value` into a fresh `FheUintPrepared`.
    pub(crate) fn encrypt_fhe_uint_prepared(
        &mut self,
        value: u32,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> poulpy_bin_fhe::bdd_arithmetic::FheUintPrepared<Vec<u8>, u32, BE> {
        let mut ct = poulpy_bin_fhe::bdd_arithmetic::FheUintPrepared::alloc_from_infos(
            self.params.module(),
            &self.params.fhe_uint_prepared_infos(),
        );
        ct.encrypt_sk(
            self.params.module(),
            value,
            &self.sk_glwe_prepared,
            &self.fhe_uint_prepared_noise,
            &mut self.source_xe,
            &mut self.source_xa,
            scratch,
        );
        ct
    }
}
