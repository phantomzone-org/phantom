use std::f64;

use poulpy_bin_fhe::bdd_arithmetic::{
    FheUint, FheUintPrepared, FromBits, GLWEBlindRetriever, GetGGSWBit, ToBits,
};
use poulpy_core::{
    layouts::{
        GGLWEInfos, GGSWInfos, GLWEInfos, GLWELayout, GLWEPlaintext, GLWEPlaintextLayout,
        GLWESecretPreparedToBackendRef, GLWEToBackendMut, GetAutomorphismKey, TorusPrecision, GLWE,
    },
    ScratchArenaTakeCore,
};
use poulpy_hal::{
    layouts::{Module, ScratchArena},
    source::Source,
};

#[allow(unused_imports)]
use crate::backend::prelude::*;
use crate::backend::{default_noise_infos, for_each_with_scratch, PhantomBackend, PhantomModule};

pub struct Memory {
    bits: Vec<BitArray>,
    size: usize,
    bit_size: usize,
    state: bool,
}

impl Memory {
    pub(crate) fn alloc<A, BE: PhantomBackend>(
        module: &Module<BE>,
        infos: &A,
        word_size: usize,
        size: usize,
    ) -> Self
    where
        A: GLWEInfos,
        Module<BE>: PhantomModule<BE>,
    {
        Self {
            bits: (0..word_size)
                .map(|_| BitArray::alloc(module, infos, size))
                .collect(),
            size,
            bit_size: (usize::BITS - (size - 1).leading_zeros()) as usize,
            state: false,
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn encrypt_sk<S, BE: PhantomBackend>(
        &mut self,
        module: &Module<BE>,
        data: &[u32],
        sk: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        S: GLWESecretPreparedToBackendRef<BE> + GLWEInfos,
        u32: ToBits,
    {
        let size: usize = self.size;
        let ram_chunks: usize = self.bits.len();

        assert!(data.len() / ram_chunks <= size);

        let mut bits: Vec<u8> = vec![0u8; size];

        for i in 0..ram_chunks {
            for (x, y) in bits.iter_mut().zip(data.iter()) {
                *x = y.bit(i);
            }
            self.bits[i].encrypt_sk(module, &bits, sk, source_xa, source_xe, scratch);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn decrypt<S, BE: PhantomBackend>(
        &self,
        module: &Module<BE>,
        data: &mut [u32],
        sk: &S,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        S: GLWESecretPreparedToBackendRef<BE> + GLWEInfos,
        u32: FromBits,
    {
        let max_addr: usize = self.size;
        let ram_chunks: usize = self.bits.len();

        assert!(data.len() / ram_chunks <= max_addr);

        let mut bits: Vec<u8> = vec![0u8; max_addr];

        for i in 0..ram_chunks {
            self.bits[i].decrypt(module, bits.as_mut_slice(), sk, scratch);

            for (x, y) in bits.iter().zip(data.iter_mut()) {
                if *x == 1 {
                    *y |= 1 << i;
                } else {
                    *y &= !(1 << i);
                }
            }
        }
    }

    pub(crate) fn noise<S, BE: PhantomBackend>(
        &self,
        module: &Module<BE>,
        data: &[u32],
        sk: &S,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Vec<f64>
    where
        Module<BE>: PhantomModule<BE>,
        S: GLWESecretPreparedToBackendRef<BE> + GLWEInfos,
        u32: FromBits,
    {
        let max_addr: usize = self.size;
        let ram_chunks: usize = self.bits.len();

        assert!(data.len() / ram_chunks <= max_addr);

        let mut bits: Vec<u8> = vec![0u8; max_addr];
        let mut noise: Vec<f64> = vec![0f64; self.bits.len()];
        for i in 0..ram_chunks {
            for (x, y) in bits.iter_mut().zip(data.iter()) {
                *x = y.bit(i)
            }
            noise[i] = self.bits[i].noise(module, bits.as_slice(), sk, scratch);
        }

        noise
    }

    /// Zeroes the word at `addr`.
    pub(crate) fn zero<BE: PhantomBackend, H>(
        &mut self,
        threads: usize,
        module: &Module<BE>,
        addr: usize,
        keys: &H,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        H: Sync + GetAutomorphismKey<BE>,
    {
        let poly: usize = addr / module.n();
        let idx: usize = poly % module.n();

        let glwe_infos: GLWELayout = self.bits[0].data[0].glwe_layout();
        let key_infos = keys
            .get_automorphism_key(-1, glwe_infos.k())
            .unwrap_or_else(|e| panic!("{e}"));

        let scratch_thread_size: usize = crate::backend::worker_bytes::<BE>(
            module
                .glwe_rotate_tmp_bytes()
                .max(module.glwe_trace_tmp_bytes(&glwe_infos, &glwe_infos, &key_infos))
                + module.glwe_bytes_of_from_infos(&glwe_infos),
        );

        for_each_with_scratch(
            threads,
            &mut self.bits,
            0,
            scratch_thread_size,
            scratch,
            &|_, subram, scratch| {
                let (mut tmp, mut scratch_1) = scratch.borrow().take_glwe_scratch(&glwe_infos);
                let a: &mut GLWE<Vec<u8>, i64> = &mut subram.data[poly];
                module.glwe_rotate_assign(-(idx as i64), a, &mut scratch_1);
                module.glwe_trace(&mut tmp, 0, a, keys, &mut scratch_1);
                module.glwe_sub_assign(a, &tmp);
                module.glwe_rotate_assign(idx as i64, a, &mut scratch_1);
            },
        );
    }

    pub(crate) fn read_stateless<H, BE: PhantomBackend>(
        &mut self,
        threads: usize,
        module: &Module<BE>,
        res: &mut FheUint<Vec<u8>, u32, i64>,
        address: &FheUintPrepared<Vec<u8>, u32, BE>,
        offset: usize,
        keys: &H,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        H: Sync + GetAutomorphismKey<BE>,
    {
        assert!(
            !self.bits.is_empty(),
            "unitialized memory: self.data.len()=0"
        );

        assert!(!self.state);

        let (mut tmp_res, mut scratch_1) = scratch
            .borrow()
            .take_glwe_slice_scratch(self.bits.len(), res);

        let scratch_thread_size: usize = crate::backend::worker_bytes::<BE>(
            BitArray::retrieve_stateless_tmp_bytes(module, &self.bits[0].data[0], address),
        );

        let mut items: Vec<(&mut BitArray, &mut _)> =
            self.bits.iter_mut().zip(tmp_res.iter_mut()).collect();

        for_each_with_scratch(
            threads,
            &mut items,
            0,
            scratch_thread_size,
            &mut scratch_1,
            &|_, (subram, res), scratch| {
                subram.retrieve_stateless(module, *res, address, offset, scratch);
            },
        );

        res.pack(module, tmp_res, keys, &mut scratch_1);
    }

    pub(crate) fn read_statefull<A, H, BE: PhantomBackend>(
        &mut self,
        threads: usize,
        module: &Module<BE>,
        res: &mut FheUint<Vec<u8>, u32, i64>,
        address: &A,
        offset: usize,
        keys: &H,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        A: Sync + GetGGSWBit<BE> + GGSWInfos + 'static,
        H: Sync + GetAutomorphismKey<BE>,
    {
        assert!(
            !self.bits.is_empty(),
            "unitialized memory: self.data.len()=0"
        );

        assert!(!self.state);

        let (mut tmp_res, mut scratch_1) = scratch
            .borrow()
            .take_glwe_slice_scratch(self.bits.len(), res);

        let scratch_thread_size: usize =
            crate::backend::worker_bytes::<BE>(BitArray::retrieve_statefull_tmp_bytes(
                module,
                self.bit_size,
                &self.bits[0].data[0],
                address,
            ));

        let mut items: Vec<(&mut BitArray, &mut _)> =
            self.bits.iter_mut().zip(tmp_res.iter_mut()).collect();

        for_each_with_scratch(
            threads,
            &mut items,
            0,
            scratch_thread_size,
            &mut scratch_1,
            &|_, (subram, res), scratch| {
                subram.retrieve_statefull(module, *res, address, offset, scratch);
            },
        );

        res.pack(module, tmp_res, keys, &mut scratch_1);

        self.state = true;
    }

    pub(crate) fn read_statefull_rev<A, H, BE: PhantomBackend>(
        &mut self,
        threads: usize,
        module: &Module<BE>,
        w: &FheUint<Vec<u8>, u32, i64>, // Must encrypt [w, 0, 0, ..., 0];
        address: &A,
        offset: usize,
        keys: &H,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        A: Sync + GetGGSWBit<BE> + GGSWInfos + 'static,
        H: Sync + GetAutomorphismKey<BE>,
    {
        assert!(self.state);

        let key_infos = keys
            .get_automorphism_key(-1, w.k())
            .unwrap_or_else(|e| panic!("{e}"));

        let scratch_thread_size: usize = crate::backend::worker_bytes::<BE>(
            BitArray::retrieve_statefull_rev_tmp_bytes(
                module,
                self.bit_size,
                &self.bits[0].data[0],
                address,
                &key_infos,
            ) + module.glwe_bytes_of_from_infos(w),
        );

        for_each_with_scratch(
            threads,
            &mut self.bits,
            0,
            scratch_thread_size,
            scratch,
            &|bit, subram, scratch| {
                // Overwrites the coefficient that was read: to_write_on = to_write_on - TRACE(to_write_on) + w
                let (mut bit_glwe, mut scratch_1) = scratch.borrow().take_glwe_scratch(w);
                w.get_bit_glwe(module, bit, &mut bit_glwe, keys, &mut scratch_1);
                subram.retrieve_statefull_rev(
                    module,
                    &bit_glwe,
                    address,
                    keys,
                    offset,
                    &mut scratch_1,
                );
            },
        );

        self.state = false;
    }

    pub(crate) fn write<A, H, BE: PhantomBackend>(
        &mut self,
        threads: usize,
        module: &Module<BE>,
        w: &FheUint<Vec<u8>, u32, i64>, // Must encrypt [w, 0, 0, ..., 0];
        address: &A,
        offset: usize,
        keys: &H,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        A: Sync + GetGGSWBit<BE> + GGSWInfos + 'static,
        H: Sync + GetAutomorphismKey<BE>,
    {
        assert!(!self.state);

        let key_infos = keys
            .get_automorphism_key(-1, w.k())
            .unwrap_or_else(|e| panic!("{e}"));

        let scratch_thread_size: usize = crate::backend::worker_bytes::<BE>(
            BitArray::write_tmp_bytes(
                module,
                self.bit_size,
                &self.bits[0].data[0],
                address,
                &key_infos,
            ) + module.glwe_bytes_of_from_infos(w),
        );

        for_each_with_scratch(
            threads,
            &mut self.bits,
            0,
            scratch_thread_size,
            scratch,
            &|bit, subram, scratch| {
                // Overwrites the coefficient that was read: to_write_on = to_write_on - TRACE(to_write_on) + w
                let (mut bit_glwe, mut scratch_1) = scratch.borrow().take_glwe_scratch(w);
                w.get_bit_glwe(module, bit, &mut bit_glwe, keys, &mut scratch_1);
                subram.write(module, &bit_glwe, address, keys, offset, &mut scratch_1);
            },
        );
    }
}

struct BitArray {
    data: Vec<GLWE<Vec<u8>, i64>>,
    retriever: GLWEBlindRetriever<Vec<u8>, i64>,
    bit_size: usize,
}

impl BitArray {
    fn alloc<A, BE: PhantomBackend>(module: &Module<BE>, infos: &A, size: usize) -> Self
    where
        A: GLWEInfos,
        Module<BE>: PhantomModule<BE>,
    {
        let n: usize = infos.n().into();
        Self {
            data: (0..size.div_ceil(n))
                .map(|_| module.glwe_alloc_from_infos(infos))
                .collect(),
            retriever: GLWEBlindRetriever::alloc(module, infos, size),
            bit_size: (usize::BITS - (size - 1).leading_zeros()) as usize,
        }
    }

    fn plaintext_infos(&self) -> GLWEPlaintextLayout {
        GLWEPlaintextLayout {
            n: self.data[0].n(),
            base2k: self.data[0].base2k(),
            k: TorusPrecision(2),
        }
    }

    fn encrypt_sk<BE: PhantomBackend, S>(
        &mut self,
        module: &Module<BE>,
        data: &[u8],
        sk_prepared: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        S: GLWESecretPreparedToBackendRef<BE> + GLWEInfos,
    {
        let noise = default_noise_infos(&self.data[0]);
        let mut pt: GLWEPlaintext<Vec<u8>, i64> =
            module.glwe_plaintext_alloc_from_infos(&self.plaintext_infos());
        let mut data_i64: Vec<i64> = vec![0i64; module.n()];

        for (chunk, ct) in data.chunks(module.n()).zip(self.data.iter_mut()) {
            data_i64.fill(0);

            for (y, x) in data_i64.iter_mut().zip(chunk.iter()) {
                *y = *x as i64
            }

            pt.encode_vec_i64(&data_i64, TorusPrecision(2));
            module.glwe_encrypt_sk(ct, &pt, sk_prepared, &noise, source_xe, source_xa, scratch);
        }
    }

    #[allow(dead_code)]
    fn decrypt<BE: PhantomBackend, S>(
        &self,
        module: &Module<BE>,
        data: &mut [u8],
        sk_prepared: &S,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        S: GLWESecretPreparedToBackendRef<BE> + GLWEInfos,
    {
        let mut pt: GLWEPlaintext<Vec<u8>, i64> =
            module.glwe_plaintext_alloc_from_infos(&self.plaintext_infos());
        let mut data_i64: Vec<i64> = vec![0i64; module.n()];

        for (chunk, ct) in data.chunks_mut(module.n()).zip(self.data.iter()) {
            module.glwe_decrypt(ct, &mut pt, sk_prepared, scratch);
            pt.decode_vec_i64(&mut data_i64, TorusPrecision(2));
            for (y, x) in data_i64.iter_mut().zip(chunk.iter_mut()) {
                *x = *y as u8;
            }
        }
    }

    #[allow(dead_code)]
    fn noise<BE: PhantomBackend, S>(
        &self,
        module: &Module<BE>,
        data: &[u8],
        sk_prepared: &S,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> f64
    where
        Module<BE>: PhantomModule<BE>,
        S: GLWESecretPreparedToBackendRef<BE> + GLWEInfos,
    {
        let mut pt: GLWEPlaintext<Vec<u8>, i64> =
            module.glwe_plaintext_alloc_from_infos(&self.plaintext_infos());
        let mut data_i64: Vec<i64> = vec![0i64; module.n()];

        let mut max_noise: f64 = f64::MIN;

        for (chunk, ct) in data.chunks(module.n()).zip(self.data.iter()) {
            data_i64.fill(0);
            for (y, x) in data_i64.iter_mut().zip(chunk.iter()) {
                *y = *x as i64;
            }
            pt.encode_vec_i64(&data_i64, TorusPrecision(2));
            max_noise = max_noise.max(
                module
                    .glwe_noise(ct, &pt, sk_prepared, scratch)
                    .max()
                    .log2(),
            );
        }

        max_noise
    }

    fn retrieve_stateless_tmp_bytes<R, A, BE: PhantomBackend>(
        module: &Module<BE>,
        res: &R,
        addr: &A,
    ) -> usize
    where
        Module<BE>: PhantomModule<BE>,
        R: GLWEInfos,
        A: GGSWInfos,
    {
        GLWEBlindRetriever::retrieve_tmp_bytes(module, res, addr)
            .max(module.glwe_blind_rotation_tmp_bytes(res, addr))
    }

    fn retrieve_stateless<R, A, BE: PhantomBackend>(
        &mut self,
        module: &Module<BE>,
        res: &mut R,
        address: &A,
        offset: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GetGGSWBit<BE> + 'static,
    {
        self.retriever.retrieve(
            module,
            res,
            &self.data,
            address,
            offset + module.log_n(),
            scratch,
        );

        module.glwe_blind_rotation_assign(
            res,
            address,
            false,
            offset,
            module.log_n().min(self.bit_size),
            0,
            scratch,
        );
    }

    fn retrieve_statefull_tmp_bytes<R, A, BE: PhantomBackend>(
        module: &Module<BE>,
        bit_size: usize,
        res: &R,
        addr: &A,
    ) -> usize
    where
        Module<BE>: PhantomModule<BE>,
        R: GLWEInfos,
        A: GGSWInfos,
    {
        // The blind rotation over the low address bits always runs; the
        // conditional swaps only when the memory spans several polynomials.
        let rotation: usize = module.glwe_blind_rotation_tmp_bytes(res, addr);
        let retrieval: usize = if bit_size > module.log_n() {
            module.glwe_blind_retrieval_tmp_bytes(res, addr)
        } else {
            0
        };

        rotation.max(retrieval)
    }

    fn retrieve_statefull<R, A, BE: PhantomBackend>(
        &mut self,
        module: &Module<BE>,
        res: &mut R,
        address: &A,
        offset: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GetGGSWBit<BE> + 'static,
    {
        if self.bit_size > module.log_n() {
            module.glwe_blind_retrieval_statefull(
                &mut self.data,
                address,
                offset + module.log_n(),
                self.bit_size - module.log_n(),
                scratch,
            );
        }
        module.glwe_blind_rotation_assign(
            &mut self.data[0],
            address,
            false,
            offset,
            module.log_n().min(self.bit_size),
            0,
            scratch,
        );
        module.glwe_copy(res, &self.data[0]);
    }

    fn retrieve_statefull_rev_tmp_bytes<R, A, K, BE: PhantomBackend>(
        module: &Module<BE>,
        bit_size: usize,
        res: &R,
        addr: &A,
        key: &K,
    ) -> usize
    where
        Module<BE>: PhantomModule<BE>,
        R: GLWEInfos,
        A: GGSWInfos,
        K: GGLWEInfos,
    {
        let trace: usize = module.glwe_trace_tmp_bytes(res, res, key)
            + module
                .glwe_bytes_of_from_infos(res)
                .max(module.glwe_normalize_tmp_bytes());
        // The blind rotation over the low address bits always runs; the
        // conditional swaps only when the memory spans several polynomials.
        let rotation: usize = module.glwe_blind_rotation_tmp_bytes(res, addr);
        let retrieval: usize = if bit_size > module.log_n() {
            module.glwe_blind_retrieval_tmp_bytes(res, addr)
        } else {
            0
        };

        trace.max(rotation).max(retrieval)
    }

    fn retrieve_statefull_rev<R, A, H, BE: PhantomBackend>(
        &mut self,
        module: &Module<BE>,
        res: &R,
        address: &A,
        keys: &H,
        offset: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        R: GLWEToBackendMut<BE>,
        A: GetGGSWBit<BE> + 'static,
        H: GetAutomorphismKey<BE>,
    {
        {
            let (mut tmp, mut scratch_1) = scratch.borrow().take_glwe_scratch(&self.data[0]);
            module.glwe_trace(&mut tmp, 0, &self.data[0], keys, &mut scratch_1);
            module.glwe_sub_assign(&mut self.data[0], &tmp);
        }

        module.glwe_add_assign(&mut self.data[0], res);
        module.glwe_normalize_assign(&mut self.data[0], scratch);

        module.glwe_blind_rotation_assign(
            &mut self.data[0],
            address,
            true,
            offset,
            module.log_n().min(self.bit_size),
            0,
            scratch,
        );
        if self.bit_size > module.log_n() {
            module.glwe_blind_retrieval_statefull_rev(
                &mut self.data,
                address,
                offset + module.log_n(),
                self.bit_size - module.log_n(),
                scratch,
            );
        }
    }

    fn write_tmp_bytes<R, A, K, BE: PhantomBackend>(
        module: &Module<BE>,
        bit_size: usize,
        res: &R,
        addr: &A,
        key: &K,
    ) -> usize
    where
        Module<BE>: PhantomModule<BE>,
        R: GLWEInfos,
        A: GGSWInfos,
        K: GGLWEInfos,
    {
        Self::retrieve_statefull_rev_tmp_bytes(module, bit_size, res, addr, key)
    }

    fn write<R, A, H, BE: PhantomBackend>(
        &mut self,
        module: &Module<BE>,
        res: &R,
        address: &A,
        keys: &H,
        offset: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        Module<BE>: PhantomModule<BE>,
        R: GLWEToBackendMut<BE>,
        A: GetGGSWBit<BE> + 'static,
        H: GetAutomorphismKey<BE>,
    {
        if self.bit_size > module.log_n() {
            module.glwe_blind_retrieval_statefull(
                &mut self.data,
                address,
                offset + module.log_n(),
                self.bit_size - module.log_n(),
                scratch,
            );
        }
        module.glwe_blind_rotation_assign(
            &mut self.data[0],
            address,
            false,
            offset,
            module.log_n().min(self.bit_size),
            0,
            scratch,
        );

        {
            let (mut tmp, mut scratch_1) = scratch.borrow().take_glwe_scratch(&self.data[0]);
            module.glwe_trace(&mut tmp, 0, &self.data[0], keys, &mut scratch_1);
            module.glwe_sub_assign(&mut self.data[0], &tmp);
        }

        module.glwe_add_assign(&mut self.data[0], res);
        module.glwe_normalize_assign(&mut self.data[0], scratch);

        module.glwe_blind_rotation_assign(
            &mut self.data[0],
            address,
            true,
            offset,
            module.log_n().min(self.bit_size),
            0,
            scratch,
        );

        if self.bit_size > module.log_n() {
            module.glwe_blind_retrieval_statefull_rev(
                &mut self.data,
                address,
                offset + module.log_n(),
                self.bit_size - module.log_n(),
                scratch,
            );
        }
    }
}
