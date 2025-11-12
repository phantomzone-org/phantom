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
        GGLWELayout, GGLWEToGGSWKey, GGLWEToGGSWKeyPrepared, GGLWEToGGSWKeyPreparedFactory,
        GLWEAutomorphismKey, GLWEAutomorphismKeyHelper, GLWEAutomorphismKeyPrepared,
        GLWEAutomorphismKeyPreparedFactory, GLWEInfos, GLWESecretToRef, GLWEToLWEKeyLayout,
        GLWEToLWEKeyPrepared, LWEInfos, LWESecretToRef, GLWE,
    },
    GGLWEToGGSWKeyEncryptSk, GLWEAutomorphismKeyEncryptSk, GLWETrace, GetDistribution,
    ScratchTakeCore,
};

use crate::parameters::CryptographicParameters;

/// Struct storing the FHE evaluation keys for the read/write on FHE-RAM.
pub struct VMKeys<D: Data, BRA: BlindRotationAlgo> {
    atk_glwe: HashMap<i64, GLWEAutomorphismKey<D>>,
    atk_ggsw_inv: GLWEAutomorphismKey<D>,
    gglwe_to_ggsw_key: GGLWEToGGSWKey<D>,

    bdd_key: BDDKey<D, BRA>,
}

pub struct VMKeysPrepared<D: Data, BRA: BlindRotationAlgo, B: Backend> {
    pub(crate) atk_glwe: HashMap<i64, GLWEAutomorphismKeyPrepared<D, B>>,
    pub(crate) atk_ggsw_inv: GLWEAutomorphismKeyPrepared<D, B>,
    pub(crate) tsk_ggsw_inv: GGLWEToGGSWKeyPrepared<D, B>,

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
        let evk_glwe_infos: &GGLWELayout = &params.evk_glwe_infos();
        let evk_ggsw_infos: &GGLWELayout = &params.evk_ggsw_infos();

        // let cbt_infos: &CircuitBootstrappingKeyLayout = &params.cbt_key_layout();
        // let glwe_to_lwe_infos: &GLWEToLWEKeyLayout = &params.glwe_to_lwe_key_layout();
        let bdd_infos: &BDDKeyLayout = &params.bdd_key_layout();

        Self {
            atk_glwe: HashMap::from_iter(gal_els.iter().map(|gal_el| {
                (
                    *gal_el,
                    GLWEAutomorphismKeyPrepared::alloc_from_infos(module, evk_glwe_infos),
                )
            })),
            atk_ggsw_inv: GLWEAutomorphismKeyPrepared::alloc_from_infos(module, evk_ggsw_infos),
            tsk_ggsw_inv: GGLWEToGGSWKeyPrepared::alloc_from_infos(module, evk_ggsw_infos),

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
            let other: &GLWEAutomorphismKey<O> = other.atk_glwe.get(k).unwrap();
            key.prepare(module, other, scratch);
        }
        self.atk_ggsw_inv
            .prepare(module, &other.atk_ggsw_inv, scratch);
        self.tsk_ggsw_inv
            .prepare(module, &other.gglwe_to_ggsw_key, scratch);

        self.bdd_key.prepare(module, &other.bdd_key, scratch);
    }
}

impl<BRA: BlindRotationAlgo> VMKeys<Vec<u8>, BRA> {
    /// Constructor for EvaluationKeys
    pub fn new(
        atk_glwe: HashMap<i64, GLWEAutomorphismKey<Vec<u8>>>,
        atk_ggsw_inv: GLWEAutomorphismKey<Vec<u8>>,
        gglwe_to_ggsw_key: GGLWEToGGSWKey<Vec<u8>>,

        bdd_key: BDDKey<Vec<u8>, BRA>,
    ) -> Self {
        Self {
            atk_glwe,
            atk_ggsw_inv,
            gglwe_to_ggsw_key,

            bdd_key,
        }
    }

    /// Getter for auto_keys at glwe level
    pub fn atk_glwe(&self) -> &HashMap<i64, GLWEAutomorphismKey<Vec<u8>>> {
        &self.atk_glwe
    }

    /// Mutable getter for auto_keys at glwe level
    pub fn atk_glwe_mut(&mut self) -> &mut HashMap<i64, GLWEAutomorphismKey<Vec<u8>>> {
        &mut self.atk_glwe
    }

    /// Setter for auto_keys at glwe level
    pub fn set_atk_glwe(&mut self, atk_glwe: HashMap<i64, GLWEAutomorphismKey<Vec<u8>>>) {
        self.atk_glwe = atk_glwe;
    }

    /// Getter for tensor_key at ggsw level
    pub fn tsk_ggsw_inv(&self) -> &GGLWEToGGSWKey<Vec<u8>> {
        &self.gglwe_to_ggsw_key
    }

    /// Mutable getter for tensor_key at ggsw level
    pub fn tsk_ggsw_inv_mut(&mut self) -> &mut GGLWEToGGSWKey<Vec<u8>> {
        &mut self.gglwe_to_ggsw_key
    }

    /// Setter for tensor_key at ggsw level
    pub fn set_tsk_ggsw_inv(&mut self, key: GGLWEToGGSWKey<Vec<u8>>) {
        self.gglwe_to_ggsw_key = key;
    }

    /// Getter for auto_key(-1) at ggsw level
    pub fn atk_ggsw_inv(&self) -> &GLWEAutomorphismKey<Vec<u8>> {
        &self.atk_ggsw_inv
    }

    /// Mutable getter for auto_key(-1) at ggsw level
    pub fn atk_ggsw_inv_mut(&mut self) -> &mut GLWEAutomorphismKey<Vec<u8>> {
        &mut self.atk_ggsw_inv
    }

    /// Setter for auto_key(-1) at ggsw level
    pub fn set_atk_ggsw_inv(&mut self, atk_ggsw_inv: GLWEAutomorphismKey<Vec<u8>>) {
        self.atk_ggsw_inv = atk_ggsw_inv;
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

        let evk_glwe_infos: GGLWELayout = params.evk_glwe_infos();
        let evk_ggsw_infos: GGLWELayout = params.evk_ggsw_infos();

        let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(
            GLWEAutomorphismKey::encrypt_sk_tmp_bytes(module, &evk_glwe_infos)
                | GLWEAutomorphismKey::encrypt_sk_tmp_bytes(module, &evk_ggsw_infos)
                | GGLWEToGGSWKey::encrypt_sk_tmp_bytes(module, &evk_ggsw_infos),
        );

        let gal_els: Vec<i64> = GLWE::trace_galois_elements(module);
        let atk_glwe: HashMap<i64, GLWEAutomorphismKey<Vec<u8>>> =
            HashMap::from_iter(gal_els.iter().map(|gal_el| {
                let mut key: GLWEAutomorphismKey<Vec<u8>> =
                    GLWEAutomorphismKey::alloc_from_infos(&evk_glwe_infos);
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

        let mut gglwe_to_ggsw_key: GGLWEToGGSWKey<Vec<u8>> =
            GGLWEToGGSWKey::alloc_from_infos(&evk_ggsw_infos);
        gglwe_to_ggsw_key.encrypt_sk(module, sk_glwe, source_xa, source_xe, scratch.borrow());

        let mut atk_ggsw_inv: GLWEAutomorphismKey<Vec<u8>> =
            GLWEAutomorphismKey::alloc_from_infos(&evk_ggsw_infos);
        atk_ggsw_inv.encrypt_sk(module, -1, sk_glwe, source_xa, source_xe, scratch.borrow());

        let mut bdd_key = BDDKey::alloc_from_infos(&params.bdd_key_layout());
        bdd_key.encrypt_sk(
            module,
            sk_lwe,
            sk_glwe,
            source_xa,
            source_xe,
            scratch.borrow(),
        );

        VMKeys {
            atk_glwe,
            atk_ggsw_inv,
            gglwe_to_ggsw_key,
            bdd_key,
        }
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
