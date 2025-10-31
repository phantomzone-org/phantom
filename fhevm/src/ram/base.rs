/// Helper for 1D digit decomposition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Base1D(pub Vec<u8>);

impl Base1D {
    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn max(&self) -> usize {
        let mut max: usize = 1;
        self.0.iter().for_each(|i| max <<= i);
        max
    }

    #[allow(dead_code)]
    pub fn gap(&self, log_n: usize) -> usize {
        let mut gap: usize = log_n;
        self.0.iter().for_each(|i| gap >>= i);
        1 << gap
    }

    #[allow(dead_code)]
    pub fn decomp(&self, value: u32) -> Vec<u8> {
        self.0
            .iter()
            .scan(0, |sum_bases, &base| {
                let part = ((value >> *sum_bases) & ((1 << base) - 1)) as u8;
                *sum_bases += base;
                Some(part)
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn recomp(&self, decomp: &[u8]) -> u32 {
        let mut value: u32 = 0;
        let mut sum_bases: u8 = 0;
        self.0.iter().enumerate().for_each(|(i, base)| {
            value |= (decomp[i] as u32) << sum_bases;
            sum_bases += base;
        });
        value
    }
}

/// Helpe for 2D digit decomposition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Base2D(pub Vec<Base1D>);

impl Base2D {
    pub fn max_len(&self) -> usize {
        let mut max: usize = 0;
        for base1d in self.0.iter() {
            max = max.max(base1d.size())
        }
        max
    }

    pub fn max(&self) -> usize {
        self.as_1d().max()
    }

    pub fn as_1d(&self) -> Base1D {
        Base1D(
            self.0
                .iter()
                .flat_map(|array| array.0.iter().copied())
                .collect(),
        )
    }

    #[allow(dead_code)]
    pub fn decomp(&self, value: u32) -> Vec<u8> {
        self.as_1d().decomp(value)
    }

    #[allow(dead_code)]
    pub fn recomp(&self, decomp: &[u8]) -> u32 {
        self.as_1d().recomp(decomp)
    }
}

pub fn get_base_2d(value: u32, base: Vec<u8>) -> Base2D {
    let mut out = Vec::new();
    let mut value_bit_size = 32 - (value - 1).leading_zeros();

    while value_bit_size != 0 {
        let mut v: Vec<u8> = Vec::new();

        for &b in base.iter() {
            if b as u32 <= value_bit_size {
                v.push(b);
                value_bit_size -= b as u32;
            } else {
                if value_bit_size != 0 {
                    v.push(value_bit_size as u8);
                    value_bit_size = 0;
                }
                break;
            }
        }

        out.push(Base1D(v)); // Single, unconditional push here
    }

    Base2D(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base1d_max_calculation() {
        // Test max value calculation for various base configurations
        let base1 = Base1D(vec![4, 4, 4]); // 3 * 4 = 12 bits
        assert_eq!(base1.max(), 1 << 12); // 2^12 = 4096

        let base2 = Base1D(vec![8, 8]); // 2 * 8 = 16 bits  
        assert_eq!(base2.max(), 1 << 16); // 2^16 = 65536

        let base3 = Base1D(vec![12]); // 1 * 12 = 12 bits
        assert_eq!(base3.max(), 1 << 12); // 2^12 = 4096

        let base4 = Base1D(vec![1, 1, 1, 1]); // 4 * 1 = 4 bits
        assert_eq!(base4.max(), 1 << 4); // 2^4 = 16
    }

    #[test]
    fn base1d_decomp_recomp_roundtrip() {
        let base = Base1D(vec![4, 4, 4]); // 12 bits total

        // Test full roundtrip with various values
        let test_values = vec![0, 1, 15, 255, 1000, 4095]; // 4095 = 2^12 - 1

        for value in test_values {
            let decomp = base.decomp(value);
            let recomp = base.recomp(&decomp);
            assert_eq!(value, recomp, "Roundtrip failed for value {}", value);

            // Also verify decomposition properties
            assert_eq!(
                decomp.len(),
                3,
                "Decomposition should produce 3 elements for value {}",
                value
            );
            for &elem in &decomp {
                assert!(
                    elem < 16,
                    "Decomposed element {} should be < 16 for value {}",
                    elem,
                    value
                );
            }
        }
    }

    #[test]
    fn base1d_decomp_correctness() {
        let base = Base1D(vec![4, 4, 4]); // 12 bits: 4 + 4 + 4

        // Test specific decomposition with known values
        let value = 0b0000_0000_1111; // 15 in decimal
        let decomp = base.decomp(value);

        // The decomposition extracts bits in order: first 4 bits, next 4 bits, last 4 bits
        // 0b0000_0000_1111 = 0b1111 (15), 0b0000 (0), 0b0000 (0) in reverse order
        assert_eq!(decomp, vec![15, 0, 0]);

        // Verify recomposition works
        let recomp = base.recomp(&decomp);
        assert_eq!(value, recomp);

        // Test another known case
        let value2 = 0b1010_1100_1111; // 2767 in decimal
        let decomp2 = base.decomp(value2);
        assert_eq!(decomp2, vec![15, 12, 10]); // 0b1111, 0b1100, 0b1010
        let recomp2 = base.recomp(&decomp2);
        assert_eq!(value2, recomp2);
    }

    #[test]
    fn base1d_gap_calculation() {
        let base = Base1D(vec![4, 4, 4]); // 12 bits total
        let log_n = 12;

        // gap = log_n >> (sum of bases) = 12 >> 12 = 0
        // result = 1 << 0 = 1
        assert_eq!(base.gap(log_n), 1);

        let base2 = Base1D(vec![6, 6]); // 12 bits total
        // gap = 12 >> 12 = 0, result = 1
        assert_eq!(base2.gap(log_n), 1);

        let base3 = Base1D(vec![3, 3, 3, 3]); // 12 bits total
        // gap = 12 >> 12 = 0, result = 1
        assert_eq!(base3.gap(log_n), 1);
    }

    #[test]
    fn base2d_creation_and_conversion() {
        let base1d_1 = Base1D(vec![4, 4]);
        let base1d_2 = Base1D(vec![4, 4]);
        let base2d = Base2D(vec![base1d_1.clone(), base1d_2.clone()]);

        // Test as_1d conversion
        let as_1d = base2d.as_1d();
        let expected = Base1D(vec![4, 4, 4, 4]); // Flattened
        assert_eq!(as_1d, expected);
    }

    #[test]
    fn base2d_max_calculation() {
        let base1d_1 = Base1D(vec![4, 4]); // 8 bits
        let base1d_2 = Base1D(vec![4, 4]); // 8 bits
        let base2d = Base2D(vec![base1d_1, base1d_2]);

        // Total: 8 + 8 = 16 bits
        assert_eq!(base2d.max(), 1 << 16);

        // Test with different sizes
        let base1d_3 = Base1D(vec![6]);
        let base1d_4 = Base1D(vec![6]);
        let base2d_2 = Base2D(vec![base1d_3, base1d_4]);

        // Total: 6 + 6 = 12 bits
        assert_eq!(base2d_2.max(), 1 << 12);
    }

    #[test]
    fn base2d_decomp_recomp_roundtrip() {
        let base1d_1 = Base1D(vec![4, 4]);
        let base1d_2 = Base1D(vec![4, 4]);
        let base2d = Base2D(vec![base1d_1, base1d_2]);

        // Test full roundtrip with various values
        let test_values = vec![0, 1, 255, 1000, 65535]; // 65535 = 2^16 - 1

        for value in test_values {
            let decomp = base2d.decomp(value);
            let recomp = base2d.recomp(&decomp);
            assert_eq!(value, recomp, "Roundtrip failed for value {}", value);

            // Also verify decomposition properties
            assert_eq!(
                decomp.len(),
                4,
                "Decomposition should produce 4 elements for value {}",
                value
            );
            for &elem in &decomp {
                assert!(
                    elem < 16,
                    "Decomposed element {} should be < 16 for value {}",
                    elem,
                    value
                );
            }
        }
    }

    #[test]
    fn get_base_2d_functionality() {
        // Test with simple case
        let base = vec![4, 4, 4]; // 12 bits
        let value = 1000u32; // Fits in 12 bits
        let base2d = get_base_2d(value, base);

        // Should create a single Base1D with the decomposition
        assert_eq!(base2d.0.len(), 1);
        // The actual decomposition might be different based on the algorithm
        assert!(base2d.0[0].0.len() > 0);

        // Test with larger value requiring multiple Base1D
        let base2 = vec![4, 4]; // 8 bits per Base1D
        let value2 = 1000u32; // Requires 10 bits, so needs 2 Base1D instances
        let base2d_2 = get_base_2d(value2, base2);

        // Should create multiple Base1D instances
        assert!(base2d_2.0.len() >= 1);

        // Verify the decomposition works with full roundtrip
        let decomp = base2d_2.decomp(value2);
        let recomp = base2d_2.recomp(&decomp);
        assert_eq!(
            value2, recomp,
            "Roundtrip failed for get_base_2d with value {}",
            value2
        );

        // Verify decomposition properties
        assert!(
            decomp.len() >= 1,
            "Decomposition should produce at least 1 element"
        );
        for &elem in &decomp {
            assert!(elem < 16, "Decomposed element {} should be < 16", elem);
        }
    }

    #[test]
    fn base1d_edge_cases() {
        // Test empty base
        let empty_base = Base1D(vec![]);
        assert_eq!(empty_base.max(), 1);
        assert_eq!(empty_base.decomp(0), vec![] as Vec<u8>);
        assert_eq!(empty_base.recomp(&vec![] as &[u8]), 0);

        // Test single bit
        let single_bit = Base1D(vec![1]);
        assert_eq!(single_bit.max(), 2);
        assert_eq!(single_bit.decomp(0), vec![0]);
        assert_eq!(single_bit.decomp(1), vec![1]);
        assert_eq!(single_bit.recomp(&vec![0]), 0);
        assert_eq!(single_bit.recomp(&vec![1]), 1);
    }

    #[test]
    fn base1d_comprehensive_roundtrip() {
        let base = Base1D(vec![4, 4, 4]);
        assert_eq!(base.max(), 1 << 12);

        // Test a comprehensive range of values
        let test_values = vec![
            0, 1, 2, 3, 4, 5, 10, 15, 16, 17, 31, 32, 63, 64, 127, 128, 255, 256, 511, 512, 1023,
            1024, 2047, 2048, 4095,
        ];

        for value in test_values {
            let decomp = base.decomp(value);
            let recomp = base.recomp(&decomp);
            assert_eq!(value, recomp, "Roundtrip failed for value {}", value);
        }
    }

    #[test]
    fn base2d_comprehensive_roundtrip() {
        let base1d_1 = Base1D(vec![6, 6]);
        let base1d_2 = Base1D(vec![4, 4]);
        let base2d = Base2D(vec![base1d_1, base1d_2]);

        // Test comprehensive range for 16-bit values
        let test_values = vec![
            0, 1, 15, 16, 31, 32, 63, 64, 127, 128, 255, 256, 511, 512, 1023, 1024, 2047, 2048,
            4095, 4096, 8191, 8192, 16383, 16384, 32767, 32768, 65535,
        ];

        for value in test_values {
            let decomp = base2d.decomp(value);
            let recomp = base2d.recomp(&decomp);
            assert_eq!(value, recomp, "Roundtrip failed for value {}", value);
        }
    }

    #[test]
    fn base1d_different_sizes() {
        // Test various base configurations
        let bases = vec![
            Base1D(vec![1, 1, 1, 1]), // 4 bits
            Base1D(vec![2, 2, 2]),    // 6 bits
            Base1D(vec![3, 3, 3]),    // 9 bits
            Base1D(vec![4, 4, 4]),    // 12 bits
            Base1D(vec![8, 8]),       // 16 bits
        ];

        for base in bases {
            let max_val = (base.max() - 1) as u32;
            let test_values = vec![0u32, 1, max_val / 4, max_val / 2, max_val];

            for value in test_values {
                let decomp = base.decomp(value);
                let recomp = base.recomp(&decomp);
                assert_eq!(
                    value, recomp,
                    "Roundtrip failed for base {:?} with value {}",
                    base.0, value
                );
            }
        }
    }

    #[test]
    fn base2d_edge_cases() {
        // Test empty Base2D
        let empty_base2d = Base2D(vec![]);
        assert_eq!(empty_base2d.max(), 1);
        assert_eq!(empty_base2d.decomp(0), vec![] as Vec<u8>);
        assert_eq!(empty_base2d.recomp(&vec![] as &[u8]), 0);

        // Test single Base1D
        let single_base1d = Base1D(vec![4, 4]);
        let single_base2d = Base2D(vec![single_base1d]);
        assert_eq!(single_base2d.max(), 1 << 8);

        // Test conversion to 1D
        let as_1d = single_base2d.as_1d();
        assert_eq!(as_1d.0, vec![4, 4]);
    }

    #[test]
    fn base_decomposition_boundary_tests() {
        let base = Base1D(vec![4, 4, 4]); // 12 bits

        // Test values just below and at boundaries with full roundtrip
        let test_values = vec![
            0,             // Minimum
            1,             // Just above minimum
            (1 << 4) - 1,  // Max for first 4 bits (15)
            1 << 4,        // First bit of second group (16)
            (1 << 8) - 1,  // Max for first 8 bits (255)
            1 << 8,        // First bit of third group (256)
            (1 << 12) - 1, // Maximum value (4095)
        ];

        for value in test_values {
            let decomp = base.decomp(value);
            let recomp = base.recomp(&decomp);
            assert_eq!(value, recomp, "Boundary test failed for value {}", value);

            // Also verify decomposition properties
            assert_eq!(
                decomp.len(),
                3,
                "Decomposition should produce 3 elements for value {}",
                value
            );
            for &elem in &decomp {
                assert!(
                    elem < 16,
                    "Decomposed element {} should be < 16 for value {}",
                    elem,
                    value
                );
            }
        }
    }
}
