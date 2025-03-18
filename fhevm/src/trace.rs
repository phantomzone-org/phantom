use base2k::{Infos, Module, VecZnx, VecZnxOps};

pub fn trace_tmp_bytes(module: &Module, cols: usize) -> usize {
    module.bytes_of_vec_znx(cols) + module.vec_znx_normalize_tmp_bytes()
}

pub fn trace(
    module: &Module,
    log_base2k: usize,
    step_start: usize,
    step_end: usize,
    b: &mut VecZnx,
    a: &VecZnx,
    tmp_bytes: &mut [u8],
) {
    b.copy_from(a);
    trace_inplace_core(module, log_base2k, step_start, step_end, b, tmp_bytes);
}

pub fn trace_inv(
    module: &Module,
    log_base2k: usize,
    step_start: usize,
    step_end: usize,
    b: &mut VecZnx,
    a: &VecZnx,
    tmp_bytes: &mut [u8],
) {
    b.copy_from(a);
    trace_inplace_core(module, log_base2k, step_start, step_end, b, tmp_bytes);
    module.vec_znx_negate_inplace(b);
    module.vec_znx_add_inplace(b, a);
    b.normalize(log_base2k, tmp_bytes);
}

pub fn trace_inplace(
    module: &Module,
    log_base2k: usize,
    step_start: usize,
    step_end: usize,
    a: &mut VecZnx,
    tmp_bytes: &mut [u8],
) {
    trace_inplace_core(module, log_base2k, step_start, step_end, a, tmp_bytes);
    a.normalize(log_base2k, tmp_bytes);
}

pub fn trace_inv_tmp_bytes(module: &Module, cols: usize) -> usize {
    2 * module.bytes_of_vec_znx(cols) + module.vec_znx_normalize_tmp_bytes()
}

pub fn trace_inplace_inv(
    module: &Module,
    log_base2k: usize,
    step_start: usize,
    step_end: usize,
    a: &mut VecZnx,
    tmp_bytes: &mut [u8],
) {
    let a_cols: usize = a.cols();
    assert!(tmp_bytes.len() >= trace_inv_tmp_bytes(module, a_cols));
    let vec_znx_bytes: usize = module.bytes_of_vec_znx(a_cols);
    let (tmp_bytes_vec_znx, tmp_bytes) = tmp_bytes.split_at_mut(vec_znx_bytes);
    let mut tmp_a: VecZnx = VecZnx::from_bytes_borrow(module.n(), a_cols, tmp_bytes_vec_znx);
    tmp_a.copy_from(a);
    trace_inplace_core(module, log_base2k, step_start, step_end, a, tmp_bytes);
    module.vec_znx_negate_inplace(a);
    module.vec_znx_add_inplace(a, &tmp_a);
    a.normalize(log_base2k, tmp_bytes);
}

pub fn trace_inplace_core_tmp_bytes(module: &Module, cols: usize) -> usize {
    module.bytes_of_vec_znx(cols) + module.vec_znx_normalize_tmp_bytes()
}

pub fn trace_inplace_core(
    module: &Module,
    log_base2k: usize,
    step_start: usize,
    step_end: usize,
    a: &mut VecZnx,
    tmp_bytes: &mut [u8],
) {
    let a_cols: usize = a.cols();
    assert!(tmp_bytes.len() >= trace_tmp_bytes(module, a_cols));

    let vec_znx_bytes: usize = module.bytes_of_vec_znx(a_cols);
    let (tmp_bytes_vec_znx, tmp_bytes_carry) = tmp_bytes.split_at_mut(vec_znx_bytes);
    let mut tmp_a: VecZnx = VecZnx::from_bytes_borrow(module.n(), a_cols, tmp_bytes_vec_znx);

    (step_start..step_end).for_each(|i| {
        a.rsh(log_base2k, 1, tmp_bytes_carry);
        if i == 0 {
            module.vec_znx_automorphism(-1, &mut tmp_a, a, a_cols);
        } else {
            module.vec_znx_automorphism(module.galois_element(1 << (i - 1)), &mut tmp_a, a, a_cols);
        }

        module.vec_znx_add_inplace(a, &tmp_a);
    });
}
