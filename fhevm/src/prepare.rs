use poulpy_bin_fhe::bdd_arithmetic::{
    BDDKeyHelper, BDDKeyInfos, FheUint, FheUintPrepare, FheUintPrepared, GetGGSWBitMut,
    UnsignedInteger,
};
use poulpy_bin_fhe::blind_rotation::{BlindRotationAlgo, BlindRotationKeyInfos};
use poulpy_bin_fhe::circuit_bootstrapping::CircuitBootstrappingKeyInfos;
use poulpy_core::layouts::{
    prepared::GGLWEPreparedToBackendRef, GGSWInfos, GGSWLayout, GGSWPreparedViewMut, LWEInfos,
    LWELayout,
};
use poulpy_core::ScratchArenaTakeCore;
use poulpy_hal::layouts::{Module, ScratchArena};

#[allow(unused_imports)]
use crate::backend::prelude::*;
use crate::backend::{for_each_with_scratch, PhantomBackend, PhantomBootstrapModule};

pub trait PrepareMultiple<BE: PhantomBackend, BRA: BlindRotationAlgo> {
    /// Bootstraps the first `bit_count[i]` bits of `bits[i]` into `res[i]`,
    /// spreading every bit of every word over `threads` workers.
    ///
    /// The remaining bits of each `res[i]` are zeroed.
    fn prepare_multiple_fheuint<K, T: UnsignedInteger>(
        &self,
        threads: usize,
        res: &mut Vec<&mut FheUintPrepared<Vec<u8>, T, BE>>,
        bits: Vec<&FheUint<Vec<u8>, T, i64>>,
        bit_count: Vec<usize>,
        key: &K,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        K: BDDKeyHelper<Vec<u8>, BRA, BE> + BDDKeyInfos + Sync;
}

impl<BE: PhantomBackend, BRA: BlindRotationAlgo> PrepareMultiple<BE, BRA> for Module<BE>
where
    Self: PhantomBootstrapModule<BRA, BE>,
{
    // Assumes all FHEUints start from bit 0
    fn prepare_multiple_fheuint<K, T: UnsignedInteger>(
        &self,
        threads: usize,
        res: &mut Vec<&mut FheUintPrepared<Vec<u8>, T, BE>>,
        bits: Vec<&FheUint<Vec<u8>, T, i64>>,
        bit_counts: Vec<usize>,
        key: &K,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        K: BDDKeyHelper<Vec<u8>, BRA, BE> + BDDKeyInfos + Sync,
    {
        assert_eq!(res.len(), bits.len());
        assert_eq!(res.len(), bit_counts.len());

        let (cbt, ks_glwe, ks_lwe) = key.get_cbt_key();

        // Bits beyond the requested count are zeroed.
        for (dst, &count) in res.iter_mut().zip(bit_counts.iter()) {
            for i in count..T::BITS as usize {
                self.ggsw_zero(&mut GGSWPreparedViewMut::from_inner(dst.get_bit(i)));
            }
        }

        let total_bit_count: usize = bit_counts.iter().sum::<usize>();
        if total_bit_count == 0 {
            return;
        }

        // Per worker: the temporaries of `fhe_uint_prepare_tmp_bytes`, which
        // already includes the GGSW and LWE temporaries below.
        let scratch_thread_size: usize =
            self.fhe_uint_prepare_tmp_bytes(cbt.block_size(), 1, res[0], bits[0], key);

        let ggsw_infos: GGSWLayout = res[0].ggsw_layout();
        let mut lwe_infos: LWELayout = bits[0].lwe_layout();
        lwe_infos.n = cbt.brk_infos().n_lwe();

        // Prefix sums of the bit counts, to map a global bit index back to
        // its word and local bit.
        let bit_counts_running_sums: Vec<usize> = std::iter::once(0)
            .chain(bit_counts.iter().scan(0, |sum, &x| {
                *sum += x;
                Some(*sum)
            }))
            .collect();

        let mut combined_res: Vec<GGSWPreparedViewMut<'_, BE>> = res
            .iter_mut()
            .zip(bit_counts.iter())
            .flat_map(|(r, &count)| r.get_bits(0, count))
            .map(GGSWPreparedViewMut::from_inner)
            .collect();

        let bits: &[&FheUint<Vec<u8>, T, i64>] = &bits;
        let bit_counts_running_sums: &[usize] = &bit_counts_running_sums;

        for_each_with_scratch(
            threads,
            &mut combined_res,
            0,
            scratch_thread_size,
            scratch,
            &|global_bit, dst, scratch| {
                let mut word: usize = 0;
                while bit_counts_running_sums[word + 1] <= global_bit {
                    word += 1;
                }
                let local_bit: usize = global_bit - bit_counts_running_sums[word];

                let (mut tmp_ggsw, scratch_1) = scratch.borrow().take_ggsw_scratch(&ggsw_infos);
                let (mut tmp_lwe, mut scratch_2) = scratch_1.take_lwe_scratch(&lwe_infos);
                let ks_glwe_ref = ks_glwe.map(|k| k.to_backend_ref());
                bits[word].get_bit_lwe(
                    self,
                    local_bit,
                    &mut tmp_lwe,
                    ks_glwe_ref.as_ref(),
                    &ks_lwe.to_backend_ref(),
                    &mut scratch_2,
                );

                cbt.execute_to_constant(
                    self,
                    &mut tmp_ggsw.to_backend_mut(),
                    &tmp_lwe,
                    1,
                    1,
                    &mut scratch_2,
                );
                self.ggsw_prepare(dst, &tmp_ggsw, &mut scratch_2);
            },
        );
    }
}
