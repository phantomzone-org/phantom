use base2k::{Module, VecZnx};

pub fn trace<const INV: bool>(
    module: &Module,
    step_start: usize,
    step_end: usize,
    b: &mut VecZnx,
    a: &VecZnx,
    buf: &mut VecZnx,
    carry: &mut [u8],
) {
    b.copy_from(a);
    trace_inplace_core(module, step_start, step_end, b, buf, carry);
    if INV {
        module.vec_znx_negate_inplace(b);
        module.vec_znx_add_inplace(b, a);
    }
    b.normalize(carry);
}

pub fn trace_inplace<const INV: bool>(
    module: &Module,
    step_start: usize,
    step_end: usize,
    a: &mut VecZnx,
    buf_a: Option<&mut VecZnx>,
    buf: &mut VecZnx,
    carry: &mut [u8],
) {
    if INV {
        if let Some(buf_a) = buf_a {
            buf_a.copy_from(a);
            trace_inplace_core(module, step_start, step_end, a, buf, carry);
            module.vec_znx_negate_inplace(a);
            module.vec_znx_add_inplace(a, buf_a);
        } else {
            panic!("invalid buf_a: should note be NONE if INV=true")
        }
    } else {
        trace_inplace_core(module, step_start, step_end, a, buf, carry);
    }
    a.normalize(carry);
}

pub fn trace_inplace_core(
    module: &Module,
    step_start: usize,
    step_end: usize,
    a: &mut VecZnx,
    buf: &mut VecZnx,
    carry: &mut [u8],
) {
    (step_start..step_end).for_each(|i| {
        a.rsh(1, carry);

        if i == 0 {
            module.vec_znx_automorphism(-1, buf, a);
        } else {
            module.vec_znx_automorphism(module.galois_element(1 << (i - 1)), buf, a);
        }

        module.vec_znx_add_inplace(a, buf);
    });
}
