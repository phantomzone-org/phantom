use std::marker::PhantomData;
use std::thread;

use poulpy_core::layouts::{
    Base2K, Dnum, Dsize, GGSWInfos, GGSWPreparedFactory, GLWEInfos, LWEInfos, Rank, TorusPrecision, prepared::GGSWPrepared,
};
use poulpy_core::layouts::{
    GGLWEInfos, GGLWEPreparedToRef, GGSW, GGSWLayout, GGSWPreparedToMut, GGSWPreparedToRef, GLWEAutomorphismKeyHelper,
    GetGaloisElement, LWE,
};
use poulpy_core::{GLWECopy, GLWEDecrypt, GLWEPacking, LWEFromGLWE};

use poulpy_core::{GGSWEncryptSk, ScratchTakeCore, layouts::GLWESecretPreparedToRef};
use poulpy_hal::api::{ModuleLogN, ScratchAvailable, ScratchFromBytes};
use poulpy_hal::layouts::{Backend, Data, DataRef, Module};

use poulpy_hal::{
    api::ModuleN,
    layouts::{DataMut, Scratch},
    source::Source,
};

use poulpy_schemes::tfhe::bdd_arithmetic::{
    BDDKey, BDDKeyHelper, BDDKeyInfos, BDDKeyPrepared, BDDKeyPreparedFactory, BitSize, FheUint, FheUintPrepare, FheUintPrepared, ToBits
};
use poulpy_schemes::tfhe::bdd_arithmetic::{Cmux, FromBits, ScratchTakeBDD, UnsignedInteger};
use poulpy_schemes::tfhe::blind_rotation::BlindRotationAlgo;
use poulpy_schemes::tfhe::circuit_bootstrapping::{CircuitBootstrappingKeyInfos, CirtuitBootstrappingExecute};


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
        let (cbt, ks) = key.get_cbt_key();

        let scratch_thread_size = self.fhe_uint_prepare_tmp_bytes(cbt.block_size(), 1, res[0], bits[0], key);

        assert!(
            scratch.available() >= threads * scratch_thread_size,
            "scratch.available():{} < threads:{threads} * scratch_thread_size:{scratch_thread_size}",
            scratch.available()
        );

        // let combined_res: Vec<&mut GGSWPrepared<DM, BE>> = res.iter().enumerate().map(|(i, r)| r.bits[..bit_counts[i]]).flatten().collect();
        let mut combined_res: Vec<_> = res.iter_mut()
            .zip(bit_counts.iter())
            .flat_map(|(r, &count)| r.bits[..count].iter_mut())
            .collect();

        let total_bit_count = bit_counts.iter().sum::<usize>();
        let chunk_size: usize = total_bit_count.div_ceil(threads);

        let (mut scratches, _) = scratch.split_mut(threads, scratch_thread_size);

        let ggsw_infos: &GGSWLayout = &combined_res[0].ggsw_layout();

        let bits_clone = bits.clone();
        let bit_counts_clone = bit_counts.clone();

        thread::scope(|scope| {
            for (thread_index, (scratch_thread, res_bits_chunk)) in scratches
                .iter_mut()
                .zip(combined_res.chunks_mut(chunk_size))
                .enumerate()
            {
                let start: usize = thread_index * chunk_size;

                let bits = bits_clone.clone();
                let bit_counts = bit_counts_clone.clone();

                scope.spawn(move || {
                    let (mut tmp_ggsw, scratch_1) = scratch_thread.take_ggsw(ggsw_infos);
                    let (mut tmp_lwe, scratch_2) = scratch_1.take_lwe(bits[0]);
                    for (local_bit, dst) in res_bits_chunk.iter_mut().enumerate() {
                        // bits[0].get_bit_lwe(self, start + local_bit, &mut tmp_lwe, ks, scratch_2);
                        let mut start_cnt = 1;
                        let mut sum_til_now_prev = 0;
                        let mut sum_til_now = bit_counts[0];
                        while sum_til_now <= start + local_bit {
                            sum_til_now_prev += bit_counts[start_cnt-1];
                            sum_til_now += bit_counts[start_cnt];
                            start_cnt += 1;
                        }
                        bits[start_cnt-1].get_bit_lwe(self, start + local_bit - sum_til_now_prev, &mut tmp_lwe, ks, scratch_2);

                        cbt.execute_to_constant(self, &mut tmp_ggsw, &tmp_lwe, 1, 1, scratch_2);
                        dst.prepare(self, &tmp_ggsw, scratch_2);
                    }
                });
            }
        });

        for res_index in 0..res.len() {
            for i in bit_counts[res_index]..T::BITS as usize {
                res[res_index].bits[i].zero(self);
            }
        }
    }
}
