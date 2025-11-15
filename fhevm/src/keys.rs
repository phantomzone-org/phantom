use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{Backend, Data, DataMut, DataRef, Module, Scratch, ScratchOwned},
    source::Source,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{
        BDDKey, BDDKeyEncryptSk, BDDKeyHelper, BDDKeyInfos, BDDKeyLayout, BDDKeyPrepared,
        BDDKeyPreparedFactory,
    },
    blind_rotation::{BlindRotationAlgo, BlindRotationKey, BlindRotationKeyFactory},
    circuit_bootstrapping::{
        CircuitBootstrappingKeyLayout, CircuitBootstrappingKeyPrepared,
        CircuitBootstrappingKeyPreparedFactory,
    },
};
use std::collections::HashMap;

use poulpy_core::{
    layouts::{
        GGLWELayout, GGLWEToGGSWKeyPreparedFactory, GLWEAutomorphismKey, GLWEAutomorphismKeyHelper,
        GLWEAutomorphismKeyPrepared, GLWEAutomorphismKeyPreparedFactory, GLWEInfos,
        GLWESecretToRef, GLWEToLWEKeyLayout, GLWEToLWEKeyPrepared, LWEInfos, LWESecretToRef, GLWE,
    },
    GGLWEToGGSWKeyEncryptSk, GLWEAutomorphismKeyEncryptSk, GLWETrace, GetDistribution,
    ScratchTakeCore,
};

use crate::parameters::CryptographicParameters;

/// Struct storing the FHE evaluation keys for the read/write on FHE-RAM.
pub struct VMKeys<D: Data, BRA: BlindRotationAlgo> {
    evk_ram: HashMap<i64, GLWEAutomorphismKey<D>>,
    bdd_key: BDDKey<D, BRA>,
}

pub struct VMKeysPrepared<D: Data, BRA: BlindRotationAlgo, B: Backend> {
    pub(crate) atk_glwe: HashMap<i64, GLWEAutomorphismKeyPrepared<D, B>>,
    pub(crate) bdd_key: BDDKeyPrepared<D, BRA, B>,
}

impl<D: DataRef, BRA: BlindRotationAlgo, BE: Backend> BDDKeyInfos for VMKeysPrepared<D, BRA, BE> {
    fn cbt_infos(&self) -> CircuitBootstrappingKeyLayout {
        self.bdd_key.cbt_infos()
    }

    fn ks_infos(&self) -> GLWEToLWEKeyLayout {
        self.bdd_key.ks_infos()
    }
}

impl<D: DataRef, BRA: BlindRotationAlgo, BE: Backend>
    GLWEAutomorphismKeyHelper<GLWEAutomorphismKeyPrepared<D, BE>, BE>
    for VMKeysPrepared<D, BRA, BE>
{
    fn automorphism_key_infos(&self) -> GGLWELayout {
        self.atk_glwe.automorphism_key_infos()
    }

    fn get_automorphism_key(&self, k: i64) -> Option<&GLWEAutomorphismKeyPrepared<D, BE>> {
        self.atk_glwe.get_automorphism_key(k)
    }
}

impl<BRA: BlindRotationAlgo, B: Backend> VMKeysPrepared<Vec<u8>, BRA, B> {
    pub fn alloc(params: &CryptographicParameters<B>) -> Self
    where
        Module<B>: GLWETrace<B>
            + GLWEAutomorphismKeyPreparedFactory<B>
            + CircuitBootstrappingKeyPreparedFactory<BRA, B>,
    {
        let module: &Module<B> = params.module();
        let gal_els: Vec<i64> = GLWE::trace_galois_elements(module);
        let evk_ram_infos: &GGLWELayout = &params.evk_ram_infos();

        // let cbt_infos: &CircuitBootstrappingKeyLayout = &params.cbt_key_layout();
        // let glwe_to_lwe_infos: &GLWEToLWEKeyLayout = &params.glwe_to_lwe_key_layout();
        let bdd_infos: &BDDKeyLayout = &params.bdd_key_layout();

        Self {
            atk_glwe: HashMap::from_iter(gal_els.iter().map(|gal_el| {
                (
                    *gal_el,
                    GLWEAutomorphismKeyPrepared::alloc_from_infos(module, evk_ram_infos),
                )
            })),
            bdd_key: BDDKeyPrepared::alloc_from_infos(module, bdd_infos),
        }
    }
}

impl<D: DataMut, BRA: BlindRotationAlgo, B: Backend> VMKeysPrepared<D, BRA, B> {
    pub fn prepare<O, M>(&mut self, module: &M, other: &VMKeys<O, BRA>, scratch: &mut Scratch<B>)
    where
        O: DataRef,
        M: GLWEAutomorphismKeyPreparedFactory<B>
            + GGLWEToGGSWKeyPreparedFactory<B>
            + BDDKeyPreparedFactory<BRA, B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        for (k, key) in self.atk_glwe.iter_mut() {
            let other: &GLWEAutomorphismKey<O> = other.evk_ram.get(k).unwrap();
            key.prepare(module, other, scratch);
        }
        self.bdd_key.prepare(module, &other.bdd_key, scratch);
    }
}

impl<BRA: BlindRotationAlgo> VMKeys<Vec<u8>, BRA> {
    /// Constructor for EvaluationKeys
    pub fn new(
        evk_ram: HashMap<i64, GLWEAutomorphismKey<Vec<u8>>>,
        bdd_key: BDDKey<Vec<u8>, BRA>,
    ) -> Self {
        Self { evk_ram, bdd_key }
    }

    /// Getter for auto_keys at glwe level
    pub fn evk_ram(&self) -> &HashMap<i64, GLWEAutomorphismKey<Vec<u8>>> {
        &self.evk_ram
    }

    /// Mutable getter for auto_keys at glwe level
    pub fn evk_ram_mut(&mut self) -> &mut HashMap<i64, GLWEAutomorphismKey<Vec<u8>>> {
        &mut self.evk_ram
    }

    /// Setter for auto_keys at glwe level
    pub fn set_evk_ram(&mut self, evk_ram: HashMap<i64, GLWEAutomorphismKey<Vec<u8>>>) {
        self.evk_ram = evk_ram;
    }
}

impl<BRA: BlindRotationAlgo> VMKeys<Vec<u8>, BRA> {
    pub fn encrypt_sk<SL, SG, BE: Backend>(
        params: &CryptographicParameters<BE>,
        sk_lwe: &SL,
        sk_glwe: &SG,
        source_xa: &mut Source,
        source_xe: &mut Source,
    ) -> VMKeys<Vec<u8>, BRA>
    where
        SL: LWESecretToRef + GetDistribution + LWEInfos,
        SG: GLWESecretToRef + GetDistribution + GLWEInfos,
        Module<BE>: GLWEAutomorphismKeyEncryptSk<BE>
            + GGLWEToGGSWKeyEncryptSk<BE>
            + GLWETrace<BE>
            + BDDKeyEncryptSk<BRA, BE>,
        BlindRotationKey<Vec<u8>, BRA>: BlindRotationKeyFactory<BRA>,
        ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let module: &Module<BE> = params.module();

        let evk_ram_infos: GGLWELayout = params.evk_ram_infos();

        let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(
            GLWEAutomorphismKey::encrypt_sk_tmp_bytes(module, &evk_ram_infos).max(module.bdd_key_encrypt_sk_tmp_bytes(&params.bdd_key_layout())),
        );

        let gal_els: Vec<i64> = GLWE::trace_galois_elements(module);
        let evk_ram: HashMap<i64, GLWEAutomorphismKey<Vec<u8>>> =
            HashMap::from_iter(gal_els.iter().map(|gal_el| {
                let mut key: GLWEAutomorphismKey<Vec<u8>> =
                    GLWEAutomorphismKey::alloc_from_infos(&evk_ram_infos);
                key.encrypt_sk(
                    module,
                    *gal_el,
                    sk_glwe,
                    source_xa,
                    source_xe,
                    scratch.borrow(),
                );
                (*gal_el, key)
            }));

        let mut bdd_key: BDDKey<Vec<u8>, BRA> = BDDKey::alloc_from_infos(&params.bdd_key_layout());
        bdd_key.encrypt_sk(
            module,
            sk_lwe,
            sk_glwe,
            source_xa,
            source_xe,
            scratch.borrow(),
        );

        VMKeys { evk_ram, bdd_key }
    }
}

impl<D: DataRef, BRA: BlindRotationAlgo, BE: Backend> BDDKeyHelper<D, BRA, BE>
    for VMKeysPrepared<D, BRA, BE>
{
    fn get_cbt_key(
        &self,
    ) -> (
        &CircuitBootstrappingKeyPrepared<D, BRA, BE>,
        &GLWEToLWEKeyPrepared<D, BE>,
    ) {
        self.bdd_key.get_cbt_key()
    }
}
