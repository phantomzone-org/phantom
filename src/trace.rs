use base2k::{Module, VecZnxCommon, VecZnxOps};

pub fn trace<A: VecZnxCommon, B: VecZnxCommon>(
    module: &Module,
    log_base2k: usize,
    step_start: usize,
    step_end: usize,
    a: &mut A,
    buf: &mut B,
    carry: &mut [u8],
) {
    trace_inplace_core(module, log_base2k, step_start, step_end, a, buf, carry);
}

pub fn trace_inv<A: VecZnxCommon, B: VecZnxCommon, C: VecZnxCommon>(
    module: &Module,
    log_base2k: usize,
    step_start: usize,
    step_end: usize,
    b: &mut B,
    a: &A,
    buf: &mut C,
    carry: &mut [u8],
) {
    b.copy_from(a);
    trace_inplace_core(module, log_base2k, step_start, step_end, b, buf, carry);
    module.vec_znx_negate_inplace(b);
    module.vec_znx_add_inplace(b, a);
    b.normalize(log_base2k, carry);
}

pub fn trace_inplace<A: VecZnxCommon, B: VecZnxCommon>(
    module: &Module,
    log_base2k: usize,
    step_start: usize,
    step_end: usize,
    a: &mut A,
    buf: &mut B,
    carry: &mut [u8],
) {
    trace_inplace_core(module, log_base2k, step_start, step_end, a, buf, carry);
    a.normalize(log_base2k, carry);
}

pub fn trace_inplace_inv<A: VecZnxCommon, B: VecZnxCommon, C: VecZnxCommon>(
    module: &Module,
    log_base2k: usize,
    step_start: usize,
    step_end: usize,
    a: &mut A,
    buf_a: &mut B,
    buf: &mut C,
    carry: &mut [u8],
) {
    buf_a.copy_from(a);
    trace_inplace_core(module, log_base2k, step_start, step_end, a, buf, carry);
    module.vec_znx_negate_inplace(a);
    module.vec_znx_add_inplace(a, buf_a);
    a.normalize(log_base2k, carry);
}

pub fn trace_inplace_core<A: VecZnxCommon, B: VecZnxCommon>(
    module: &Module,
    log_base2k: usize,
    step_start: usize,
    step_end: usize,
    a: &mut A,
    buf: &mut B,
    carry: &mut [u8],
) {
    assert!(
        buf.cols() >= a.cols(),
        "invalid buf: buf.limbs={} < a.limbs()={}",
        buf.cols(),
        a.cols()
    );

    (step_start..step_end).for_each(|i| {
        a.rsh(log_base2k, 1, carry);

        if i == 0 {
            module.vec_znx_automorphism(-1, buf, a, a.cols());
        } else {
            module.vec_znx_automorphism(module.galois_element(1 << (i - 1)), buf, a, a.cols());
        }

        module.vec_znx_add_inplace(a, buf);
    });
}
