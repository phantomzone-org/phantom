use std::collections::HashMap;

use poulpy_bin_fhe::{
    bdd_arithmetic::{
        BDDEncryptionInfos, BDDKey, BDDKeyHelper, BDDKeyInfos, BDDKeyLayout, BDDKeyPrepared,
    },
    blind_rotation::BlindRotationAlgo,
    circuit_bootstrapping::{CircuitBootstrappingKeyLayout, CircuitBootstrappingKeyPrepared},
};
use poulpy_core::{
    layouts::{
        prepared::GLWEAutomorphismKeyPreparedBackendRef, GGLWELayout, GLWEAutomorphismKey,
        GLWEAutomorphismKeyPrepared, GLWEInfos, GLWESecretToBackendRef, GLWESwitchingKeyLayout,
        GLWESwitchingKeyPrepared, GLWEToLWEKeyLayout, GLWEToLWEKeyPrepared, GetAutomorphismKey,
        LWEInfos, LWESecretToBackendRef, SetGaloisElement, TorusPrecision,
    },
    GetDistribution, DEFAULT_BOUND_XE, DEFAULT_SIGMA_XE,
};
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{Data, Module, NoiseInfos, ScratchArena, ScratchOwned},
    source::Source,
};

#[allow(unused_imports)]
use crate::backend::prelude::*;
use crate::{
    backend::{PhantomBackend, PhantomBootstrapModule},
    parameters::CryptographicParameters,
};

/// Struct storing the FHE evaluation keys for the read/write on FHE-RAM.
pub struct VMKeys<D: Data, BRA: BlindRotationAlgo> {
    evk_ram: HashMap<i64, GLWEAutomorphismKey<D, i64>>,
    bdd_key: BDDKey<D, BRA, i64>,
}

pub struct VMKeysPrepared<D: Data, BRA: BlindRotationAlgo, B: PhantomBackend> {
    pub(crate) atk_glwe: HashMap<i64, GLWEAutomorphismKeyPrepared<D, B>>,
    pub(crate) bdd_key: BDDKeyPrepared<D, BRA, B>,
}

impl<D: Data, BRA: BlindRotationAlgo, BE: PhantomBackend> BDDKeyInfos
    for VMKeysPrepared<D, BRA, BE>
{
    fn cbt_infos(&self) -> CircuitBootstrappingKeyLayout {
        self.bdd_key.cbt_infos()
    }

    fn ks_glwe_infos(&self) -> Option<GLWESwitchingKeyLayout> {
        self.bdd_key.ks_glwe_infos()
    }

    fn ks_lwe_infos(&self) -> GLWEToLWEKeyLayout {
        self.bdd_key.ks_lwe_infos()
    }
}

/// The RAM automorphism keys answer every trace/packing lookup of the VM.
impl<BRA: BlindRotationAlgo, BE: PhantomBackend> GetAutomorphismKey<BE>
    for VMKeysPrepared<Vec<u8>, BRA, BE>
{
    fn lookup_automorphism_key(
        &self,
        p: i64,
        k: TorusPrecision,
    ) -> poulpy_core::Result<GLWEAutomorphismKeyPreparedBackendRef<'_, BE>> {
        self.atk_glwe.lookup_automorphism_key(p, k)
    }
}

impl<BRA: BlindRotationAlgo, BE: PhantomBackend> BDDKeyHelper<Vec<u8>, BRA, BE>
    for VMKeysPrepared<Vec<u8>, BRA, BE>
{
    fn get_cbt_key(
        &self,
    ) -> (
        &CircuitBootstrappingKeyPrepared<Vec<u8>, BRA, BE>,
        Option<&GLWESwitchingKeyPrepared<Vec<u8>, BE>>,
        &GLWEToLWEKeyPrepared<Vec<u8>, BE>,
    ) {
        self.bdd_key.get_cbt_key()
    }
}

impl<BRA: BlindRotationAlgo, BE: PhantomBackend> VMKeysPrepared<Vec<u8>, BRA, BE> {
    pub fn alloc(params: &CryptographicParameters<BE>) -> Self
    where
        Module<BE>: PhantomBootstrapModule<BRA, BE>,
    {
        let module: &Module<BE> = params.module();
        let gal_els: Vec<i64> = module.glwe_trace_galois_elements();
        let evk_ram_infos: &GGLWELayout = &params.evk_ram_infos();
        let bdd_infos: &BDDKeyLayout = &params.bdd_key_layout();

        Self {
            atk_glwe: HashMap::from_iter(gal_els.iter().map(|gal_el| {
                let mut key: GLWEAutomorphismKeyPrepared<Vec<u8>, BE> =
                    module.glwe_automorphism_key_prepared_alloc_from_infos(evk_ram_infos);
                key.set_p(*gal_el);
                (*gal_el, key)
            })),
            bdd_key: BDDKeyPrepared::alloc_from_infos(module, bdd_infos),
        }
    }

    pub fn prepare(
        &mut self,
        module: &Module<BE>,
        other: &VMKeys<Vec<u8>, BRA>,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomBootstrapModule<BRA, BE>,
    {
        for (k, key) in self.atk_glwe.iter_mut() {
            let other: &GLWEAutomorphismKey<Vec<u8>, i64> = other.evk_ram.get(k).unwrap();
            module.glwe_automorphism_key_prepare(key, other, scratch);
        }
        self.bdd_key.prepare(module, &other.bdd_key, scratch);
    }
}

impl<BRA: BlindRotationAlgo> VMKeys<Vec<u8>, BRA> {
    /// Constructor for EvaluationKeys
    pub fn new(
        evk_ram: HashMap<i64, GLWEAutomorphismKey<Vec<u8>, i64>>,
        bdd_key: BDDKey<Vec<u8>, BRA, i64>,
    ) -> Self {
        Self { evk_ram, bdd_key }
    }

    /// Getter for auto_keys at glwe level
    pub fn evk_ram(&self) -> &HashMap<i64, GLWEAutomorphismKey<Vec<u8>, i64>> {
        &self.evk_ram
    }

    /// Mutable getter for auto_keys at glwe level
    pub fn evk_ram_mut(&mut self) -> &mut HashMap<i64, GLWEAutomorphismKey<Vec<u8>, i64>> {
        &mut self.evk_ram
    }

    /// Setter for auto_keys at glwe level
    pub fn set_evk_ram(&mut self, evk_ram: HashMap<i64, GLWEAutomorphismKey<Vec<u8>, i64>>) {
        self.evk_ram = evk_ram;
    }
}

impl<BRA: BlindRotationAlgo> VMKeys<Vec<u8>, BRA> {
    pub fn encrypt_sk<SL, SG, BE: PhantomBackend>(
        params: &CryptographicParameters<BE>,
        sk_lwe: &SL,
        sk_glwe: &SG,
        source_xa: &mut Source,
        source_xe: &mut Source,
    ) -> VMKeys<Vec<u8>, BRA>
    where
        SL: LWESecretToBackendRef<BE> + GetDistribution + LWEInfos,
        SG: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos,
        Module<BE>: PhantomBootstrapModule<BRA, BE>,
        ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
    {
        let module: &Module<BE> = params.module();

        let evk_ram_infos: GGLWELayout = params.evk_ram_infos();
        let bdd_layout: BDDKeyLayout = params.bdd_key_layout();

        let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(
            module
                .glwe_automorphism_key_encrypt_sk_tmp_bytes(&evk_ram_infos)
                .max(module.bdd_key_encrypt_sk_tmp_bytes(&bdd_layout)),
        );

        let evk_ram_noise: NoiseInfos = NoiseInfos::new(
            evk_ram_infos.k().as_usize(),
            DEFAULT_SIGMA_XE,
            DEFAULT_BOUND_XE,
        )
        .expect("default encryption noise parameters are valid");

        let gal_els: Vec<i64> = module.glwe_trace_galois_elements();
        let evk_ram: HashMap<i64, GLWEAutomorphismKey<Vec<u8>, i64>> =
            HashMap::from_iter(gal_els.iter().map(|gal_el| {
                let mut key: GLWEAutomorphismKey<Vec<u8>, i64> =
                    module.glwe_automorphism_key_alloc_from_infos(&evk_ram_infos);
                module.glwe_automorphism_key_encrypt_sk(
                    &mut key,
                    *gal_el,
                    sk_glwe,
                    &evk_ram_noise,
                    source_xe,
                    source_xa,
                    &mut scratch.borrow(),
                );
                (*gal_el, key)
            }));

        let bdd_noise: BDDEncryptionInfos = BDDEncryptionInfos::from_default_sigma(&bdd_layout)
            .expect("default BDD key noise parameters are valid");

        let mut bdd_key: BDDKey<Vec<u8>, BRA, i64> = BDDKey::alloc_from_infos(module, &bdd_layout);
        module.bdd_key_encrypt_sk(
            &mut bdd_key,
            sk_lwe,
            sk_glwe,
            &bdd_noise,
            source_xe,
            source_xa,
            &mut scratch.borrow(),
        );

        VMKeys { evk_ram, bdd_key }
    }
}
