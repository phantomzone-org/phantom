use std::collections::HashMap;

use poulpy_core::{
    GLWEAdd, GLWECopy, GLWEDecrypt, GLWEPacking, GLWERotate, GLWESub, GLWETrace, ScratchTakeCore, layouts::{GGLWEInfos, GGLWEPreparedToRef, GLWEAutomorphismKeyHelper, GLWEInfos, GLWESecretPreparedToRef, GetGaloisElement}
};
use poulpy_hal::{
    api::ModuleLogN,
    layouts::{Backend, DataMut, DataRef, Scratch},
};
use poulpy_schemes::tfhe::bdd_arithmetic::{
    FheUint, FheUintPrepared, GLWEBlinSelection, UnsignedInteger,
};

use crate::RAM_UPDATE;

pub trait Store<T: UnsignedInteger> {
    fn eval_enc<R, D, A, B, S, H, K, M, BE: Backend>(
        &self,
        threads: usize,
        module: &M,
        res: &mut FheUint<R, T>,
        rs2: &FheUint<A, T>,
        loaded: &FheUint<B, T>,
        offset: &FheUintPrepared<D, u32, BE>,
        keys: &H,
        sk: &S,
        scratch: &mut Scratch<BE>,
    ) where
        R: DataMut,
        A: DataRef,
        D: DataRef,
        B: DataRef,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        M: ModuleLogN
            + GLWEBlinSelection<u32, BE>
            + GLWERotate<BE>
            + GLWETrace<BE>
            + GLWESub
            + GLWEAdd
            + GLWECopy + GLWEDecrypt<BE> + GLWEPacking<BE>,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        Scratch<BE>: ScratchTakeCore<BE>;
}

impl Store<u32> for RAM_UPDATE {
    fn eval_enc<R, D, A, B, S, H, K, M, BE: Backend>(
        &self,
        _threads: usize,
        module: &M,
        res: &mut FheUint<R, u32>,
        rs2: &FheUint<A, u32>,
        loaded: &FheUint<B, u32>,
        offset: &FheUintPrepared<D, u32, BE>,
        keys: &H,
        sk: &S,
        scratch: &mut Scratch<BE>,
    ) where
        R: DataMut,
        A: DataRef,
        D: DataRef,
        B: DataRef,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        M: ModuleLogN
            + GLWEBlinSelection<u32, BE>
            + GLWERotate<BE>
            + GLWETrace<BE>
            + GLWESub
            + GLWEAdd
            + GLWECopy + GLWEDecrypt<BE> + GLWEPacking<BE>,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        match self {
            Self::NONE => {
                module.glwe_copy(res, loaded);
            }

            Self::SW => {
                module.glwe_copy(res, rs2);
            }

            Self::SB => {
                let mut cts: HashMap<usize, FheUint<Vec<u8>, u32>> = HashMap::new();
                for i in 0..4 {
                    let mut tmp: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(res);
                    tmp.splice_u8(module, i, 0, loaded, rs2, keys, scratch);
                    cts.insert(i, tmp);
                }

                let mut cts_ref: HashMap<usize, &mut FheUint<Vec<u8>, u32>> = HashMap::new();
                
                for (key, object) in cts.iter_mut() {
                    println!("key: {key}, value: {}", object.decrypt(module, sk, scratch));
                    cts_ref.insert(*key, object);
                }

                module.glwe_blind_selection(res, cts_ref, offset, 0, 2, scratch);
                println!("offset: {} selected {}", offset.decrypt(module, sk, keys, scratch), res.decrypt(module, sk, scratch));
            }

            Self::SH => {
                let mut cts: HashMap<usize, FheUint<Vec<u8>, u32>> = HashMap::new();
                for i in 0..2 {
                    let mut tmp: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(res);
                    tmp.splice_u16(module, i, 0, loaded, rs2, keys, scratch);
                    cts.insert(i << 1, tmp);
                }
                let mut cts_ref: HashMap<usize, &mut FheUint<Vec<u8>, u32>> = HashMap::new();
                for (key, object) in cts.iter_mut() {
                    cts_ref.insert(*key, object);
                }
                module.glwe_blind_selection(res, cts_ref, offset, 0, 2, scratch);
            }
        }
    }
}
