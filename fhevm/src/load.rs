use poulpy_core::{GLWEAdd, GLWECopy, GLWERotate, GLWESub, GLWETrace, ScratchTakeCore};
use poulpy_hal::{
    api::ModuleLogN,
    layouts::{Backend, DataMut, DataRef, Scratch},
};
use poulpy_schemes::tfhe::bdd_arithmetic::{FheUint, UnsignedInteger};

use crate::{keys::RAMKeysHelper, LoadOps, OpIDRd};

pub trait Load<T: UnsignedInteger, BE: Backend> {
    fn id(&self) -> u32;

    fn load<R, A, H, K, M>(
        &self,
        module: &M,
        res: &mut FheUint<R, u32>,
        a: &FheUint<A, u32>,
        key: &H,
        scratch: &mut Scratch<BE>,
    ) where
        R: DataMut,
        A: DataRef,
        K: DataRef,
        H: RAMKeysHelper<K, BE>,
        M: ModuleLogN + GLWERotate<BE> + GLWETrace<BE> + GLWESub + GLWEAdd + GLWECopy,
        Scratch<BE>: ScratchTakeCore<BE>;
}

impl<BE: Backend> Load<u32, BE> for LoadOps {
    fn id(&self) -> u32 {
        match self {
            Self::Lb => OpIDRd::LB,
            Self::Lbu => OpIDRd::LBU,
            Self::Lh => OpIDRd::LH,
            Self::Lhu => OpIDRd::LHU,
            Self::Lw => OpIDRd::LW,
        }
    }

    fn load<R, A, H, K, M>(
        &self,
        module: &M,
        res: &mut FheUint<R, u32>,
        a: &FheUint<A, u32>,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        R: DataMut,
        A: DataRef,
        K: DataRef,
        H: RAMKeysHelper<K, BE>,
        M: ModuleLogN + GLWERotate<BE> + GLWETrace<BE> + GLWESub + GLWEAdd + GLWECopy,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        module.glwe_copy(res, a);

        match self {
            Self::Lb => {
                res.zero_byte(module, 1, keys, scratch);
                res.zero_byte(module, 2, keys, scratch);
                res.zero_byte(module, 3, keys, scratch);
            }
            Self::Lbu => {
                res.zero_byte(module, 1, keys, scratch);
                res.zero_byte(module, 2, keys, scratch);
                res.zero_byte(module, 3, keys, scratch);
                res.sext(module, 0, keys, scratch);
            }
            Self::Lh => {
                res.zero_byte(module, 2, keys, scratch);
                res.zero_byte(module, 3, keys, scratch);
            }
            Self::Lhu => {
                res.zero_byte(module, 2, keys, scratch);
                res.zero_byte(module, 3, keys, scratch);
                res.sext(module, 1, keys, scratch);
            }
            Self::Lw => {}
        }
    }
}
