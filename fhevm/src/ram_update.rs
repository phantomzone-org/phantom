use std::collections::HashMap;

use poulpy_core::{GLWEAdd, GLWECopy, GLWERotate, GLWESub, GLWETrace, ScratchTakeCore};
use poulpy_hal::{
    api::ModuleLogN,
    layouts::{Backend, DataMut, DataRef, Scratch},
};
use poulpy_schemes::tfhe::bdd_arithmetic::{
    FheUint, FheUintPrepared, GLWEBlinSelection, UnsignedInteger,
};

use crate::{keys::RAMKeysHelper, RAM_UPDATE};

pub trait Store<T: UnsignedInteger> {
    fn eval_enc<R, D, A, H, K, M, BE: Backend>(
        &self,
        module: &M,
        res: &mut FheUint<R, T>,
        rs2: &FheUint<A, T>,
        loaded: &FheUint<A, T>,
        offset: &FheUintPrepared<D, u32, BE>,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        R: DataMut,
        A: DataRef,
        K: DataRef,
        D: DataRef,
        H: RAMKeysHelper<K, BE>,
        M: ModuleLogN
            + GLWEBlinSelection<u32, BE>
            + GLWERotate<BE>
            + GLWETrace<BE>
            + GLWESub
            + GLWEAdd
            + GLWECopy,
        Scratch<BE>: ScratchTakeCore<BE>;
}

impl Store<u32> for RAM_UPDATE {
    fn eval_enc<R, D, A, H, K, M, BE: Backend>(
        &self,
        module: &M,
        res: &mut FheUint<R, u32>,
        rs2: &FheUint<A, u32>,
        loaded: &FheUint<A, u32>,
        offset: &FheUintPrepared<D, u32, BE>,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        R: DataMut,
        A: DataRef,
        K: DataRef,
        D: DataRef,
        H: RAMKeysHelper<K, BE>,
        M: ModuleLogN
            + GLWEBlinSelection<u32, BE>
            + GLWERotate<BE>
            + GLWETrace<BE>
            + GLWESub
            + GLWEAdd
            + GLWECopy,
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
                    cts_ref.insert(*key, object);
                }
                module.glwe_blind_selection(res, cts_ref, offset, 0, 4, scratch);
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
                module.glwe_blind_selection(res, cts_ref, offset, 0, 4, scratch);
            }
        }
    }
}
