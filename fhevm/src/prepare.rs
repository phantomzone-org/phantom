use std::thread;

use poulpy_core::layouts::{
    GGSWInfos, GGSWPreparedFactory, GGSWLayout,
};
use poulpy_core::{LWEFromGLWE, ScratchTakeCore};
use poulpy_hal::api::{ScratchAvailable};
use poulpy_hal::layouts::{Backend, DataRef, Module};

use poulpy_hal::{
    layouts::{DataMut, Scratch},
};

use poulpy_schemes::bin_fhe::bdd_arithmetic::{
    BDDKeyHelper, BDDKeyInfos, FheUint, FheUintPrepare, FheUintPrepared, GetGGSWBitMut,
};
use poulpy_schemes::bin_fhe::bdd_arithmetic::UnsignedInteger;
use poulpy_schemes::bin_fhe::blind_rotation::BlindRotationAlgo;
use poulpy_schemes::bin_fhe::circuit_bootstrapping::{CircuitBootstrappingKeyInfos, CirtuitBootstrappingExecute};


pub trait PrepareMultiple<BE: Backend, BRA: BlindRotationAlgo> {
    fn prepare_multiple_fheuint<DM, DB, DK, K, T: UnsignedInteger>(
        &self,
        threads: usize,
        res: &mut Vec<&mut FheUintPrepared<DM, T, BE>>,
        bits: Vec<&FheUint<DB, T>>,
        bit_count: Vec<usize>,
        key: &K,
        scratch: &mut Scratch<BE>,
    ) where 
        DM: DataMut,
        DB: DataRef,
        DK: DataRef,
        K: BDDKeyHelper<DK, BRA, BE> + BDDKeyInfos;
}

impl<BE: Backend, BRA: BlindRotationAlgo> PrepareMultiple<BE, BRA> for Module<BE>
where
    Self: LWEFromGLWE<BE> + CirtuitBootstrappingExecute<BRA, BE> + GGSWPreparedFactory<BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
{

    // Assumes all FHEUints start from bit 0
    fn prepare_multiple_fheuint<DM, DB, DK, K, T: UnsignedInteger>(
        &self,
        threads: usize,
        res: &mut Vec<&mut FheUintPrepared<DM, T, BE>>,
        bits: Vec<&FheUint<DB, T>>,
        bit_counts: Vec<usize>,
        key: &K,
        scratch: &mut Scratch<BE>,
    ) where 
        DM: DataMut,
        DB: DataRef,
        DK: DataRef,
        K: BDDKeyHelper<DK, BRA, BE> + BDDKeyInfos,
    {
        let (cbt, ks_glwe, ks_lwe) = key.get_cbt_key();

        let scratch_thread_size = self.fhe_uint_prepare_tmp_bytes(cbt.block_size(), 1, res[0], bits[0], key);

        assert!(
            scratch.available() >= threads * scratch_thread_size,
            "scratch.available():{} < threads:{threads} * scratch_thread_size:{scratch_thread_size}",
            scratch.available()
        );

        let mut combined_res: Vec<_> = res.iter_mut()
            .zip(bit_counts.iter())
            .flat_map(|(r, &count)| r.get_bits(0, count))
            .collect();

        let total_bit_count = bit_counts.iter().sum::<usize>();
        let chunk_size: usize = total_bit_count.div_ceil(threads);

        let (mut scratches, _) = scratch.split_mut(threads, scratch_thread_size);

        let ggsw_infos: &GGSWLayout = &combined_res[0].ggsw_layout();

        let bits_clone = bits.clone();
        let bit_counts_running_sums = std::iter::once(0)
            .chain(bit_counts.iter().scan(0, |sum, &x| { *sum += x; Some(*sum) }))
            .collect::<Vec<_>>();

        thread::scope(|scope| {
            for (thread_index, (scratch_thread, res_bits_chunk)) in scratches
                .iter_mut()
                .zip(combined_res.chunks_mut(chunk_size))
                .enumerate()
            {
                let start: usize = thread_index * chunk_size;

                let bits = bits_clone.clone();
                let bit_counts_running_sums = bit_counts_running_sums.clone();

                scope.spawn(move || {
                    let (mut tmp_ggsw, scratch_1) = scratch_thread.take_ggsw(ggsw_infos);
                    let (mut tmp_lwe, scratch_2) = scratch_1.take_lwe(bits[0]);
                    for (local_bit, dst) in res_bits_chunk.iter_mut().enumerate() {
                        let mut cnt = 0;
                        while bit_counts_running_sums[cnt+1] <= start + local_bit {
                            cnt += 1;
                        }
                        bits[cnt].get_bit_lwe(self, start + local_bit - bit_counts_running_sums[cnt], &mut tmp_lwe, ks_glwe, ks_lwe, scratch_2);

                        cbt.execute_to_constant(self, &mut tmp_ggsw, &tmp_lwe, 1, 1, scratch_2);
                        dst.prepare(self, &tmp_ggsw, scratch_2);
                    }
                });
            }
        });

        for res_index in 0..res.len() {
            for i in bit_counts[res_index]..T::BITS as usize {
                res[res_index].get_bit(i).zero(self);
            }
        }
    }
}
