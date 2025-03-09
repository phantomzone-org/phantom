use base2k::{alloc_aligned_u8, Encoding, Module, VecZnx, VecZnxOps, FFT64};
use fhevm::trace::{trace_inplace, trace_inplace_inv, trace_inv_tmp_bytes, trace_tmp_bytes};

#[test]
fn trace_u64() {
    let n: usize = 1 << 5;
    let log_base2k: usize = 15;
    let cols: usize = 3;
    let module: Module = Module::new::<FFT64>(n);

    sub_test("test_trace::<INV:false, NTT:false>", || {
        test_trace::<false>(&module, log_base2k, cols)
    });
}

fn sub_test<F: FnOnce()>(name: &str, f: F) {
    println!("Running {}", name);
    f();
}

fn test_trace<const INV: bool>(module: &Module, log_base2k: usize, cols: usize) {
    let log_k: usize = cols * log_base2k - 5;

    let mut a: VecZnx = module.new_vec_znx(cols);

    let mut have: Vec<i64> = vec![i64::default(); module.n()];
    have.iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = (i + 1) as i64);

    a.encode_vec_i64(log_base2k, log_k, &have, 32);

    let step_start: usize = 2;
    let step_end: usize = module.log_n();

    if INV {
        let mut tmp_bytes: Vec<u8> = alloc_aligned_u8(trace_inv_tmp_bytes(module, cols));

        trace_inplace_inv(
            module,
            log_base2k,
            step_start,
            step_start + 1,
            &mut a,
            &mut tmp_bytes,
        );
        trace_inplace_inv(
            module,
            log_base2k,
            step_start + 1,
            step_end,
            &mut a,
            &mut tmp_bytes,
        );
    } else {
        let mut tmp_bytes: Vec<u8> = alloc_aligned_u8(trace_tmp_bytes(module, cols));

        trace_inplace(
            module,
            log_base2k,
            step_start,
            step_start + 1,
            &mut a,
            &mut tmp_bytes,
        );
        trace_inplace(
            module,
            log_base2k,
            step_start + 1,
            step_end,
            &mut a,
            &mut tmp_bytes,
        );
    }

    let gap: usize = 1 << (module.log_n() - step_start);

    let mut have = vec![i64::default(); module.n()];

    a.decode_vec_i64(log_base2k, log_k, &mut have);

    if INV {
        have.iter().enumerate().for_each(|(i, x)| {
            if i % gap == 0 {
                assert_eq!(*x, 0i64)
            } else {
                assert_eq!(*x, 1 + i as i64)
            }
        });
    } else {
        have.iter().enumerate().for_each(|(i, x)| {
            if i % gap == 0 {
                assert_eq!(*x, 1 + i as i64)
            } else {
                assert_eq!(*x, 0i64)
            }
        });
    }
}
