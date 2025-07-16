pub(crate) fn split_by_bounds_mut<'a, T>(
    mut slice: &'a mut [T],
    bounds: &[usize],
) -> Vec<&'a mut [T]> {
    let mut chunks = Vec::with_capacity(bounds.len() - 1);
    let mut last_end = 0;
    for end in bounds.iter() {
        let (left, right) = slice.split_at_mut(*end - last_end);
        chunks.push(left);
        slice = right;
        last_end = *end;
    }

    chunks
}

#[cfg(test)]
pub(crate) mod tests {
    pub(crate) fn u32_to_bits(v: u32) -> Vec<u64> {
        (0..u32::BITS)
            .into_iter()
            .map(|i| ((v >> i) & 1) as u64)
            .collect()
    }

    pub(crate) fn bits_to_u32(bits: &[u64]) -> u32 {
        bits.iter().enumerate().fold(0u32, |acc, (i, b)| {
            assert!(*b == 0 || *b == 1);
            acc + (*b << i) as u32
        })
    }

    pub(crate) fn uint_to_i64(v: u64, log_modulus: u64) -> i64 {
        let v = v % log_modulus;
        if v >= log_modulus / 2 {
            return -((log_modulus - v) as i64);
        } else {
            return v as i64;
        }
    }

    pub(crate) fn i64_to_u64(v: i64, log_modulus: u64) -> u64 {
        if v.is_negative() {
            (1 << log_modulus) - v.abs() as u64
        } else {
            v.abs() as u64
        }
    }
}
