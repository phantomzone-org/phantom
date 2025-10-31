use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{Backend, Data, DataMut, DataRef, Module, Scratch, ScratchOwned},
    source::Source,
};
use std::collections::HashMap;

use poulpy_core::{
    GGLWEToGGSWKeyEncryptSk, GLWEAutomorphismKeyEncryptSk, GLWETrace, GetDistribution,
    ScratchTakeCore,
    layouts::{
        GGLWELayout, GGLWEToGGSWKey, GGLWEToGGSWKeyPrepared, GGLWEToGGSWKeyPreparedFactory, GLWE,
        GLWEAutomorphismKey, GLWEAutomorphismKeyPrepared, GLWEAutomorphismKeyPreparedFactory,
        GLWEInfos, GLWESecretToRef,
    },
};

use crate::Parameters;

/// Struct storing the FHE evaluation keys for the read/write on FHE-RAM.
pub struct EvaluationKeys<D: Data> {
    atk_glwe: HashMap<i64, GLWEAutomorphismKey<D>>,
    atk_ggsw_inv: GLWEAutomorphismKey<D>,
    gglwe_to_ggsw_key: GGLWEToGGSWKey<D>,
}

pub struct EvaluationKeysPrepared<D: Data, B: Backend> {
    pub(crate) atk_glwe: HashMap<i64, GLWEAutomorphismKeyPrepared<D, B>>,
    pub(crate) atk_ggsw_inv: GLWEAutomorphismKeyPrepared<D, B>,
    pub(crate) tsk_ggsw_inv: GGLWEToGGSWKeyPrepared<D, B>,
}

impl<B: Backend> EvaluationKeysPrepared<Vec<u8>, B> {
    pub fn alloc(params: &Parameters<B>) -> Self
    where
        Module<B>: GLWETrace<B> + GLWEAutomorphismKeyPreparedFactory<B>,
    {
        let module: &Module<B> = params.module();
        let gal_els: Vec<i64> = GLWE::trace_galois_elements(module);
        let evk_glwe_infos: &GGLWELayout = &params.evk_glwe_infos();
        let evk_ggsw_infos: &GGLWELayout = &params.evk_ggsw_infos();

        Self {
            atk_glwe: HashMap::from_iter(gal_els.iter().map(|gal_el| {
                (
                    *gal_el,
                    GLWEAutomorphismKeyPrepared::alloc_from_infos(module, evk_glwe_infos),
                )
            })),
            atk_ggsw_inv: GLWEAutomorphismKeyPrepared::alloc_from_infos(module, evk_ggsw_infos),
            tsk_ggsw_inv: GGLWEToGGSWKeyPrepared::alloc_from_infos(module, evk_ggsw_infos),
        }
    }
}

impl<D: DataMut, B: Backend> EvaluationKeysPrepared<D, B> {
    pub fn prepare<O, M>(&mut self, module: &M, other: &EvaluationKeys<O>, scratch: &mut Scratch<B>)
    where
        O: DataRef,
        M: GLWEAutomorphismKeyPreparedFactory<B> + GGLWEToGGSWKeyPreparedFactory<B>,
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
    }
}

impl EvaluationKeys<Vec<u8>> {
    /// Constructor for EvaluationKeys
    pub fn new(
        atk_glwe: HashMap<i64, GLWEAutomorphismKey<Vec<u8>>>,
        atk_ggsw_inv: GLWEAutomorphismKey<Vec<u8>>,
        gglwe_to_ggsw_key: GGLWEToGGSWKey<Vec<u8>>,
    ) -> Self {
        Self {
            atk_glwe,
            atk_ggsw_inv,
            gglwe_to_ggsw_key: gglwe_to_ggsw_key,
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

impl EvaluationKeys<Vec<u8>> {
    pub fn encrypt_sk<S, BE: Backend>(
        params: &Parameters<BE>,
        sk: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
    ) -> EvaluationKeys<Vec<u8>>
    where
        S: GLWESecretToRef + GetDistribution + GLWEInfos,
        Module<BE>: GLWEAutomorphismKeyEncryptSk<BE> + GGLWEToGGSWKeyEncryptSk<BE> + GLWETrace<BE>,
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
                key.encrypt_sk(module, *gal_el, sk, source_xa, source_xe, scratch.borrow());
                (*gal_el, key)
            }));

        let mut gglwe_to_ggsw_key: GGLWEToGGSWKey<Vec<u8>> =
            GGLWEToGGSWKey::alloc_from_infos(&evk_ggsw_infos);
        gglwe_to_ggsw_key.encrypt_sk(module, sk, source_xa, source_xe, scratch.borrow());

        let mut atk_ggsw_inv: GLWEAutomorphismKey<Vec<u8>> =
            GLWEAutomorphismKey::alloc_from_infos(&evk_ggsw_infos);
        atk_ggsw_inv.encrypt_sk(module, -1, sk, source_xa, source_xe, scratch.borrow());

        EvaluationKeys {
            atk_glwe,
            atk_ggsw_inv,
            gglwe_to_ggsw_key: gglwe_to_ggsw_key,
        }
    }
}
