use std::collections::HashMap;

use itertools::izip;
use poulpy_core::{
    GGSWAutomorphism, GLWEAdd, GLWECopy, GLWEEncryptSk, GLWEExternalProduct, GLWENormalize,
    GLWEPacker, GLWEPackerOps, GLWERotate, GLWESub, GLWETrace, GetDistribution, ScratchTakeCore,
    layouts::{
        GGLWELayout, GGLWEToGGSWKeyPrepared, GGSWInfos, GGSWLayout, GGSWPreparedFactory, GLWE,
        GLWEAutomorphismKeyPrepared, GLWEInfos, GLWELayout, GLWESecret, GLWESecretPreparedFactory,
        GLWESecretToRef, LWEInfos,
    },
};
use poulpy_hal::{
    api::{ModuleNew, ScratchOwnedAlloc, ScratchOwnedBorrow, TakeSlice},
    layouts::{Backend, DataRef, Module, Scratch, ScratchOwned},
    source::Source,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{BDDKeyPrepared, FheUint, UnsignedInteger},
    blind_rotation::BlindRotationAlgo,
};

use crate::{
    Address, Coordinate, CoordinatePrepared, CryptographicParameters, TakeCoordinatePrepared, EvaluationKeysPrepared, Parameters, reverse_bits_msb
};

/// [Ram] core implementation of the FHE-RAM.
pub struct Ram<B: Backend> {
    pub params: Parameters<B>,
    pub subrams: Vec<SubRam>,
    pub scratch: ScratchOwned<B>,
}

impl<B: Backend> Default for Ram<B>
where
    Module<B>: ModuleNew<B>
        + GLWEEncryptSk<B>
        + GGSWPreparedFactory<B>
        + GGSWAutomorphism<B>
        + GLWEExternalProduct<B>
        + GLWEPackerOps<B>
        + GLWETrace<B>,
    ScratchOwned<B>: ScratchOwnedAlloc<B>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend> Ram<B>
where
    Module<B>: ModuleNew<B>
        + GLWEEncryptSk<B>
        + GGSWPreparedFactory<B>
        + GGSWAutomorphism<B>
        + GLWEExternalProduct<B>
        + GLWEPackerOps<B>
        + GLWETrace<B>,
    ScratchOwned<B>: ScratchOwnedAlloc<B>,
{
    /// Instantiates a new [Ram].
    pub fn new() -> Self {
        let params: Parameters<B> = Parameters::new();
        let scratch: ScratchOwned<B> = ScratchOwned::alloc(Self::scratch_bytes(&params));
        Self {
            subrams: (0..params.word_size())
                .map(|_| SubRam::alloc(&params))
                .collect(),
            params,
            scratch,
        }
    }

    /// Instantiates a new [Ram].
    pub fn new_from_ram_params(word_size: usize, decomp_n: Vec<u8>, max_addr: usize) -> Self {
        let params = Parameters {
            cryptographic_parameters: CryptographicParameters::new(),
            max_addr,
            decomp_n,
            word_size,
        };
        let scratch: ScratchOwned<B> = ScratchOwned::alloc(Self::scratch_bytes(&params));
        Self {
            subrams: (0..params.word_size())
                .map(|_| SubRam::alloc(&params))
                .collect(),
            params,
            scratch,
        }
    }
}

impl<B: Backend> Ram<B> {
    /// Scratch space size required by the [Ram].
    pub(crate) fn scratch_bytes(params: &Parameters<B>) -> usize
    where
        Module<B>: GLWEPackerOps<B>
            + GLWEEncryptSk<B>
            + GGSWPreparedFactory<B>
            + GGSWAutomorphism<B>
            + GLWEExternalProduct<B>
            + GLWETrace<B>,
    {
        let module: &Module<B> = params.module();
        let glwe_infos: GLWELayout = params.glwe_ct_infos();
        let ggsw_infos: GGSWLayout = params.ggsw_infos();
        let evk_glwe_infos: GGLWELayout = params.evk_glwe_infos();

        let enc_sk: usize = GLWE::encrypt_sk_tmp_bytes(module, &glwe_infos);

        // Read
        let coordinate_product: usize = Coordinate::product_scratch_space(params);
        let packing: usize = GLWEPacker::tmp_bytes(module, &glwe_infos, &evk_glwe_infos);
        let trace: usize = GLWE::trace_tmp_bytes(module, &glwe_infos, &glwe_infos, &evk_glwe_infos);
        let ct: usize = GLWE::bytes_of_from_infos(&glwe_infos);
        let read: usize = coordinate_product.max(trace).max(packing);

        // Write
        let inv_addr: usize =
            CoordinatePrepared::alloc_bytes(module, &ggsw_infos, params.base2d().max_len());
        let prepare_inv: usize = Coordinate::prepare_inv_scratch_space(params);
        let write_first_step: usize = ct + trace;
        let write_mit_step: usize = coordinate_product.max(ct + trace);
        let write_end_step: usize = coordinate_product;
        let write: usize =
            write_first_step.max(inv_addr + (prepare_inv.max(write_mit_step).max(write_end_step)));

        enc_sk.max(read).max(write)
    }

    /// Initialize the FHE-[Ram] with provided values (encrypted inder the provided secret).
    pub fn encrypt_sk(
        &mut self,
        data: &[u8],
        sk: &GLWESecret<Vec<u8>>,
        source_xa: &mut Source,
        source_xe: &mut Source,
    ) where
        Module<B>: GLWESecretPreparedFactory<B> + GLWEEncryptSk<B>,
        ScratchOwned<B>: ScratchOwnedBorrow<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        let params: &Parameters<B> = &self.params;
        let max_addr: usize = params.max_addr();
        let ram_chunks: usize = params.word_size();

        assert!(
            data.len().is_multiple_of(ram_chunks),
            "invalid data: data.len()%ram_chunks={} != 0",
            data.len().is_multiple_of(ram_chunks),
        );

        assert!(
            data.len() / ram_chunks == max_addr,
            "invalid data: data.len()/ram_chunks={} != max_addr={}",
            data.len() / ram_chunks,
            max_addr
        );

        let scratch: &mut Scratch<B> = self.scratch.borrow();

        let mut data_split: Vec<u8> = vec![0u8; max_addr];

        for i in 0..ram_chunks {
            for (j, x) in data_split.iter_mut().enumerate() {
                *x = data[j * ram_chunks + i];
            }
            self.subrams[i].encrypt_sk(params, &data_split, sk, source_xa, source_xe, scratch);
        }
    }

    /// Simple read from the [Ram] at the provided encrypted address.
    /// Returns a vector of [GLWE], where each ciphertext stores
    /// Enc(m_i) where is the i-th digit of the word-size such that m = m_0 | m-1 | ...
    pub fn read<D: DataRef, DA: DataRef>(
        &mut self,
        address: &Address<DA>,
        keys: &EvaluationKeysPrepared<D, B>,
    ) -> Vec<GLWE<Vec<u8>>>
    where
        Module<B>: GGSWPreparedFactory<B> + GLWEExternalProduct<B> + GLWEPackerOps<B>,
        ScratchOwned<B>: ScratchOwnedBorrow<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        assert!(
            !self.subrams.is_empty(),
            "unitialized memory: self.data.len()=0"
        );

        self.subrams
            .iter_mut()
            .map(|subram| subram.read(&self.params, address, &keys.atk_glwe, self.scratch.borrow()))
            .collect()
    }

    pub fn read_to_fheuint<D: DataRef, DA: DataRef, BRA: BlindRotationAlgo, T: UnsignedInteger>(
        &mut self,
        address: &Address<DA>,
        keys: &EvaluationKeysPrepared<D, B>,
        bdd_key_prepared: &BDDKeyPrepared<D, BRA, B>,
    ) -> FheUint<Vec<u8>, T>
    where
        Module<B>: GGSWPreparedFactory<B> + GLWEExternalProduct<B> + GLWEPackerOps<B>,
        ScratchOwned<B>: ScratchOwnedBorrow<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        let vec_glwe: Vec<GLWE<Vec<u8>>> = self.read(&address, keys);

        let mut vec_glwe_fheuint: FheUint<Vec<u8>, T> =
            FheUint::alloc_from_infos(&self.params.ggsw_infos());
        vec_glwe_fheuint.pack(
            self.params.module(),
            vec_glwe,
            bdd_key_prepared,
            self.scratch.borrow(),
        );

        vec_glwe_fheuint
    }

    /// Read that prepares the [Ram] of a subsequent [Self::write].
    /// Outside of preparing the [Ram] for a write, the Bhavior and
    /// output format is identical to [Self::read].
    pub fn read_prepare_write<D: DataRef, DA: DataRef>(
        &mut self,
        address: &Address<DA>,
        keys: &EvaluationKeysPrepared<D, B>,
    ) -> Vec<GLWE<Vec<u8>>>
    where
        Module<B>: GGSWPreparedFactory<B> + GLWEExternalProduct<B> + GLWECopy + GLWEPackerOps<B>,
        ScratchOwned<B>: ScratchOwnedBorrow<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        assert!(
            !self.subrams.is_empty(),
            "unitialized memory: self.data.len()=0"
        );

        self.subrams
            .iter_mut()
            .map(|subram| {
                subram.read_prepare_write(
                    &self.params,
                    address,
                    &keys.atk_glwe,
                    self.scratch.borrow(),
                )
            })
            .collect()
    }

    /// Writes w to the [Ram]. Requires that [Self::read_prepare_write] was
    /// called Bforehand.
    pub fn write<D: DataRef, DA: DataRef, K: DataRef>(
        &mut self,
        w: &[GLWE<D>], // Must encrypt [w, 0, 0, ..., 0];
        address: &Address<DA>,
        keys: &EvaluationKeysPrepared<K, B>,
    ) where
        Module<B>: GGSWPreparedFactory<B>
            + GGSWAutomorphism<B>
            + GLWENormalize<B>
            + GLWEAdd
            + GLWESub
            + GLWETrace<B>
            + GLWERotate<B>
            + GLWEExternalProduct<B>,
        ScratchOwned<B>: ScratchOwnedBorrow<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        assert!(w.len() == self.subrams.len());

        let params: &Parameters<B> = &self.params;
        let module: &Module<B> = params.module();

        let scratch: &mut Scratch<B> = self.scratch.borrow();
        let atk_glwe: &HashMap<i64, GLWEAutomorphismKeyPrepared<K, B>> = &keys.atk_glwe;
        let atk_ggsw_inv: &GLWEAutomorphismKeyPrepared<K, B> = &keys.atk_ggsw_inv;
        let tsk_ggsw_inv: &GGLWEToGGSWKeyPrepared<K, B> = &keys.tsk_ggsw_inv;

        // Overwrites the coefficient that was read: to_write_on = to_write_on - TRACE(to_write_on) + w
        for (i, subram) in self.subrams.iter_mut().enumerate() {
            subram.write_first_step(params, &w[i], address.n2(), atk_glwe, scratch);
        }

        for i in (0..address.n2() - 1).rev() {
            // Index polynomial X^{i}
            let coordinate: &Coordinate<DA> = address.at(i + 1);

            let (mut inv_coordinate_prepared, scratch_1) =
                scratch.take_coordinate_prepared(module, coordinate, &coordinate.base1d);

            inv_coordinate_prepared.prepare_inv(
                module,
                coordinate,
                atk_ggsw_inv,
                tsk_ggsw_inv,
                scratch_1,
            );

            for subram in self.subrams.iter_mut() {
                subram.write_mid_step(i, params, &inv_coordinate_prepared, atk_glwe, scratch_1);
            }
        }

        let coordinate: &Coordinate<DA> = address.at(0);

        let (mut inv_coordinate_prepared, scratch_1) =
            scratch.take_coordinate_prepared(module, coordinate, &coordinate.base1d);

        inv_coordinate_prepared.prepare_inv(
            module,
            coordinate,
            atk_ggsw_inv,
            tsk_ggsw_inv,
            scratch_1,
        );

        for subram in self.subrams.iter_mut() {
            subram.write_last_step(module, &inv_coordinate_prepared, scratch_1);
        }
    }
}

/// [SubRam] stores a digit of the word.
pub struct SubRam {
    data: Vec<GLWE<Vec<u8>>>,
    tree: Vec<Vec<GLWE<Vec<u8>>>>,
    packer: GLWEPacker,
    state: bool,
}

impl SubRam {
    pub fn alloc<B: Backend>(params: &Parameters<B>) -> Self {
        let module: &Module<B> = params.module();

        let glwe_infos: GLWELayout = params.glwe_ct_infos();

        let n: usize = module.n();
        let mut tree: Vec<Vec<GLWE<Vec<u8>>>> = Vec::new();
        let max_addr_split: usize = params.max_addr(); // u8 -> u32

        if max_addr_split > n {
            let mut size: usize = max_addr_split.div_ceil(n);
            while size != 1 {
                size = size.div_ceil(n);
                let tmp: Vec<GLWE<Vec<u8>>> = (0..size)
                    .map(|_| GLWE::alloc_from_infos(&glwe_infos))
                    .collect();
                tree.push(tmp);
            }
        }

        Self {
            data: Vec::new(),
            tree,
            packer: GLWEPacker::alloc(&glwe_infos, 0),
            state: false,
        }
    }

    pub fn encrypt_sk<B: Backend, S>(
        &mut self,
        params: &Parameters<B>,
        data: &[u8],
        sk: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<B>,
    ) where
        Module<B>: GLWESecretPreparedFactory<B> + GLWEEncryptSk<B>,
        S: GLWESecretToRef + GetDistribution + GLWEInfos,
        Scratch<B>: ScratchTakeCore<B>,
    {
        let module: &Module<B> = params.module();

        let glwe_infos: GLWELayout = params.glwe_ct_infos();
        let pt_infos: GLWELayout = params.glwe_pt_infos();

        let (mut pt, scratch_1) = scratch.take_glwe_plaintext(&pt_infos);
        let (data_i64, scratch_2) = scratch_1.take_slice(module.n());
        let (mut sk_glwe_prepared, scratch_3) =
            scratch_2.take_glwe_secret_prepared(module, sk.rank());
        sk_glwe_prepared.prepare(module, sk);

        self.data = data
            .chunks(module.n())
            .map(|chunk| {
                let mut ct: GLWE<Vec<u8>> = GLWE::alloc_from_infos(&glwe_infos);

                for (x_i64, x_u8) in izip!(data_i64.iter_mut(), chunk.iter()) {
                    *x_i64 = (*x_u8 as i8) as i64
                }

                data_i64[chunk.len()..].iter_mut().for_each(|x| *x = 0);
                pt.encode_vec_i64(&data_i64, pt.k());
                ct.encrypt_sk(
                    module,
                    &pt,
                    &sk_glwe_prepared,
                    source_xa,
                    source_xe,
                    scratch_3,
                );
                ct
            })
            .collect();
    }

    fn read<K: DataRef, DA: DataRef, B: Backend>(
        &mut self,
        params: &Parameters<B>,
        address: &Address<DA>,
        auto_keys: &HashMap<i64, GLWEAutomorphismKeyPrepared<K, B>>,
        scratch: &mut Scratch<B>,
    ) -> GLWE<Vec<u8>>
    where
        Module<B>: GGSWPreparedFactory<B> + GLWEExternalProduct<B> + GLWEPackerOps<B>,
        Scratch<B>: ScratchTakeCore<B> + TakeCoordinatePrepared<B>,
    {
        assert!(
            !self.state,
            "invalid call to Memory.read: internal state is true -> requires calling Memory.write"
        );

        let module: &Module<B> = params.module();
        let log_n: usize = module.log_n();

        let glwe_infos: GLWELayout = params.glwe_ct_infos();
        let ggsw_infos: GGSWLayout = params.ggsw_infos();

        assert_eq!(ggsw_infos, address.ggsw_layout());

        let packer: &mut GLWEPacker = &mut self.packer;

        let mut results: Vec<GLWE<Vec<u8>>> = Vec::new();
        let mut tmp_ct: GLWE<Vec<u8>> = GLWE::alloc_from_infos(&glwe_infos);

        for i in 0..address.n2() {
            let coordinate: &Coordinate<DA> = address.at(i);

            let res_prev: &Vec<GLWE<Vec<u8>>> = if i == 0 { &self.data } else { &results };

            let (mut coordinate_prepared, scratch_1) =
                scratch.take_coordinate_prepared(module, &ggsw_infos, &coordinate.base1d);

            coordinate_prepared.prepare(module, coordinate, scratch_1);

            if i < address.n2() - 1 {
                // let mut result_next: Vec<GLWE<Vec<u8>>> = Vec::new();

                for chunk in res_prev.chunks(module.n()) {
                    for j in 0..module.n() {
                        let j_rev = reverse_bits_msb(j, log_n as u32);

                        if j_rev < chunk.len() {
                            coordinate_prepared.product(
                                module,
                                &mut tmp_ct,
                                &chunk[j_rev],
                                scratch_1,
                            );
                            packer.add(module, Some(&tmp_ct), auto_keys, scratch_1);
                        } else {
                            packer.add(
                                module,
                                // &mut result_next,
                                None::<&GLWE<Vec<u8>>>,
                                auto_keys,
                                scratch_1,
                            );
                        }
                    }
                }

                packer.flush(module, &mut tmp_ct); //, auto_keys, scratch); // TODO: that to put instead of tmp_ct
                results.push(tmp_ct.clone());
            } else if i == 0 {
                coordinate_prepared.product(module, &mut tmp_ct, &self.data[0], scratch_1);
                results.push(tmp_ct.clone());
            } else {
                coordinate_prepared.product(module, &mut tmp_ct, &results[0], scratch_1);
            }
        }
        tmp_ct.trace_inplace(module, 0, log_n, auto_keys, scratch);
        tmp_ct
    }

    fn read_prepare_write<D: DataRef, DA: DataRef, B: Backend>(
        &mut self,
        params: &Parameters<B>,
        address: &Address<DA>,
        auto_keys: &HashMap<i64, GLWEAutomorphismKeyPrepared<D, B>>,
        scratch: &mut Scratch<B>,
    ) -> GLWE<Vec<u8>>
    where
        Module<B>: GGSWPreparedFactory<B> + GLWEExternalProduct<B> + GLWECopy + GLWEPackerOps<B>,
        Scratch<B>: ScratchTakeCore<B> + TakeCoordinatePrepared<B>,
    {
        assert!(
            !self.state,
            "invalid call to Memory.read: internal state is true -> requires calling Memory.write"
        );

        let module: &Module<B> = params.module();
        let log_n: usize = module.log_n();
        let ggsw_infos: GGSWLayout = params.ggsw_infos();
        let packer: &mut GLWEPacker = &mut self.packer;

        assert_eq!(ggsw_infos, address.ggsw_layout());

        let mut results: Vec<GLWE<Vec<u8>>> = Vec::new();
        let mut tmp_ct: GLWE<Vec<u8>> = GLWE::alloc_from_infos(&params.glwe_ct_infos());

        for i in 0..address.n2() {
            let coordinate: &Coordinate<DA> = address.at(i);

            let res_prev: &mut Vec<GLWE<Vec<u8>>> = if i == 0 {
                &mut self.data
            } else {
                &mut self.tree[i - 1]
            };

            let (mut coordinate_prepared, scratch_1) =
                scratch.take_coordinate_prepared(module, &ggsw_infos, &coordinate.base1d);

            coordinate_prepared.prepare(module, coordinate, scratch_1);

            // Shift polynomial of the last iteration by X^{-i}
            for poly in res_prev.iter_mut() {
                coordinate_prepared.product_inplace(module, poly, scratch_1);
            }

            if i < address.n2() - 1 {
                // let mut result_next: Vec<GLWE<Vec<u8>>> = Vec::new();

                // Packs the first coefficient of each polynomial.
                for chunk in res_prev.chunks(module.n()) {
                    for i in 0..module.n() {
                        let i_rev: usize = reverse_bits_msb(i, log_n as u32);
                        if i_rev < chunk.len() {
                            packer.add(module, Some(&chunk[i_rev]), auto_keys, scratch);
                        } else {
                            packer.add(module, None::<&GLWE<Vec<u8>>>, auto_keys, scratch);
                        }
                    }
                }

                packer.flush(module, &mut tmp_ct);
                results.push(tmp_ct.clone());

                // Stores the packed polynomial
                izip!(self.tree[i].iter_mut(), results.iter()).for_each(|(a, b)| {
                    module.glwe_copy(a, b);
                });
            }
        }

        let mut res: GLWE<Vec<u8>> = GLWE::alloc_from_infos(&params.glwe_ct_infos());

        self.state = true;
        if address.n2() != 1 {
            module.glwe_copy(&mut res, &self.tree.last().unwrap()[0]);
        } else {
            module.glwe_copy(&mut res, &self.data[0]);
        }

        res.trace_inplace(module, 0, log_n, auto_keys, scratch);
        res
    }

    fn write_first_step<DataW: DataRef, K: DataRef, B: Backend>(
        &mut self,
        params: &Parameters<B>,
        w: &GLWE<DataW>,
        n2: usize,
        auto_keys: &HashMap<i64, GLWEAutomorphismKeyPrepared<K, B>>,
        scratch: &mut Scratch<B>,
    ) where
        Module<B>: GLWENormalize<B> + GLWEAdd + GLWESub + GLWETrace<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        assert!(
            self.state,
            "invalid call to Memory.write: internal state is false -> requires calling Memory.read_prepare_write"
        );

        let module: &Module<B> = params.module();
        let log_n: usize = module.log_n();

        let glwe_infos: GLWELayout = params.glwe_ct_infos();

        let to_write_on: &mut GLWE<Vec<u8>> = if n2 != 1 {
            &mut self.tree.last_mut().unwrap()[0]
        } else {
            &mut self.data[0]
        };

        let (mut tmp_a, scratch_1) = scratch.take_glwe(&glwe_infos);
        tmp_a.trace(module, 0, log_n, to_write_on, auto_keys, scratch_1);

        module.glwe_sub_inplace(to_write_on, &tmp_a);
        module.glwe_add_inplace(to_write_on, w);
        module.glwe_normalize_inplace(to_write_on, scratch_1);
    }

    fn write_mid_step<DC: DataRef, K: DataRef, B: Backend>(
        &mut self,
        step: usize,
        params: &Parameters<B>,
        inv_coordinate: &CoordinatePrepared<DC, B>,
        auto_keys: &HashMap<i64, GLWEAutomorphismKeyPrepared<K, B>>,
        scratch: &mut Scratch<B>,
    ) where
        Module<B>: GLWEExternalProduct<B>
            + GLWESub
            + GLWETrace<B>
            + GLWEAdd
            + GLWENormalize<B>
            + GLWERotate<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        let module: &Module<B> = params.module();
        let log_n: usize = module.log_n();

        // Top of the tree is not stored in results.
        let (tree_hi, tree_lo) = if step == 0 {
            (&mut self.data, &mut self.tree[0])
        } else {
            let (left, right) = self.tree.split_at_mut(step);
            (&mut left[left.len() - 1], &mut right[0])
        };

        for (j, chunk) in tree_hi.chunks_mut(module.n()).enumerate() {
            // Retrieve the associated polynomial to extract and pack related to the current chunk
            let ct_lo: &mut GLWE<Vec<u8>> = &mut tree_lo[j];

            inv_coordinate.product_inplace(module, ct_lo, scratch);

            for ct_hi in chunk.iter_mut() {
                // Zeroes the first coefficient of ct_hi
                // ct_hi = [a, b, c, d] - TRACE([a, b, c, d]) = [0, b, c, d]
                let (mut tmp_a, scratch_1) = scratch.take_glwe(&params.glwe_ct_infos());
                tmp_a.trace(module, 0, log_n, ct_hi, auto_keys, scratch_1);
                module.glwe_sub_inplace(ct_hi, &tmp_a);

                // Extract the first coefficient ct_lo
                // tmp_a = TRACE([a, b, c, d]) -> [a, 0, 0, 0]
                tmp_a.trace(module, 0, log_n, ct_lo, auto_keys, scratch_1);

                // Adds extracted coefficient of ct_lo on ct_hi
                // [a, 0, 0, 0] + [0, b, c, d]
                module.glwe_add_inplace(ct_hi, &tmp_a);
                module.glwe_normalize_inplace(ct_hi, scratch_1);

                // Cyclic shift ct_lo by X^-1
                module.glwe_rotate_inplace(-1, ct_lo, scratch_1);
            }
        }
    }

    fn write_last_step<DC: DataRef, B: Backend>(
        &mut self,
        module: &Module<B>,
        inv_coordinate: &CoordinatePrepared<DC, B>,
        scratch: &mut Scratch<B>,
    ) where
        Scratch<B>: ScratchTakeCore<B>,
        Module<B>: GLWEExternalProduct<B>,
    {
        // Apply the last reverse shift to the top of the tree.
        for ct_lo in self.data.iter_mut() {
            inv_coordinate.product_inplace(module, ct_lo, scratch);
        }

        self.state = false;
    }
}
