use std::{f64, thread};

use poulpy_core::{
    layouts::{
        GGLWEInfos, GGLWELayout, GGLWEPreparedToRef, GGSWInfos, GLWEAutomorphismKeyHelper,
        GLWEInfos, GLWELayout, GLWESecretPreparedToRef, GLWEToMut, GetGaloisElement,
        TorusPrecision, GLWE,
    },
    GLWEAdd, GLWECopy, GLWEDecrypt, GLWEEncryptSk, GLWENoise, GLWENormalize, GLWEPacking,
    GLWERotate, GLWESub, GLWETrace, ScratchTakeCore,
};
use poulpy_hal::{
    api::{ModuleLogN, ModuleN, ScratchAvailable, TakeSlice},
    layouts::{Backend, DataMut, DataRef, Scratch},
    source::Source,
};
use poulpy_schemes::bin_fhe::bdd_arithmetic::{
    Cmux, FheUint, FheUintPrepared, FromBits, GLWEBlindRetrieval, GLWEBlindRetriever,
    GLWEBlindRotation, GetGGSWBit, ToBits,
};

pub struct Memory {
    bits: Vec<BitArray>,
    size: usize,
    bit_size: usize,
    state: bool,
}

impl Memory {
    pub(crate) fn alloc<A>(infos: &A, word_size: usize, size: usize) -> Self
    where
        A: GLWEInfos,
    {
        Self {
            bits: (0..word_size)
                .map(|_| BitArray::alloc(infos, size))
                .collect(),
            size,
            bit_size: (usize::BITS - (size - 1).leading_zeros()) as usize,
            state: false,
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn encrypt_sk<M, S, BE: Backend>(
        &mut self,
        module: &M,
        data: &[u32],
        sk: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleN + GLWEEncryptSk<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        S: GLWESecretPreparedToRef<BE>,
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
    pub(crate) fn decrypt<M, S, BE: Backend>(
        &self,
        module: &M,
        data: &mut [u32],
        sk: &S,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleN + GLWEDecrypt<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        S: GLWESecretPreparedToRef<BE>,
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

    pub(crate) fn noise<M, S, BE: Backend>(
        &self,
        module: &M,
        data: &[u32],
        sk: &S,
        scratch: &mut Scratch<BE>,
    ) -> Vec<f64>
    where
        M: ModuleN + GLWEDecrypt<BE> + GLWENoise<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        S: GLWESecretPreparedToRef<BE>,
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

    pub(crate) fn zero<M, BE: Backend, K, H>(
        &mut self,
        threads: usize,
        module: &M,
        addr: usize,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        M: Sync + ModuleN + GLWETrace<BE> + GLWESub + GLWERotate<BE>,
        H: Sync + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let poly: usize = addr / module.n();
        let idx: usize = poly % module.n();

        let glwe_infos: GLWELayout = self.bits[0].data[0].glwe_layout();
        let key_infos: GGLWELayout = keys.automorphism_key_infos();

        let scratch_thread_size: usize = module
            .glwe_rotate_tmp_bytes()
            .max(module.glwe_trace_tmp_bytes(&glwe_infos, &glwe_infos, &key_infos))
            + GLWE::bytes_of_from_infos(&glwe_infos);

        assert!(
            scratch.available() >= threads * scratch_thread_size,
            "scratch.available(): {} < threads:{threads} * scratch_thread_size: {scratch_thread_size}",
            scratch.available()
        );

        let (mut scratches, _) = scratch.split_mut(threads, scratch_thread_size);

        let chunk_size: usize = self.bits.len().div_ceil(threads);

        thread::scope(|scope| {
            for (scratch_thread, subram_chunk) in
                scratches.iter_mut().zip(self.bits.chunks_mut(chunk_size))
            {
                scope.spawn(move || {
                    let (mut tmp, scratch_1) = scratch_thread.take_glwe(&glwe_infos);

                    for subram in subram_chunk.iter_mut() {
                        let a: &mut GLWE<Vec<u8>> = &mut subram.data[poly];
                        module.glwe_rotate_inplace(-(idx as i64), a, scratch_1);
                        module.glwe_trace(&mut tmp, 0, a, keys, scratch_1);
                        module.glwe_sub_inplace(a, &tmp);
                        module.glwe_rotate_inplace(idx as i64, a, scratch_1);
                    }
                });
            }
        });
    }

    pub(crate) fn read_stateless<DR: DataMut, D: DataRef, H, M, K, BE: Backend>(
        &mut self,
        threads: usize,
        module: &M,
        res: &mut FheUint<DR, u32>,
        address: &FheUintPrepared<D, u32, BE>,
        offset: usize,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        M: Sync + ModuleLogN + GLWEPacking<BE> + GLWEBlindRotation<BE>,
        H: Sync + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert!(
            !self.bits.is_empty(),
            "unitialized memory: self.data.len()=0"
        );

        assert_eq!(self.state, false);

        let (mut tmp_res, scratch_1) = scratch.take_glwe_slice(self.bits.len(), res);

        let scratch_thread_size =
            BitArray::retrieve_stateless_tmp_bytes(module, &self.bits[0].data[0], address);

        assert!(
            scratch_1.available() >= threads * scratch_thread_size,
            "scratch.available(): {} < threads:{threads} * scratch_thread_size: {scratch_thread_size}",
            scratch_1.available()
        );

        let (mut scratches, _) = scratch_1.split_mut(threads, scratch_thread_size);

        let chunk_size: usize = self.bits.len().div_ceil(threads);

        thread::scope(|scope| {
            for ((scratch_thread, subram_chunk), tmp_chunk) in scratches
                .iter_mut()
                .zip(self.bits.chunks_mut(chunk_size))
                .zip(tmp_res.chunks_mut(chunk_size))
            {
                scope.spawn(move || {
                    for (subram, res) in subram_chunk.iter_mut().zip(tmp_chunk.iter_mut()) {
                        subram.retrieve_stateless(module, res, address, offset, scratch_thread);
                    }
                });
            }
        });

        res.pack(module, tmp_res, keys, scratch_1);
    }

    pub(crate) fn read_statefull<DR: DataMut, A, H, M, K, BE: Backend>(
        &mut self,
        threads: usize,
        module: &M,
        res: &mut FheUint<DR, u32>,
        address: &A,
        offset: usize,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        M: Sync + ModuleLogN + GLWEBlindRetrieval<BE> + GLWEBlindRotation<BE> + GLWEPacking<BE>,
        A: Sync + GetGGSWBit<BE> + GGSWInfos,
        H: Sync + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert!(
            !self.bits.is_empty(),
            "unitialized memory: self.data.len()=0"
        );

        assert_eq!(self.state, false);

        let (mut tmp_res, scratch_1) = scratch.take_glwe_slice(self.bits.len(), res);

        let scratch_thread_size = BitArray::retrieve_statefull_tmp_bytes(
            module,
            self.bit_size,
            &self.bits[0].data[0],
            address,
        );

        assert!(
            scratch_1.available() >= threads * scratch_thread_size,
            "scratch.available(): {} < threads:{threads} * scratch_thread_size: {scratch_thread_size}",
            scratch_1.available()
        );

        let (mut scratches, _) = scratch_1.split_mut(threads, scratch_thread_size);

        let chunk_size: usize = self.bits.len().div_ceil(threads);

        thread::scope(|scope| {
            for ((scratch_thread, subram_chunk), tmp_chunk) in scratches
                .iter_mut()
                .zip(self.bits.chunks_mut(chunk_size))
                .zip(tmp_res.chunks_mut(chunk_size))
            {
                scope.spawn(move || {
                    for (subram, res) in subram_chunk.iter_mut().zip(tmp_chunk.iter_mut()) {
                        subram.retrieve_statefull(module, res, address, offset, scratch_thread);
                    }
                });
            }
        });

        res.pack(module, tmp_res, keys, scratch_1);

        self.state = true;
    }

    pub(crate) fn read_statefull_rev<M, D, A, K, H, BE: Backend>(
        &mut self,
        threads: usize,
        module: &M,
        w: &FheUint<D, u32>, // Must encrypt [w, 0, 0, ..., 0];
        address: &A,
        offset: usize,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        D: DataRef,
        A: GetGGSWBit<BE> + GGSWInfos,
        M: Sync
            + ModuleLogN
            + GLWEBlindRetrieval<BE>
            + GLWETrace<BE>
            + GLWEBlindRotation<BE>
            + GLWEAdd
            + GLWESub
            + GLWENormalize<BE>,
        H: Sync + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert_eq!(self.state, true);

        let scratch_thread_size = BitArray::retrieve_statefull_rev_tmp_bytes(
            module,
            self.bit_size,
            &self.bits[0].data[0],
            address,
            &keys.automorphism_key_infos(),
        ) + GLWE::bytes_of_from_infos(w);

        assert!(
            scratch.available() >= threads * scratch_thread_size,
            "scratch.available(): {} < threads:{threads} * scratch_thread_size: {scratch_thread_size}",
            scratch.available()
        );

        let (mut scratches, _) = scratch.split_mut(threads, scratch_thread_size);

        let chunk_size: usize = self.bits.len().div_ceil(threads);

        thread::scope(|scope| {
            for (idx, (scratch_thread, subram_chunk)) in scratches
                .iter_mut()
                .zip(self.bits.chunks_mut(chunk_size))
                .enumerate()
            {
                scope.spawn(move || {
                    // Overwrites the coefficient that was read: to_write_on = to_write_on - TRACE(to_write_on) + w
                    for (i, subram) in subram_chunk.iter_mut().enumerate() {
                        let (mut bit, scratch_1) = scratch_thread.take_glwe(w);
                        w.get_bit_glwe(module, chunk_size * idx + i, &mut bit, keys, scratch_1);
                        subram
                            .retrieve_statefull_rev(module, &bit, address, keys, offset, scratch_1);
                    }
                });
            }
        });

        self.state = false;
    }

    pub(crate) fn write<M, D, A, K, H, BE: Backend>(
        &mut self,
        threads: usize,
        module: &M,
        w: &FheUint<D, u32>, // Must encrypt [w, 0, 0, ..., 0];
        address: &A,
        offset: usize,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        D: DataRef,
        A: GetGGSWBit<BE> + GGSWInfos,
        M: Sync
            + ModuleLogN
            + GLWEBlindRetrieval<BE>
            + GLWETrace<BE>
            + GLWEBlindRotation<BE>
            + GLWEAdd
            + GLWESub
            + GLWENormalize<BE>,
        H: Sync + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert_eq!(self.state, false);

        let scratch_thread_size = BitArray::write_tmp_bytes(
            module,
            self.bit_size,
            &self.bits[0].data[0],
            address,
            &keys.automorphism_key_infos(),
        ) + GLWE::bytes_of_from_infos(w);

        assert!(
            scratch.available() >= threads * scratch_thread_size,
            "scratch.available(): {} < threads:{threads} * scratch_thread_size: {scratch_thread_size}",
            scratch.available()
        );

        let (mut scratches, _) = scratch.split_mut(threads, scratch_thread_size);

        let chunk_size: usize = self.bits.len().div_ceil(threads);

        thread::scope(|scope| {
            for (idx, (scratch_thread, subram_chunk)) in scratches
                .iter_mut()
                .zip(self.bits.chunks_mut(chunk_size))
                .enumerate()
            {
                scope.spawn(move || {
                    // Overwrites the coefficient that was read: to_write_on = to_write_on - TRACE(to_write_on) + w
                    for (i, subram) in subram_chunk.iter_mut().enumerate() {
                        let (mut bit, scratch_1) = scratch_thread.take_glwe(w);
                        w.get_bit_glwe(module, chunk_size * idx + i, &mut bit, keys, scratch_1);
                        subram.write(module, &bit, address, keys, offset, scratch_1);
                    }
                });
            }
        });
    }
}

struct BitArray {
    data: Vec<GLWE<Vec<u8>>>,
    retriever: GLWEBlindRetriever,
    bit_size: usize,
}

impl BitArray {
    fn alloc<A>(infos: &A, size: usize) -> Self
    where
        A: GLWEInfos,
    {
        let n: usize = infos.n().into();
        Self {
            data: (0..size.div_ceil(n))
                .map(|_| GLWE::alloc_from_infos(infos))
                .collect(),
            retriever: GLWEBlindRetriever::alloc(infos, size),
            bit_size: (usize::BITS - (size - 1).leading_zeros()) as usize,
        }
    }

    fn encrypt_sk<M, BE: Backend, S>(
        &mut self,
        module: &M,
        data: &[u8],
        sk_prepared: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleN + GLWEEncryptSk<BE>,
        S: GLWESecretPreparedToRef<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let (mut pt, scratch_1) = scratch.take_glwe_plaintext(&self.data[0]);
        let (data_i64, scratch_2) = scratch_1.take_slice(module.n());

        for (chunk, ct) in data.chunks(module.n()).zip(self.data.iter_mut()) {
            data_i64.fill(0);

            for (y, x) in data_i64.iter_mut().zip(chunk.iter()) {
                *y = *x as i64
            }

            pt.encode_vec_i64(&data_i64, TorusPrecision(2));
            ct.encrypt_sk(module, &pt, sk_prepared, source_xa, source_xe, scratch_2);
        }
    }

    #[allow(dead_code)]
    fn decrypt<M, BE: Backend, S>(
        &self,
        module: &M,
        data: &mut [u8],
        sk_prepared: &S,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleN + GLWEDecrypt<BE>,
        S: GLWESecretPreparedToRef<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let (mut pt, scratch_1) = scratch.take_glwe_plaintext(&self.data[0]);
        let (mut data_i64, scratch_2) = scratch_1.take_slice(module.n());

        for (chunk, ct) in data.chunks_mut(module.n()).zip(self.data.iter()) {
            ct.decrypt(module, &mut pt, sk_prepared, scratch_2);
            pt.decode_vec_i64(&mut data_i64, TorusPrecision(2));
            for (y, x) in data_i64.iter_mut().zip(chunk.iter_mut()) {
                *x = *y as u8;
            }
        }
    }

    #[allow(dead_code)]
    fn noise<M, BE: Backend, S>(
        &self,
        module: &M,
        data: &[u8],
        sk_prepared: &S,
        scratch: &mut Scratch<BE>,
    ) -> f64
    where
        M: ModuleN + GLWEDecrypt<BE> + GLWENoise<BE>,
        S: GLWESecretPreparedToRef<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let (mut pt, scratch_1) = scratch.take_glwe_plaintext(&self.data[0]);
        let (data_i64, scratch_2) = scratch_1.take_slice(module.n());

        let mut max_noise: f64 = f64::MIN;

        for (chunk, ct) in data.chunks(module.n()).zip(self.data.iter()) {
            for (y, x) in data_i64.iter_mut().zip(chunk.iter()) {
                *y = *x as i64;
            }
            pt.encode_vec_i64(&data_i64, TorusPrecision(2));
            max_noise = max_noise.max(
                ct.noise(module, &mut pt, sk_prepared, scratch_2)
                    .max()
                    .log2(),
            );
        }

        max_noise
    }

    fn retrieve_stateless_tmp_bytes<M, R, A, BE: Backend>(module: &M, res: &R, addr: &A) -> usize
    where
        M: Cmux<BE> + GLWEBlindRotation<BE>,
        R: GLWEInfos,
        A: GGSWInfos,
    {
        GLWEBlindRetriever::retrieve_tmp_bytes(module, res, addr)
            .max(module.glwe_blind_rotation_tmp_bytes(res, addr))
    }

    fn retrieve_stateless<R, A, M, BE: Backend>(
        &mut self,
        module: &M,
        res: &mut R,
        address: &A,
        offset: usize,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleLogN + GLWECopy + Cmux<BE> + GLWEBlindRotation<BE>,
        R: GLWEToMut + GLWEInfos,
        A: GetGGSWBit<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        self.retriever.retrieve(
            module,
            res,
            &self.data,
            address,
            offset + module.log_n(),
            scratch,
        );

        module.glwe_blind_rotation_inplace(
            res,
            address,
            false,
            offset,
            module.log_n().min(self.bit_size),
            0,
            scratch,
        );
    }

    fn retrieve_statefull_tmp_bytes<M, R, A, BE: Backend>(
        module: &M,
        bit_size: usize,
        res: &R,
        addr: &A,
    ) -> usize
    where
        M: ModuleLogN + GLWEBlindRetrieval<BE> + GLWEBlindRotation<BE>,
        R: GLWEInfos,
        A: GGSWInfos,
    {
        let a: usize = module.glwe_blind_retrieval_tmp_bytes(res, addr);
        let b: usize = if bit_size > module.log_n() {
            module.glwe_blind_rotation_tmp_bytes(res, addr)
        } else {
            0
        };

        a.max(b)
    }

    fn retrieve_statefull<R, A, M, BE: Backend>(
        &mut self,
        module: &M,
        res: &mut R,
        address: &A,
        offset: usize,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleLogN + GLWECopy + GLWEBlindRotation<BE> + GLWEBlindRetrieval<BE>,
        R: GLWEToMut,
        A: GetGGSWBit<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
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
        module.glwe_blind_rotation_inplace(
            &mut self.data[0],
            address,
            false,
            offset,
            module.log_n().min(self.bit_size),
            0,
            scratch,
        );
        module.glwe_copy(res, &mut self.data[0]);
    }

    fn retrieve_statefull_rev_tmp_bytes<M, R, A, K, BE: Backend>(
        module: &M,
        bit_size: usize,
        res: &R,
        addr: &A,
        key: &K,
    ) -> usize
    where
        M: ModuleLogN
            + GLWEBlindRetrieval<BE>
            + GLWETrace<BE>
            + GLWENormalize<BE>
            + GLWEBlindRotation<BE>,
        R: GLWEInfos,
        A: GGSWInfos,
        K: GGLWEInfos,
    {
        let a: usize = module.glwe_trace_tmp_bytes(res, res, key)
            + GLWE::bytes_of_from_infos(res).max(module.glwe_normalize_tmp_bytes());
        let b: usize = module.glwe_blind_retrieval_tmp_bytes(res, addr);
        let c: usize = if bit_size > module.log_n() {
            module.glwe_blind_rotation_tmp_bytes(res, addr)
        } else {
            0
        };

        a.max(b).max(c)
    }

    fn retrieve_statefull_rev<R, A, M, H, K, BE: Backend>(
        &mut self,
        module: &M,
        res: &R,
        address: &A,
        keys: &H,
        offset: usize,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleLogN
            + GLWECopy
            + GLWEBlindRotation<BE>
            + GLWEBlindRetrieval<BE>
            + GLWETrace<BE>
            + GLWESub
            + GLWEAdd,
        R: GLWEToMut,
        A: GetGGSWBit<BE>,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        {
            let (mut tmp, scratch_1) = scratch.take_glwe(&self.data[0]);
            module.glwe_trace(&mut tmp, 0, &mut self.data[0], keys, scratch_1);
            module.glwe_sub_inplace(&mut self.data[0], &tmp);
        }

        module.glwe_add_inplace(&mut self.data[0], res);
        module.glwe_normalize_inplace(&mut self.data[0], scratch);

        module.glwe_blind_rotation_inplace(
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

    fn write_tmp_bytes<M, R, A, K, BE: Backend>(
        module: &M,
        bit_size: usize,
        res: &R,
        addr: &A,
        key: &K,
    ) -> usize
    where
        M: ModuleLogN
            + GLWEBlindRetrieval<BE>
            + GLWETrace<BE>
            + GLWENormalize<BE>
            + GLWEBlindRotation<BE>,
        R: GLWEInfos,
        A: GGSWInfos,
        K: GGLWEInfos,
    {
        Self::retrieve_statefull_rev_tmp_bytes(module, bit_size, res, addr, key)
    }

    fn write<R, A, M, H, K, BE: Backend>(
        &mut self,
        module: &M,
        res: &R,
        address: &A,
        keys: &H,
        offset: usize,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleLogN
            + GLWECopy
            + GLWEBlindRotation<BE>
            + GLWEBlindRetrieval<BE>
            + GLWETrace<BE>
            + GLWESub
            + GLWEAdd,
        R: GLWEToMut,
        A: GetGGSWBit<BE>,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE>,
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
        module.glwe_blind_rotation_inplace(
            &mut self.data[0],
            address,
            false,
            offset,
            module.log_n().min(self.bit_size),
            0,
            scratch,
        );

        {
            let (mut tmp, scratch_1) = scratch.take_glwe(&self.data[0]);
            module.glwe_trace(&mut tmp, 0, &mut self.data[0], keys, scratch_1);
            module.glwe_sub_inplace(&mut self.data[0], &tmp);
        }

        module.glwe_add_inplace(&mut self.data[0], res);
        module.glwe_normalize_inplace(&mut self.data[0], scratch);

        module.glwe_blind_rotation_inplace(
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
