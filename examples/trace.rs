use base2k::{Encoding, Infos, Module, Sampling, VecZnx, VecZnxOps, FFT64};
use fhevm::trace::trace_inplace;
use sampling::source::Source;

fn main() {
    let log_n: usize = 5;
    let n: usize = 1 << log_n;
    let limbs: usize = 5;
    let log_base2k: usize = 16;
    let log_k: usize = limbs * log_base2k - 5;

    let module: Module = Module::new::<FFT64>(n);

    let mut source = Source::new([0; 32]);

    let mut a: VecZnx = module.new_vec_znx(limbs);

    let mut values: Vec<i64> = vec![i64::default(); n];

    values
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = (1 + i as i64) << 32);

    let mut carry: Vec<u8> = vec![0; module.vec_znx_big_normalize_tmp_bytes()];

    a.encode_vec_i64(log_base2k, log_k, &values, 54);
    a.add_normal(log_base2k, limbs * log_base2k, &mut source, 3.2, 19.0);
    a.normalize(log_base2k, &mut carry);

    a.decode_vec_i64(log_base2k, log_k, &mut values);
    println!("{:?}", values);

    (0..a.limbs()).for_each(|i| println!("{}: {:?}", i, a.at(i)));

    let mut buf_bytes: Vec<u8> = vec![u8::default(); module.vec_znx_big_normalize_tmp_bytes()];
    let mut buf_b: VecZnx = module.new_vec_znx(limbs);
    trace_inplace::<false>(
        &module,
        log_base2k,
        0,
        log_n,
        &mut a,
        None,
        &mut buf_b,
        &mut buf_bytes,
    );

    a.decode_vec_i64(log_base2k, log_k, &mut values);
    (0..a.limbs()).for_each(|i| println!("{}: {:?}", i, a.at(i)));
    println!("{:?}", values);
}
