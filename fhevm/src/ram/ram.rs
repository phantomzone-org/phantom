use itertools::izip;
use poulpy_core::{
    layouts::{
        GGLWEInfos, GGLWELayout, GGLWEPreparedToRef, GGSWLayout, GGSWPreparedFactory,
        GLWEAutomorphismKeyHelper, GLWEInfos, GLWELayout, GLWESecretPreparedFactory,
        GLWESecretPreparedToRef, GetGaloisElement, TorusPrecision, GLWE,
    },
    GGSWAutomorphism, GLWEAdd, GLWECopy, GLWEEncryptSk, GLWEExternalProduct, GLWENormalize,
    GLWEPacker, GLWEPackerOps, GLWEPacking, GLWERotate, GLWESub, GLWETrace, ScratchTakeCore,
};
use poulpy_hal::{
    api::{ModuleN, ScratchOwnedBorrow, TakeSlice},
    layouts::{Backend, DataMut, DataRef, Module, Scratch, ScratchOwned},
    source::Source,
};
use poulpy_schemes::tfhe::bdd_arithmetic::{FheUint, ToBits, UnsignedInteger};

use crate::{
    get_base_2d, keys::RAMKeysHelper, parameters::CryptographicParameters, reverse_bits_msb,
    Address, Base2D, Coordinate, CoordinatePrepared, TakeCoordinatePrepared,
};

/// [Ram] core implementation of the FHE-RAM.
pub struct Ram {
    subrams: Vec<SubRam>,
    max_addr: usize,
    word_size: usize,
    base_2d: Base2D,
}

impl Ram {
    /// Instantiates a new [Ram].
    pub fn new<BE: Backend>(
        params: &CryptographicParameters<BE>,
        word_size: usize,
        decomp_n: &Vec<u8>,
        max_addr: usize,
    ) -> Self where {
        assert!(word_size <= u32::BITS as usize);

        Self {
            subrams: (0..word_size)
                .map(|_| SubRam::alloc(params, max_addr))
                .collect(),
            word_size,
            base_2d: get_base_2d(max_addr as u32, decomp_n),
            max_addr,
        }
    }

    pub fn max_addr(&self) -> usize {
        self.max_addr
    }

    pub fn word_size(&self) -> usize {
        self.word_size
    }

    pub fn glwe_infos(&self) -> GLWELayout {
        self.subrams[0].data[0].glwe_layout()
    }

    pub fn base_2d(&self) -> &Base2D {
        &self.base_2d
    }

    pub fn subram(&self, i: usize) -> &SubRam {
        &self.subrams[i]
    }

    /// Scratch space size required by the [Ram].
    pub fn scratch_bytes<BE: Backend>(&self, params: &CryptographicParameters<BE>) -> usize
    where
        Module<BE>: GLWEPackerOps<BE>
            + GLWEEncryptSk<BE>
            + GGSWPreparedFactory<BE>
            + GGSWAutomorphism<BE>
            + GLWEExternalProduct<BE>
            + GLWETrace<BE>,
    {
        let module: &Module<BE> = params.module();
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
            CoordinatePrepared::alloc_bytes(module, &ggsw_infos, self.base_2d.max_len());
        let prepare_inv: usize = Coordinate::prepare_inv_scratch_space(params);
        let write_first_step: usize = ct + trace;
        let write_mit_step: usize = coordinate_product.max(ct + trace);
        let write_end_step: usize = coordinate_product;
        let write: usize =
            write_first_step.max(inv_addr + (prepare_inv.max(write_mit_step).max(write_end_step)));

        enc_sk.max(read).max(write)
    }

    /// Initialize the FHE-[Ram] with provided values (encrypted inder the provided secret).
    pub fn encrypt_sk<M, S, BE: Backend>(
        &mut self,
        module: &M,
        data: &[u32],
        sk: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleN + GLWESecretPreparedFactory<BE> + GLWEEncryptSk<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        S: GLWESecretPreparedToRef<BE>,
    {
        let max_addr: usize = self.max_addr;
        let ram_chunks: usize = self.word_size;

        assert!(data.len() / ram_chunks <= max_addr);

        let mut bits: Vec<u8> = vec![0u8; max_addr];

        for i in 0..ram_chunks {
            for (x, y) in bits.iter_mut().zip(data.iter()) {
                *x = y.bit(i);
            }
            self.subrams[i].encrypt_sk(module, &bits, sk, source_xa, source_xe, scratch);
        }
    }

    /// Simple read from the [Ram] at the provided encrypted address.
    /// Returns a vector of [GLWE], where each ciphertext stores
    /// Enc(m_i) where is the i-th digit of the word-size such that m = m_0 | m-1 | ...
    pub fn read<DA: DataRef, M, H, K, BE: Backend>(
        &mut self,
        module: &M,
        address: &Address<DA>,
        auto_keys: &H,
        scratch: &mut Scratch<BE>,
    ) -> Vec<GLWE<Vec<u8>>>
    where
        M: GGSWPreparedFactory<BE> + GLWEExternalProduct<BE> + GLWEPackerOps<BE> + GLWETrace<BE>,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert!(
            !self.subrams.is_empty(),
            "unitialized memory: self.data.len()=0"
        );

        self.subrams
            .iter_mut()
            .map(|subram| subram.read(module, address, auto_keys, scratch))
            .collect()
    }

    pub fn read_to_fheuint<
        DR: DataMut,
        DA: DataRef,
        D: DataRef,
        H,
        M,
        T: UnsignedInteger,
        BE: Backend,
    >(
        &mut self,
        module: &M,
        res: &mut FheUint<DR, T>,
        address: &Address<DA>,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        M: GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>,
        H: RAMKeysHelper<D, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        res.pack(
            module,
            self.read(module, &address, keys, scratch),
            keys,
            scratch,
        );
    }

    pub fn read_prepare_write_to_fheuint<
        DR: DataMut,
        DA: DataRef,
        K,
        H,
        M,
        D,
        T: UnsignedInteger,
        BE: Backend,
    >(
        &mut self,
        module: &M,
        res: &mut FheUint<DR, T>,
        address: &Address<DA>,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        M: GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>,
        H: RAMKeysHelper<D, BE>,
        D: DataRef,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        res.pack(
            module,
            self.read_prepare_write(module, &address, keys, scratch),
            keys,
            scratch,
        );
    }

    /// Read that prepares the [Ram] of a subsequent [Self::write].
    /// Outside of preparing the [Ram] for a write, the Bhavior and
    /// output format is identical to [Self::read].
    pub fn read_prepare_write<DA: DataRef, D, M, H, BE: Backend>(
        &mut self,
        module: &M,
        address: &Address<DA>,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) -> Vec<GLWE<Vec<u8>>>
    where
        M: GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWECopy
            + GLWEPackerOps<BE>
            + GLWETrace<BE>,
        H: RAMKeysHelper<D, BE>,
        D: DataRef,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert!(
            !self.subrams.is_empty(),
            "unitialized memory: self.data.len()=0"
        );

        self.subrams
            .iter_mut()
            .map(|subram| subram.read_prepare_write(module, address, keys, scratch))
            .collect()
    }

    /// Writes w to the [Ram]. Requires that [Self::read_prepare_write] was
    /// called Bforehand.
    pub fn write<D: DataRef, DA: DataRef, DH, H, BE: Backend>(
        &mut self,
        module: &Module<BE>,
        w: &[GLWE<D>], // Must encrypt [w, 0, 0, ..., 0];
        address: &Address<DA>,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        DH: DataRef,
        Module<BE>: GGSWPreparedFactory<BE>
            + GGSWAutomorphism<BE>
            + GLWENormalize<BE>
            + GLWEAdd
            + GLWESub
            + GLWETrace<BE>
            + GLWERotate<BE>
            + GLWEExternalProduct<BE>,
        ScratchOwned<BE>: ScratchOwnedBorrow<BE>,
        H: RAMKeysHelper<DH, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert!(w.len() == self.subrams.len());

        //let atk_ggsw_inv: &GLWEAutomorphismKeyPrepared<K, BE> = &keys.atk_ggsw_inv;
        //let tsk_ggsw_inv: &GGLWEToGGSWKeyPrepared<K, BE> = &keys.tsk_ggsw_inv;

        // Overwrites the coefficient that was read: to_write_on = to_write_on - TRACE(to_write_on) + w
        for (i, subram) in self.subrams.iter_mut().enumerate() {
            subram.write_first_step(module, &w[i], address.n2(), keys, scratch);
        }

        for i in (0..address.n2() - 1).rev() {
            // Index polynomial X^{i}
            let coordinate: &Coordinate<DA> = address.at(i + 1);

            let (mut inv_coordinate_prepared, scratch_1) =
                scratch.take_coordinate_prepared(module, coordinate, &coordinate.base1d);

            inv_coordinate_prepared.prepare_inv(
                module,
                coordinate,
                keys.get_ggsw_inv_key(),
                keys.get_gglwe_to_ggsw_key(),
                scratch_1,
            );

            for subram in self.subrams.iter_mut() {
                subram.write_mid_step(i, module, &inv_coordinate_prepared, keys, scratch_1);
            }
        }

        let coordinate: &Coordinate<DA> = address.at(0);

        let (mut inv_coordinate_prepared, scratch_1) =
            scratch.take_coordinate_prepared(module, coordinate, &coordinate.base1d);

        inv_coordinate_prepared.prepare_inv(
            module,
            coordinate,
            keys.get_ggsw_inv_key(),
            keys.get_gglwe_to_ggsw_key(),
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
    k: TorusPrecision,
}

impl SubRam {
    pub fn alloc<BE: Backend>(params: &CryptographicParameters<BE>, max_addr: usize) -> Self {
        let module: &Module<BE> = params.module();

        let glwe_infos: GLWELayout = params.glwe_ct_infos();

        let n: usize = module.n();
        let mut tree: Vec<Vec<GLWE<Vec<u8>>>> = Vec::new();

        if max_addr > n {
            let mut size: usize = max_addr.div_ceil(n);
            while size != 1 {
                size = size.div_ceil(n);
                let tmp: Vec<GLWE<Vec<u8>>> = (0..size)
                    .map(|_| GLWE::alloc_from_infos(&glwe_infos))
                    .collect();
                tree.push(tmp);
            }
        }

        Self {
            data: (0..max_addr.div_ceil(module.n()))
                .map(|_| GLWE::alloc_from_infos(&glwe_infos))
                .collect(),
            tree,
            packer: GLWEPacker::alloc(&glwe_infos, 0),
            state: false,
            k: params.k_glwe_pt(),
        }
    }

    pub fn data(&self) -> &[GLWE<Vec<u8>>] {
        &self.data
    }

    pub fn encrypt_sk<M, BE: Backend, S>(
        &mut self,
        module: &M,
        data: &[u8],
        sk_prepared: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleN + GLWESecretPreparedFactory<BE> + GLWEEncryptSk<BE>,
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

            pt.encode_vec_i64(&data_i64, self.k);
            ct.encrypt_sk(module, &pt, sk_prepared, source_xa, source_xe, scratch_2);
        }
    }

    fn read<M, DA: DataRef, K, H, BE: Backend>(
        &mut self,
        module: &M,
        address: &Address<DA>,
        auto_keys: &H,
        scratch: &mut Scratch<BE>,
    ) -> GLWE<Vec<u8>>
    where
        M: GGSWPreparedFactory<BE> + GLWEExternalProduct<BE> + GLWEPackerOps<BE> + GLWETrace<BE>,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE> + TakeCoordinatePrepared<BE>,
    {
        assert!(
            !self.state,
            "invalid call to Memory.read: internal state is true -> requires calling Memory.write"
        );

        let log_n: usize = module.log_n();

        let packer: &mut GLWEPacker = &mut self.packer;

        let mut results: Vec<GLWE<Vec<u8>>> = Vec::new();
        let mut tmp_ct: GLWE<Vec<u8>> = GLWE::alloc_from_infos(&self.data[0]);

        for i in 0..address.n2() {
            let coordinate: &Coordinate<DA> = address.at(i);

            let res_prev: &Vec<GLWE<Vec<u8>>> = if i == 0 { &self.data } else { &results };

            let (mut coordinate_prepared, scratch_1) =
                scratch.take_coordinate_prepared(module, address, &coordinate.base1d);

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

    fn read_prepare_write<M, DA: DataRef, H, K, BE: Backend>(
        &mut self,
        module: &M,
        address: &Address<DA>,
        auto_keys: &H,
        scratch: &mut Scratch<BE>,
    ) -> GLWE<Vec<u8>>
    where
        M: GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWECopy
            + GLWEPackerOps<BE>
            + GLWETrace<BE>,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE> + TakeCoordinatePrepared<BE>,
    {
        assert!(
            !self.state,
            "invalid call to Memory.read: internal state is true -> requires calling Memory.write"
        );

        let log_n: usize = module.log_n();
        let packer: &mut GLWEPacker = &mut self.packer;

        let mut results: Vec<GLWE<Vec<u8>>> = Vec::new();
        let mut tmp_ct: GLWE<Vec<u8>> = GLWE::alloc_from_infos(&self.data[0]);

        for i in 0..address.n2() {
            let coordinate: &Coordinate<DA> = address.at(i);

            let res_prev: &mut Vec<GLWE<Vec<u8>>> = if i == 0 {
                &mut self.data
            } else {
                &mut self.tree[i - 1]
            };

            let (mut coordinate_prepared, scratch_1) =
                scratch.take_coordinate_prepared(module, address, &coordinate.base1d);

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

        let mut res: GLWE<Vec<u8>> = GLWE::alloc_from_infos(&self.data[0]);

        self.state = true;
        if address.n2() != 1 {
            module.glwe_copy(&mut res, &self.tree.last().unwrap()[0]);
        } else {
            module.glwe_copy(&mut res, &self.data[0]);
        }

        res.trace_inplace(module, 0, log_n, auto_keys, scratch);
        res
    }

    fn write_first_step<DataW: DataRef, K, H, BE: Backend>(
        &mut self,
        module: &Module<BE>,
        w: &GLWE<DataW>,
        n2: usize,
        auto_keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        Module<BE>: GLWENormalize<BE> + GLWEAdd + GLWESub + GLWETrace<BE>,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert!(
            self.state,
            "invalid call to Memory.write: internal state is false -> requires calling Memory.read_prepare_write"
        );

        let log_n: usize = module.log_n();
        let (mut tmp_a, scratch_1) = scratch.take_glwe(&self.data[0]);

        let to_write_on: &mut GLWE<Vec<u8>> = if n2 != 1 {
            &mut self.tree.last_mut().unwrap()[0]
        } else {
            &mut self.data[0]
        };

        tmp_a.trace(module, 0, log_n, to_write_on, auto_keys, scratch_1);

        module.glwe_sub_inplace(to_write_on, &tmp_a);
        module.glwe_add_inplace(to_write_on, w);
        module.glwe_normalize_inplace(to_write_on, scratch_1);
    }

    fn write_mid_step<DC: DataRef, K, H, BE: Backend>(
        &mut self,
        step: usize,
        module: &Module<BE>,
        inv_coordinate: &CoordinatePrepared<DC, BE>,
        auto_keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        Module<BE>: GLWEExternalProduct<BE>
            + GLWESub
            + GLWETrace<BE>
            + GLWEAdd
            + GLWENormalize<BE>
            + GLWERotate<BE>,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let log_n: usize = module.log_n();
        let glwe_layout: &GLWELayout = &self.data[0].glwe_layout();

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
                let (mut tmp_a, scratch_1) = scratch.take_glwe(glwe_layout);
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

    fn write_last_step<DC: DataRef, BE: Backend>(
        &mut self,
        module: &Module<BE>,
        inv_coordinate: &CoordinatePrepared<DC, BE>,
        scratch: &mut Scratch<BE>,
    ) where
        Scratch<BE>: ScratchTakeCore<BE>,
        Module<BE>: GLWEExternalProduct<BE>,
    {
        // Apply the last reverse shift to the top of the tree.
        for ct_lo in self.data.iter_mut() {
            inv_coordinate.product_inplace(module, ct_lo, scratch);
        }

        self.state = false;
    }
}
