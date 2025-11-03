use fhevm::arithmetic::{RVI32ArithmeticOps, VMArithmetic};
use poulpy_schemes::tfhe::bdd_arithmetic::FheUintPrepared;
use std::collections::HashMap;

use poulpy_backend::FFT64Ref;
use poulpy_core::{
    layouts::{
        Dnum, Dsize, GGSWLayout, GLWEAutomorphismKey, GLWEAutomorphismKeyLayout,
        GLWEAutomorphismKeyPrepared, GLWELayout, GLWESecret, GLWESecretPrepared,
    },
    GLWEPacker, GLWERotate,
};
use poulpy_hal::{
    api::{ModuleNew, ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{Module, ScratchOwned},
    source::Source,
};
use poulpy_schemes::tfhe::bdd_arithmetic::FheUint;
use strum::IntoEnumIterator;

#[test]
fn test_vm_arithmetic_rvi32_fft64_ref() {
    let module: Module<FFT64Ref> = Module::<FFT64Ref>::new(1024);

    let base2k: usize = 13;
    let rank: usize = 1;

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([1u8; 32]);
    let mut source_xe: Source = Source::new([2u8; 32]);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 24);

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
    let mut keys: HashMap<i64, GLWEAutomorphismKeyPrepared<Vec<u8>, FFT64Ref>> = HashMap::new();
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
        keys.insert(*gal_el, atk_prepared);
    });

    let rs1: u32 = 0x0000_0001;
    let rs2: u32 = 0x0000_0002;
    let pc: u32 = 0x0000_0003;
    let imm: u32 = 0x0000_0004;
    let op_id: u32 = 0x0000_00005;

    let mut rd_enc: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&glwe_infos);
    let mut rs1_enc: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::alloc_from_infos(&module, &ggsw_infos);
    let mut rs2_enc: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::alloc_from_infos(&module, &ggsw_infos);
    let mut pc_enc: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::alloc_from_infos(&module, &ggsw_infos);
    let mut imm_enc: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::alloc_from_infos(&module, &ggsw_infos);
    let mut op_id_enc: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::alloc_from_infos(&module, &ggsw_infos);

    rs1_enc.encrypt_sk(
        &module,
        rs1,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    rs2_enc.encrypt_sk(
        &module,
        rs2,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    imm_enc.encrypt_sk(
        &module,
        imm,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    pc_enc.encrypt_sk(
        &module,
        pc,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    op_id_enc.encrypt_sk(
        &module,
        op_id,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    module.eval_ops(
        &mut rd_enc,
        &rs1_enc,
        &rs2_enc,
        &imm_enc,
        &pc_enc,
        RVI32ArithmeticOps::iter(),
        &keys,
        scratch.borrow(),
    );

    let num_ops = RVI32ArithmeticOps::iter().len();

    let mut values = Vec::new();

    let mut i: usize = 0;
    for op in RVI32ArithmeticOps::iter() {
        let value = rd_enc.decrypt(&module, &sk_prep, scratch.borrow());
        println!(
            "{:2} -- {:?}: rs1: {rs1} rs2: {rs2} imm: {imm} pc: {pc} -> {}",
            i, op, value
        );
        values.push(value);
        module.glwe_rotate_inplace(-1, &mut rd_enc, scratch.borrow());
        i += 1;
    }

    module.glwe_rotate_inplace(num_ops as i64, &mut rd_enc, scratch.borrow());

    module.select_rd(&mut rd_enc, &op_id_enc, num_ops, &keys, scratch.borrow());

    println!(
        "op_id: {} -> {}",
        op_id,
        rd_enc.decrypt(&module, &sk_prep, scratch.borrow())
    );
    assert_eq!(
        values[op_id as usize],
        rd_enc.decrypt(&module, &sk_prep, scratch.borrow())
    )
}
