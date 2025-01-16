use fhevm::packing::{pack, StreamRepacker};
use fhevm::trace::gen_auto_perms;
use math::modulus::WordOps;
use math::poly::Poly;
use math::ring::Ring;

#[test]
fn packing_u64() {
    let n: usize = 1 << 5;
    let q_base: u64 = 65537u64;
    let q_power: usize = 1usize;
    let ring: Ring<u64> = Ring::new(n, q_base, q_power);

    sub_test("test_packing_u64::<NTT:false>", || {
        test_packing_sparse_u64::<false>(&ring, 1)
    });
    sub_test("test_packing_u64::<NTT:true>", || {
        test_packing_sparse_u64::<true>(&ring, 1)
    });
    sub_test("test_packing_sparse_u64::<NTT:false>", || {
        test_packing_sparse_u64::<false>(&ring, 3)
    });
    sub_test("test_packing_sparse_u64::<NTT:true>", || {
        test_packing_sparse_u64::<true>(&ring, 3)
    });
}

fn sub_test<F: FnOnce()>(name: &str, f: F) {
    println!("Running {}", name);
    f();
}

fn test_packing_sparse_u64<const NTT: bool>(ring: &Ring<u64>, gap: usize) {
    let n: usize = ring.n();

    let mut result: Vec<Option<&mut Poly<u64>>> = Vec::with_capacity(n);
    result.resize_with(n, || None);

    let mut polys: Vec<Poly<u64>> = vec![ring.new_poly(); (n + gap - 1) / gap];

    polys.iter_mut().enumerate().for_each(|(i, poly)| {
        poly.fill(&((1 + i * gap) as u64));
        if NTT {
            ring.ntt_inplace::<false>(poly);
        }
        result[i * gap] = Some(poly);
    });

    let (auto_perms, gal_els) = gen_auto_perms::<true>(ring);

    pack::<true, NTT>(ring, &mut result, &gal_els, &auto_perms, ring.log_n());

    if let Some(poly) = result[0].as_mut() {
        if NTT {
            ring.intt_inplace::<false>(poly);
        }

        poly.0.iter().enumerate().for_each(|(i, x)| {
            if i % gap == 0 {
                assert_eq!(*x, (1 + i) as u64)
            } else {
                assert_eq!(*x, 0u64)
            }
        });
    }
}

#[test]
fn packing_streaming_u64() {
    let n: usize = 1 << 5;
    let q_base: u64 = 65537u64;
    let q_power: usize = 1usize;
    let ring: Ring<u64> = Ring::new(n, q_base, q_power);

    sub_test("test_packing_streaming_dense_u64::<NTT:true>", || {
        test_packing_streaming_dense_u64::<true>(&ring)
    });
}

fn test_packing_streaming_dense_u64<const NTT: bool>(ring: &Ring<u64>) {
    let n: usize = ring.n();

    let mut values: Vec<u64> = vec![0; n];
    values
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = (i + 1) as u64);

    let gap: usize = 3;

    let mut packer = StreamRepacker::new(ring);

    let mut results: Vec<Poly<u64>> = Vec::new();

    let mut poly: Poly<u64> = ring.new_poly();
    for i in 0..n {
        let i_rev: usize = i.reverse_bits_msb(ring.log_n() as u32);

        if i_rev % gap == 0 {
            poly.fill(&values[i_rev]);
            if NTT {
                ring.ntt_inplace::<false>(&mut poly);
            }
            packer.add::<NTT>(ring, Some(&poly), &mut results)
        } else {
            packer.add::<NTT>(ring, None, &mut results)
        }
    }

    packer.flush::<NTT>(ring, &mut results);

    let result: &mut Poly<u64> = &mut results[0];

    if NTT {
        ring.intt_inplace::<false>(result);
    }

    result.0.iter().enumerate().for_each(|(i, x)| {
        if i % gap == 0 {
            assert_eq!(*x, values[i] as u64)
        } else {
            assert_eq!(*x, 0u64)
        }
    });
}
