use poulpy_core::{
    GLWEAdd, GLWECopy, GLWERotate, GLWESub, GLWETrace,
    layouts::{
        GGLWEInfos, GGLWEPreparedToRef, GLWEAutomorphismKeyHelper, GLWEToMut, GetGaloisElement,
    },
};
use poulpy_hal::{
    api::{ModuleLogN, ModuleN},
    layouts::{Backend, DataMut, DataRef, Module, Scratch},
};
use poulpy_schemes::tfhe::bdd_arithmetic::{
    FheUint, FheUintPrepared, GLWEBlindRotation, ScratchTakeBDD, UnsignedInteger,
};

impl<T: UnsignedInteger, BE: Backend> VMSelectStore<T, BE> for Module<BE> where
    Self: Sized
        + ModuleLogN
        + ModuleN
        + GLWERotate<BE>
        + GLWETrace<BE>
        + GLWECopy
        + GLWESub
        + GLWEAdd
        + GLWEBlindRotation<T, BE>
{
}

pub trait VMSelectStore<T: UnsignedInteger, BE: Backend>
where
    Self: Sized
        + ModuleLogN
        + ModuleN
        + GLWERotate<BE>
        + GLWETrace<BE>
        + GLWECopy
        + GLWESub
        + GLWEAdd
        + GLWEBlindRotation<T, BE>,
{
    fn select_store<D0, D1, D2, D3, D4, K, H>(
        &self,
        res: &mut FheUint<D0, T>,
        rs2: &FheUint<D1, T>,    //x
        loaded: &FheUint<D2, T>, //y
        offset: &FheUintPrepared<D3, T, BE>,
        op: &FheUintPrepared<D4, T, BE>,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        D0: DataMut,
        D1: DataRef,
        D2: DataRef,
        D3: DataRef,
        D4: DataRef,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        T: UnsignedInteger,
        Scratch<BE>: ScratchTakeBDD<T, BE>,
    {
        self.construct_store_test_vector(res, rs2, loaded, keys, scratch);
        // res * X^{offset<<2}
        self.glwe_blind_rotation_inplace(res, offset, false, 0, 2, 2, scratch);
        // res * X^{op}
        self.glwe_blind_rotation_inplace(res, op, false, 0, 2, 0, scratch);
        // Clean other values
        self.glwe_trace_inplace(res, T::LOG_BITS as usize, self.log_n(), keys, scratch);
    }

    fn construct_store_test_vector<R, D0, D1, H, K>(
        &self,
        res: &mut R,
        rs2: &FheUint<D0, T>,    //x
        loaded: &FheUint<D1, T>, //y
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        R: GLWEToMut,
        D0: DataRef,
        D1: DataRef,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        T: UnsignedInteger,
        Scratch<BE>: ScratchTakeBDD<T, BE>,
    {
        let (mut tmp_fhe_uint, scratch_1) = scratch.take_fhe_uint(rs2);

        // offset = 0
        // NONE: [y00...y07][y08...y15][y16...y24][y25...y31]
        self.glwe_copy(res, loaded);
        self.glwe_rotate_inplace(-1, res, scratch_1);
        // SB:   [x00...x07][y08...y15][y16...y24][y25...y31]
        tmp_fhe_uint.splice_u8(self, 0, 0, loaded, rs2, keys, scratch_1);
        self.glwe_add_inplace(res, &tmp_fhe_uint);
        self.glwe_rotate_inplace(-1, res, scratch_1);
        // SH:   [x00...x07][x08...x15][y16...y24][y25...y31]
        tmp_fhe_uint.splice_u16(self, 0, 0, loaded, rs2, keys, scratch_1);
        self.glwe_add_inplace(res, &tmp_fhe_uint);
        self.glwe_rotate_inplace(-1, res, scratch_1);
        // SW:   [x00...x07][x08...x15][x16...x24][x25...x31]
        self.glwe_add_inplace(res, rs2);
        self.glwe_rotate_inplace(-1, res, scratch_1);

        // offset = 1
        // NONE: [y00...y07][y08...y15][y16...y24][y25...y31]
        self.glwe_add_inplace(res, loaded);
        self.glwe_rotate_inplace(-1, res, scratch_1);
        // SB:   [y00...y07][x00...x07][y16...y24][y25...y31]
        tmp_fhe_uint.splice_u8(self, 1, 0, loaded, rs2, keys, scratch_1);
        self.glwe_add_inplace(res, &tmp_fhe_uint);
        self.glwe_rotate_inplace(-1, res, scratch_1);
        // SH:   [ INVALID  ]
        self.glwe_rotate_inplace(-1, res, scratch_1);
        // SW:   [ INVALID  ]
        self.glwe_rotate_inplace(-1, res, scratch_1);

        // offset = 2
        // NONE: [y00...y07][y08...y15][y16...y24][y25...y31]
        self.glwe_add_inplace(res, loaded);
        self.glwe_rotate_inplace(-1, res, scratch_1);
        // SB:   [y00...y07][y08...y15][x00...x07][y25...y31]
        tmp_fhe_uint.splice_u8(self, 2, 0, loaded, rs2, keys, scratch_1);
        self.glwe_add_inplace(res, &tmp_fhe_uint);
        self.glwe_rotate_inplace(-1, res, scratch_1);
        // SH:   [y00...y07][y08...y15][x00...x07][x08...x15]
        tmp_fhe_uint.splice_u16(self, 1, 0, loaded, rs2, keys, scratch_1);
        self.glwe_add_inplace(res, &tmp_fhe_uint);
        self.glwe_rotate_inplace(-1, res, scratch_1);
        // SW:   [ INVALID  ]
        self.glwe_rotate_inplace(-1, res, scratch_1);

        // offset = 3
        // NONE: [y00...y07][y08...y15][y16...y24][y25...y31]
        self.glwe_add_inplace(res, loaded);
        self.glwe_rotate_inplace(-1, res, scratch_1);
        // SB:   [y00...y07][y08...y15][y16...y24][x00...x07]
        tmp_fhe_uint.splice_u8(self, 3, 0, loaded, rs2, keys, scratch_1);
        self.glwe_add_inplace(res, &tmp_fhe_uint);
        self.glwe_rotate_inplace(-1, res, scratch_1);
        // SH:   [ INVALID  ]
        // SW:   [ INVALID  ]

        self.glwe_rotate_inplace(14, res, scratch_1);
    }
}

#[test]
fn vm_select_store_fft64_ref() {
    use std::collections::HashMap;

    use poulpy_backend::FFT64Ref;
    use poulpy_core::{
        GLWEPacker, GLWERotate,
        layouts::{
            Dnum, Dsize, GGSWLayout, GLWEAutomorphismKey, GLWEAutomorphismKeyLayout,
            GLWEAutomorphismKeyPrepared, GLWELayout, GLWESecret, GLWESecretPrepared,
        },
    };
    use poulpy_hal::{
        api::{ModuleNew, ScratchOwnedAlloc, ScratchOwnedBorrow},
        layouts::{Module, ScratchOwned},
        source::Source,
    };
    use poulpy_schemes::tfhe::bdd_arithmetic::FheUint;

    let module: Module<FFT64Ref> = Module::<FFT64Ref>::new(512);

    let base2k: usize = 13;
    let rank: usize = 1;

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([1u8; 32]);
    let mut source_xe: Source = Source::new([2u8; 32]);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 20);

    let glwe_infos: GLWELayout = GLWELayout {
        n: module.n().into(),
        base2k: base2k.into(),
        k: (2 * base2k).into(),
        rank: rank.into(),
    };

    let atk_infos: GLWEAutomorphismKeyLayout = GLWEAutomorphismKeyLayout {
        n: module.n().into(),
        base2k: base2k.into(),
        k: (3 * base2k).into(),
        rank: rank.into(),
        dnum: Dnum(2),
        dsize: Dsize(1),
    };

    let ggsw_infos: GGSWLayout = GGSWLayout {
        n: module.n().into(),
        base2k: base2k.into(),
        k: (3 * base2k).into(),
        rank: rank.into(),
        dnum: Dnum(2),
        dsize: Dsize(1),
    };

    let mut sk: GLWESecret<Vec<u8>> = GLWESecret::alloc(module.n().into(), rank.into());
    sk.fill_ternary_prob(0.5, &mut source_xs);

    let mut sk_prep: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc(&module, rank.into());
    sk_prep.prepare(&module, &sk);

    let gal_els: Vec<i64> = GLWEPacker::galois_elements(&module);
    let mut auto_keys: HashMap<i64, GLWEAutomorphismKeyPrepared<Vec<u8>, FFT64Ref>> =
        HashMap::new();
    let mut tmp: GLWEAutomorphismKey<Vec<u8>> = GLWEAutomorphismKey::alloc_from_infos(&atk_infos);
    gal_els.iter().for_each(|gal_el| {
        tmp.encrypt_sk(
            &module,
            *gal_el,
            &sk,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );
        let mut atk_prepared: GLWEAutomorphismKeyPrepared<Vec<u8>, FFT64Ref> =
            GLWEAutomorphismKeyPrepared::alloc_from_infos(&module, &tmp);
        atk_prepared.prepare(&module, &tmp, scratch.borrow());
        auto_keys.insert(*gal_el, atk_prepared);
    });

    let mut a_enc: FheUint<Vec<u8>, u32> = FheUint::<Vec<u8>, u32>::alloc_from_infos(&glwe_infos);
    let mut b_enc: FheUint<Vec<u8>, u32> = FheUint::<Vec<u8>, u32>::alloc_from_infos(&glwe_infos);
    let mut c_enc: FheUint<Vec<u8>, u32> = FheUint::<Vec<u8>, u32>::alloc_from_infos(&glwe_infos);

    let a: u32 = 0xFFFFFFFF;
    let b: u32 = 0xAABBCCDD;

    b_enc.encrypt_sk(
        &module,
        b,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );
    a_enc.encrypt_sk(
        &module,
        a,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    module.construct_store_test_vector(&mut c_enc, &a_enc, &b_enc, &auto_keys, scratch.borrow());

    let mut res_enc: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&glwe_infos);
    let mut rs2_enc: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&glwe_infos);
    let mut loaded_enc: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&glwe_infos);
    let mut offset_enc: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::alloc_from_infos(&module, &ggsw_infos);
    let mut op_enc: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::alloc_from_infos(&module, &ggsw_infos);

    rs2_enc.encrypt_sk(
        &module,
        a,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );
    loaded_enc.encrypt_sk(
        &module,
        b,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    for offset in 0..4 {
        offset_enc.encrypt_sk(
            &module,
            offset,
            &sk_prep,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );

        for op in 0..4 {
            op_enc.encrypt_sk(
                &module,
                op,
                &sk_prep,
                &mut source_xa,
                &mut source_xe,
                scratch.borrow(),
            );

            let c_want = match op {
                0 => b,
                1 => ((b.rotate_right(offset << 3) & 0xFFFF_FF00) | (a & 0x0000_00FF))
                    .rotate_left(offset << 3),
                2 => match offset {
                    0 | 2 => ((b.rotate_right(offset << 3) & 0xFFFF_0000) | (a & 0x0000_FFFF))
                        .rotate_left(offset << 3),
                    _ => 0,
                },

                3 => match offset {
                    0 => a,
                    _ => 0,
                },
                _ => 0,
            };

            //println!("offset: {offset} op: {op}: {c_want:08x}, {:08x}", c_enc.decrypt(&module, &sk_prep, scratch.borrow()));
            assert_eq!(c_want, c_enc.decrypt(&module, &sk_prep, scratch.borrow()));
            module.glwe_rotate_inplace(-1, &mut c_enc, scratch.borrow());

            module.select_store(
                &mut res_enc,
                &rs2_enc,
                &loaded_enc,
                &offset_enc,
                &op_enc,
                &auto_keys,
                scratch.borrow(),
            );

            //println!("{:08x} {:08x}", c_want, res_enc.decrypt(&module, &sk_prep, scratch.borrow()));
            assert_eq!(c_want, res_enc.decrypt(&module, &sk_prep, scratch.borrow()));
        }
    }
}
