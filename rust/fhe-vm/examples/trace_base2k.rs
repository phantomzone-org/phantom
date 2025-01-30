use base2k::{Module, VecZnx, FFT64};
use fhevm::trace::trace_inplace;
use sampling::source::Source;

fn main() {
    let log_n: usize = 5;
    let n: usize = 1 << log_n;
    let log_q: usize = 128;
    let log_base2k: usize = 16;

    let module: Module = Module::new::<FFT64>(n);

    let mut source = Source::new([0; 32]);

    let mut a: VecZnx = VecZnx::new(n, log_base2k, log_q);
    let mut b: VecZnx = VecZnx::new(n, log_base2k, log_q);

    let mut values: Vec<i64> = vec![i64::default(); n];

    values
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = (1 + i as i64) << 32);

    let mut carry: Vec<u8> = vec![0; module.vec_znx_big_normalize_tmp_bytes()];

    a.from_i64(&values, 54);
    a.add_normal(&mut source, 3.2, 19.0);
    a.normalize(&mut carry);

    a.to_i64(&mut values);
    println!("{:?}", values);

    (0..a.limbs()).for_each(|i| println!("{}: {:?}", i, a.at(i)));

    let mut buf_bytes: Vec<u8> = vec![u8::default(); module.vec_znx_big_normalize_tmp_bytes()];
    let mut buf_b: VecZnx = VecZnx::new(n, log_base2k, log_q);
    trace_inplace::<false>(&module, 0, log_n, &mut a, None, &mut buf_b, &mut buf_bytes);

    a.to_i64(&mut values);
    (0..a.limbs()).for_each(|i| println!("{}: {:?}", i, a.at(i)));
    println!("{:?}", values);
}
