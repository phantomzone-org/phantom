use poulpy_hal::{
    api::ModuleN,
    layouts::{Backend, Data, DataMut, DataRef, Module, Scratch},
};

use poulpy_core::{
    GGSWAutomorphism, GLWEExternalProduct, ScratchTakeCore,
    layouts::{
        GGLWEInfos, GGLWEPreparedToRef, GGLWEToGGSWKeyPreparedToRef, GGSWInfos, GGSWPrepared,
        GGSWPreparedFactory, GLWEInfos, GLWEToMut, GLWEToRef, GetGaloisElement, LWEInfos,
    },
};

use crate::{Base1D, Coordinate};

pub(crate) struct CoordinatePrepared<D: Data, B: Backend> {
    pub(crate) value: Vec<GGSWPrepared<D, B>>,
    pub(crate) base1d: Base1D,
}

impl<B: Backend> CoordinatePrepared<Vec<u8>, B>
where
    Module<B>: GGSWPreparedFactory<B>,
{
    pub(crate) fn alloc_bytes<A>(module: &Module<B>, infos: &A, size: usize) -> usize
    where
        A: GGSWInfos,
    {
        size * GGSWPrepared::bytes_of_from_infos(module, infos)
    }
}

impl<D: Data, B: Backend> LWEInfos for CoordinatePrepared<D, B> {
    fn base2k(&self) -> poulpy_core::layouts::Base2K {
        self.value[0].base2k()
    }

    fn k(&self) -> poulpy_core::layouts::TorusPrecision {
        self.value[0].k()
    }

    fn n(&self) -> poulpy_core::layouts::Degree {
        self.value[0].n()
    }
}

impl<D: Data, B: Backend> GLWEInfos for CoordinatePrepared<D, B> {
    fn rank(&self) -> poulpy_core::layouts::Rank {
        self.value[0].rank()
    }
}

impl<D: Data, B: Backend> GGSWInfos for CoordinatePrepared<D, B> {
    fn dnum(&self) -> poulpy_core::layouts::Dnum {
        self.value[0].dnum()
    }

    fn dsize(&self) -> poulpy_core::layouts::Dsize {
        self.value[0].dsize()
    }
}

pub(crate) trait TakeCoordinatePrepared<B: Backend> {
    fn take_coordinate_prepared<A, M>(
        &mut self,
        module: &M,
        infos: &A,
        base1d: &Base1D,
    ) -> (CoordinatePrepared<&mut [u8], B>, &mut Self)
    where
        M: ModuleN + GGSWPreparedFactory<B>,
        A: GGSWInfos;
}

impl<B: Backend> TakeCoordinatePrepared<B> for Scratch<B>
where
    Self: ScratchTakeCore<B>,
{
    fn take_coordinate_prepared<A, M>(
        &mut self,
        module: &M,
        infos: &A,
        base1d: &Base1D,
    ) -> (CoordinatePrepared<&mut [u8], B>, &mut Self)
    where
        A: GGSWInfos,
        M: ModuleN + GGSWPreparedFactory<B>,
    {
        let (ggsws, scratch) = self.take_ggsw_prepared_slice(module, base1d.0.len(), infos);
        (
            CoordinatePrepared {
                value: ggsws,
                base1d: base1d.clone(),
            },
            scratch,
        )
    }
}

impl<DM: DataMut, B: Backend> CoordinatePrepared<DM, B>
where
    Module<B>: GGSWPreparedFactory<B>,
{
    pub(crate) fn prepare<DR: DataRef>(
        &mut self,
        module: &Module<B>,
        other: &Coordinate<DR>,
        scratch: &mut Scratch<B>,
    ) where
        DR: DataRef,
    {
        assert_eq!(self.base1d, other.base1d);
        for (el_prep, el) in self.value.iter_mut().zip(other.value.iter()) {
            el_prep.prepare(module, el, scratch)
        }
    }
}

impl<D: DataMut, B: Backend> CoordinatePrepared<D, B> {
    /// Maps GGSW(X^{i}) to GGSW(X^{-i}).
    pub(crate) fn prepare_inv<DR: DataRef, M, G, T>(
        &mut self,
        module: &M,
        other: &Coordinate<DR>,
        auto_key: &G,
        tensor_key: &T,
        scratch: &mut Scratch<B>,
    ) where
        G: GGLWEPreparedToRef<B> + GetGaloisElement + GGLWEInfos,
        T: GGLWEToGGSWKeyPreparedToRef<B>,
        M: GGSWAutomorphism<B> + GGSWPreparedFactory<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        assert!(auto_key.p() == -1);
        assert_eq!(self.base1d, other.base1d);
        let (mut tmp_ggsw, scratch_1) = scratch.take_ggsw(other);
        for (prepared, other) in self.value.iter_mut().zip(other.value.iter()) {
            tmp_ggsw.automorphism(module, other, auto_key, tensor_key, scratch_1);
            prepared.prepare(module, &tmp_ggsw, scratch_1);
        }
        self.base1d = other.base1d.clone();
    }
}

impl<D: DataRef, B: Backend> CoordinatePrepared<D, B> {
    /// Evaluates GLWE(m) * GGSW(X^i).
    pub(crate) fn product<R, A, M>(&self, module: &M, res: &mut R, a: &A, scratch: &mut Scratch<B>)
    where
        R: GLWEToMut,
        A: GLWEToRef,
        M: GLWEExternalProduct<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        for (i, coordinate) in self.value.iter().enumerate() {
            if i == 0 {
                module.glwe_external_product(res, a, coordinate, scratch);
            } else {
                module.glwe_external_product_inplace(res, coordinate, scratch);
            }
        }
    }

    /// Evaluates GLWE(m) * GGSW(X^i).
    pub(crate) fn product_inplace<R>(
        &self,
        module: &Module<B>,
        res: &mut R,
        scratch: &mut Scratch<B>,
    ) where
        R: GLWEToMut,
        Module<B>: GLWEExternalProduct<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        for coordinate in self.value.iter() {
            module.glwe_external_product_inplace(res, coordinate, scratch);
        }
    }
}
