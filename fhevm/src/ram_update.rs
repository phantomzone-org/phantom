use std::collections::HashMap;

use poulpy_bin_fhe::bdd_arithmetic::{FheUint, FheUintPrepared, UnsignedInteger};
use poulpy_core::layouts::GetAutomorphismKey;
use poulpy_hal::layouts::{Module, ScratchArena};

#[allow(unused_imports)]
use crate::backend::prelude::*;
use crate::{
    backend::{PhantomBackend, PhantomModule},
    RAM_UPDATE,
};

pub trait Store<T: UnsignedInteger> {
    #[allow(clippy::too_many_arguments)]
    fn eval_enc<H, BE: PhantomBackend>(
        &self,
        threads: usize,
        module: &Module<BE>,
        res: &mut FheUint<Vec<u8>, T, i64>,
        rs2: &FheUint<Vec<u8>, T, i64>,
        loaded: &FheUint<Vec<u8>, T, i64>,
        offset: &FheUintPrepared<Vec<u8>, u32, BE>,
        keys: &H,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        H: GetAutomorphismKey<BE>,
        Module<BE>: PhantomModule<BE>;
}

impl Store<u32> for RAM_UPDATE {
    fn eval_enc<H, BE: PhantomBackend>(
        &self,
        _threads: usize,
        module: &Module<BE>,
        res: &mut FheUint<Vec<u8>, u32, i64>,
        rs2: &FheUint<Vec<u8>, u32, i64>,
        loaded: &FheUint<Vec<u8>, u32, i64>,
        offset: &FheUintPrepared<Vec<u8>, u32, BE>,
        keys: &H,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        H: GetAutomorphismKey<BE>,
        Module<BE>: PhantomModule<BE>,
    {
        match self {
            Self::NONE => {
                module.glwe_copy(res, loaded);
            }

            Self::SW => {
                module.glwe_copy(res, rs2);
            }

            Self::SB => {
                let mut cts: HashMap<usize, FheUint<Vec<u8>, u32, i64>> = HashMap::new();
                for i in 0..4 {
                    let mut tmp: FheUint<Vec<u8>, u32, i64> =
                        FheUint::alloc_from_infos(module, res);
                    tmp.splice_u8(module, i, 0, loaded, rs2, keys, scratch);
                    cts.insert(i, tmp);
                }
                let mut cts_ref: HashMap<usize, &mut FheUint<Vec<u8>, u32, i64>> = HashMap::new();
                for (key, object) in cts.iter_mut() {
                    cts_ref.insert(*key, object);
                }
                module.glwe_blind_selection(res, cts_ref, offset, 0, 2, scratch);
            }

            Self::SH => {
                let mut cts: HashMap<usize, FheUint<Vec<u8>, u32, i64>> = HashMap::new();
                for i in 0..2 {
                    let mut tmp: FheUint<Vec<u8>, u32, i64> =
                        FheUint::alloc_from_infos(module, res);
                    tmp.splice_u16(module, i, 0, loaded, rs2, keys, scratch);
                    cts.insert(i << 1, tmp);
                }
                let mut cts_ref: HashMap<usize, &mut FheUint<Vec<u8>, u32, i64>> = HashMap::new();
                for (key, object) in cts.iter_mut() {
                    cts_ref.insert(*key, object);
                }
                module.glwe_blind_selection(res, cts_ref, offset, 0, 2, scratch);
            }
        }
    }
}
