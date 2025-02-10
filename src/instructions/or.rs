//! or     | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] | x[rs2]

//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] & x[rs2]

use super::{decomp, reconstruct, Arithmetic};
use crate::parameters::U32DECOMP;

pub struct Or();

impl Arithmetic for Or {
    fn apply(&self, _imm: &[u32], x_rs1: &[u32], x_rs2: &[u32]) -> Vec<u32> {
        let x_rs1_u32: u32 = reconstruct(x_rs1, &U32DECOMP);
        let x_rs2_u32: u32 = reconstruct(x_rs2, &U32DECOMP);
        decomp(x_rs1_u32 | x_rs2_u32, &U32DECOMP)
    }
}
