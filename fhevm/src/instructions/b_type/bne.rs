//! bne    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] !=  x[rs2]), pc += sext(imm[19:0])

use crate::instructions::{decompose, reconstruct};

pub struct Bne();

impl Bne {
    pub fn apply(imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8], pc: &[u8; 8]) -> [u8; 8] {
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        let x_rs2_u32: u32 = reconstruct(x_rs2);
        if x_rs1_u32 != x_rs2_u32 {
            decompose(reconstruct(pc).wrapping_add(reconstruct(imm)))
        } else {
            *pc
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::{decompose, reconstruct, sext};
    #[test]
    pub fn apply() {
        let imm: u32 = sext(0x1FFF, 12);
        let x_rs1: u32 = 0x0FFF_FFFF;
        let x_rs2: u32 = 0x1FFF_FFFF;
        let pc: u32 = 4;
        let pc_w_decomp = Bne::apply(
            &decompose(imm),
            &decompose(x_rs1),
            &decompose(x_rs2),
            &decompose(pc),
        );
        let pc_w: u32 = reconstruct(&pc_w_decomp);
        assert_eq!(pc_w, pc.wrapping_add(imm));

        let x_rs1: u32 = 0x1FFF_FFFF;
        let x_rs2: u32 = x_rs1;
        let pc_w_decomp = Bne::apply(
            &decompose(imm),
            &decompose(x_rs1),
            &decompose(x_rs2),
            &decompose(pc),
        );
        let pc_w: u32 = reconstruct(&pc_w_decomp);
        assert_eq!(pc_w, pc)
    }
}
