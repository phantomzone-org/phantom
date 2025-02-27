//! # RD UPDATE         
//! ID |  OP  |     4      |      4     |     4     |     4    |     4    |  5  |  5  |  5 |
//!    |------|------------|------------|-----------|----------|----------|-----|-----|----|
//!  0 |none  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rd]
//!  1 |lui   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(imm[19:0] << 12
//!  2 |auipc | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + sext(imm[19:0] << 12)
//!  3 |addi  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] + sext(imm[11:0])
//!  4 |slti  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = (x[rs1] < sext(imm[11:0])) ? 1 : 0
//!  5 |sltiu | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = (x[rs1] <u sext(imm[11:0])) ? 1 : 0
//!  6 |xori  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] ^ sext(imm[11:0])
//!  7 |ori   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] | sext(imm[11:0])
//!  8 |andi  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] & sext(imm[11:0])
//!  9 |slli  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] << rs2
//! 10 |srli  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] >> rs2 (logical)
//! 11 |srai  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] >> rs2 (arithmetic)
//!
//! 12 |add   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] + x[rs2]
//! 13 |sub   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] - x[rs2]
//! 14 |sll   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] << x[rs2]
//! 15 |slt   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = (x[rs1] < x[rs2]) ? 1 : 0
//! 16 |sltu  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = (x[rs1] <u x[rs2]) ? 1 : 0   
//! 17 |xor   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] ^ x[rs2]
//! 18 |srl   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] >> x[rs2] (logical)
//! 19 |sra   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] >> x[rs2] (arithmetic)
//! 20 |or    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] | x[rs2]  
//! 21 |and   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] & x[rs2]
//!
//! 22 |lb    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(M[x[rs1] + sext(imm[11:0])][7:0])
//! 23 |lh    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(M[x[rs1] + sext(imm[11:0])][15:0])
//! 24 |lw    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(M[x[rs1] + sext(imm[11:0])][31:0])
//! 25 |lbu   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = M[x[rs1] + sext(imm[11:0])][7:0]
//! 26 |lhu   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = M[x[rs1] + sext(imm[11:0])][15:0]
//!
//! 27 |jal   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + 4
//! 28 |jalr  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + 4
//!
//! # MEMORY UPDATE
//! ID |  OP  |     4      |      4     |     4     |     4    |     4    |  5  |  5  |  5 |
//!    |------|------------|------------|-----------|----------|----------|-----|-----|----|
//!  0 |none  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | M[x[rs1] + sext(imm[11:0])] = M[x[rs1] + sext(imm[11:0])]
//!  1 |sb    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | M[x[rs1] + sext(imm[11:0])] = x[rs2][7:0]
//!  2 |sh    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | M[x[rs1] + sext(imm[11:0])] = x[rs2][15:0]
//!  3 |sw    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | M[x[rs1] + sext(imm[11:0])] = x[rs2][31:0]
//!
//! # PC UPDATE
//! ID |  OP  |     4      |      4     |     4     |     4    |     4    |  5  |  5  |  5 |
//!    |------|------------|------------|-----------|----------|----------|-----|-----|----|
//!  0 |one   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | pc += 4
//!  1 |jal   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | pc += sext(imm[19:0])
//!  2 |jalr  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | t = pc + 4; pc = (x[rs1] + sext(imm[11:0])) & ~1
//!  3 |beq   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] ==  x[rs2]), pc += sext(imm[19:0])
//!  4 |bne   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] !=  x[rs2]), pc += sext(imm[19:0])
//!  5 |blt   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] <   x[rs2]), pc += sext(imm[19:0])
//!  6 |bge   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] >=s x[rs2]), pc += sext(imm[19:0])
//!  7 |bltu  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] <u  x[rs2]), pc += sext(imm[19:0])
//!  8 |bgeu  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] >=u x[rs2]), pc += sext(imm[19:0])

pub mod add;
pub mod addi;
pub mod and;
pub mod andi;
pub mod auipc;
pub mod beq;
pub mod bge;
pub mod bgeu;
pub mod blt;
pub mod bltu;
pub mod bne;
pub mod jal;
pub mod jalr;
pub mod lb;
pub mod lbu;
pub mod lh;
pub mod lhu;
pub mod lui;
pub mod lw;
pub mod or;
pub mod ori;
pub mod sb;
pub mod sh;
pub mod sll;
pub mod slli;
pub mod slt;
pub mod slti;
pub mod sltiu;
pub mod sltu;
pub mod sra;
pub mod srai;
pub mod srl;
pub mod srli;
pub mod sub;
pub mod sw;
pub mod xor;
pub mod xori;

use crate::address::Address;
use crate::circuit_bootstrapping::CircuitBootstrapper;
use crate::memory::Memory;
use base2k::Module;
use itertools::izip;

pub fn sext(x: u32, i: u32) -> u32 {
    x.wrapping_sub(1 << (i - 1))
}

pub fn reconstruct(x: &[u32], base: &[usize]) -> u32 {
    let mut y: u32 = 0;
    let mut sum_bases: u32 = 0;
    izip!(x.iter(), base.iter()).for_each(|(a, b)| {
        y |= a << sum_bases;
        sum_bases += *b as u32;
    });
    y
}

pub fn decomp(x: u32, base: &[usize]) -> Vec<u32> {
    let mut y: Vec<u32> = Vec::new();
    let mut remain: u32 = x;
    base.iter().for_each(|i| {
        let mask: u32 = (1 << i) - 1;
        y.push(remain & mask);
        remain >>= i
    });
    y
}

pub trait Arithmetic {
    fn apply(&self, imm: &[u32], x_rs1: &[u32], x_rs2: &[u32]) -> Vec<u32>;
}

pub trait PcUpdates {
    fn apply(&self, imm: &[u32], x_rs1: &[u32], x_rs2: &[u32], pc: &[u32]) -> (Vec<u32>, Vec<u32>);
}

pub trait Store {
    fn apply(
        &self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm: &[u32],
        x_rs1: &[u32],
        memory: &mut Memory,
        circuit_btp: &CircuitBootstrapper,
        address: &mut Address,
        tmp_bytes: &mut [u8],
    );
}

pub trait Load {
    fn apply(
        &self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm: &[u32],
        x_rs1: &[u32],
        memory: &mut Memory,
        circuit_btp: &CircuitBootstrapper,
        address: &mut Address,
        tmp_bytes: &mut [u8],
    ) -> Vec<u32>;
}

pub enum StoreOps {
    Sb(sb::Sb),
    //Sh(sh::Sh),
    //Sw(sw::Sw),
}

pub enum LoadOps {
    Lb(lb::Lb),
    Lbu(lbu::Lbu),
    Lh(lh::Lh),
    Lhu(lhu::Lhu),
    Lw(lw::Lw),
}

pub enum PcUpdatesOps {
    Auipc(auipc::Auipc),
    Beq(beq::Beq),
    Bge(bge::Bge),
    Bgeu(bgeu::Bgeu),
    Blt(blt::Blt),
    Bltu(bltu::Bltu),
    Bne(bne::Bne),
    Jal(jal::Jal),
    Jalr(jalr::Jalr),
}

pub enum ArithmeticOps {
    Add(add::Add),
    Addi(addi::Addi),
    And(and::And),
    Andi(andi::Andi),
    Lui(lui::Lui),
    Or(or::Or),
    Ori(ori::Ori),
    //Sll(sll::Sll),
    //Slli(slli::Slli),
    //Slt(slt::Slt),
    //Slti(slti::Slti),
    //Sltiu(sltiu::Sltiu),
    //Sltu(sltu::Sltu),
    //Sra(sra::Sra),
    //Srai(srai::Srai),
    //Srl(slr::Srl),
    //Sub(sub::Sub),
    //Xor(xor::Xor),
    //Xori(xori::Xori),
}

pub struct Instructions {
    imm_19: Vec<u8>,
    imm_15: Vec<u8>,
    imm_11: Vec<u8>,
    imm_7: Vec<u8>,
    imm_3: Vec<u8>,
    rs2: Vec<u8>,
    rs1: Vec<u8>,
    rd: Vec<u8>,
    rd_w: Vec<u8>,
    mem_w: Vec<u8>,
    pc_w: Vec<u8>,
}

impl Instructions {
    pub fn new() -> Self {
        Instructions {
            imm_19: Vec::new(),
            imm_15: Vec::new(),
            imm_11: Vec::new(),
            imm_7: Vec::new(),
            imm_3: Vec::new(),
            rs2: Vec::new(),
            rs1: Vec::new(),
            rd: Vec::new(),
            rd_w: Vec::new(),
            mem_w: Vec::new(),
            pc_w: Vec::new(),
        }
    }

    pub fn add(&mut self, instruction: u32) {
        let imm_19: u8;
        let imm_15: u8;
        let imm_11: u8;
        let imm_7: u8;
        let imm_3: u8;
        let rs2: u8;
        let rs1: u8;
        let rd: u8;
        let rd_w: u8;
        let mem_w: u8;
        let pc_w: u8;

        let op_id: u32 = extract(instruction, 6, 0);

        match op_id {
            // lui imm[31:12] rd[11:7]
            0b0110111 => {
                (
                    imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
                ) = decode_0110111(instruction);
            }

            // auipc imm[31:12] rd[11:7]
            0b0010111 => {
                (
                    imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
                ) = decode_0010111(instruction);
            }

            // addi  | 00000| 000 |  3 | imm[31:20] | rs1[19:15] | rd[11:7]
            // slti  | 00000| 010 |  4 | imm[31:20] | rs1[19:15] | rd[11:7]
            // sltiu | 00000| 011 |  5 | imm[31:20] | rs1[19:15] | rd[11:7]
            // xori  | 00000| 100 |  6 | imm[31:20] | rs1[19:15] | rd[11:7]
            // ori   | 00000| 110 |  7 | imm[31:20] | rs1[19:15] | rd[11:7]
            // andi  | 00000| 111 |  8 | imm[31:20] | rs1[19:15] | rd[11:7]
            // slli  | 00000| 001 |  9 | imm[31:20] | rs1[19:15] | rd[11:7]
            // slri  | 00000| 101 | 10 | imm[31:20] | rs1[19:15] | rd[11:7]
            // srai  | 01000| 101 | 11 | imm[31:20] | rs1[19:15] | rd[11:7]
            0b0010011 => {
                (
                    imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
                ) = decode_0010011(instruction);
            }

            // add  | 12 | 00000 | 00 | 000 | rs2[24:20] | rs1[19:15] | rd[11:7]
            // sub  | 13 | 01000 | 00 | 000 | rs2[24:20] | rs1[19:15] | rd[11:7]
            // sll  | 14 | 00000 | 00 | 001 | rs2[24:20] | rs1[19:15] | rd[11:7]
            // slt  | 15 | 00000 | 00 | 010 | rs2[24:20] | rs1[19:15] | rd[11:7]
            // sltu | 16 | 00000 | 00 | 011 | rs2[24:20] | rs1[19:15] | rd[11:7]
            // xor  | 17 | 00000 | 00 | 100 | rs2[24:20] | rs1[19:15] | rd[11:7]
            // srl  | 18 | 00000 | 00 | 101 | rs2[24:20] | rs1[19:15] | rd[11:7]
            // sra  | 19 | 01000 | 00 | 101 | rs2[24:20] | rs1[19:15] | rd[11:7]
            // or   | 20 | 00000 | 00 | 110 | rs2[24:20] | rs1[19:15] | rd[11:7]
            // and  | 21 | 00000 | 00 | 111 | rs2[24:20] | rs1[19:15] | rd[11:7]
            0b0110011 => {
                (
                    imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
                ) = decode_0110011(instruction);
            }

            // lb   | 000 | 22 | imm[31:20] | rs1[19:15] | rd[11:7]
            // lh   | 001 | 23 | imm[31:20] | rs1[19:15] | rd[11:7]
            // lw   | 010 | 24 | imm[31:20] | rs1[19:15] | rd[11:7]
            // lbu  | 100 | 25 | imm[31:20] | rs1[19:15] | rd[11:7]
            // lhu  | 101 | 26 | imm[31:20] | rs1[19:15] | rd[11:7]
            0b0000011 => {
                (
                    imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
                ) = decode_0000011(instruction);
            }

            // jal  | 27 | 1 | offset[20|10:1|11|19:12] | rd[11:7]
            0b1101111 => {
                (
                    imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
                ) = decode_1101111(instruction);
            }

            // jalr | 28 | 2 | offset[11:0] | rs1[19:15] | rd[11:7]
            0b1100111 => {
                (
                    imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
                ) = decode_1100111(instruction);
            }

            // beq  | 000 | 3 | offset[12|10:5|4:1|11] | rs2[24:20] | rs1[19:15]
            // bne  | 001 | 4 | offset[12|10:5|4:1|11] | rs2[24:20] | rs1[19:15]
            // blt  | 100 | 5 | offset[12|10:5|4:1|11] | rs2[24:20] | rs1[19:15]
            // bge  | 101 | 6 | offset[12|10:5|4:1|11] | rs2[24:20] | rs1[19:15]
            // bltu | 110 | 7 | offset[12|10:5|4:1|11] | rs2[24:20] | rs1[19:15]
            // bgeu | 111 | 8 | offset[12|10:5|4:1|11] | rs2[24:20] | rs1[19:15]
            0b1100011 => {
                (
                    imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
                ) = decode_1100011(instruction);
            }

            // sb | 000 | 1 | offset[11:5] | rs2[24:20] | rs1[19:15] | offset[4:0]
            // sh | 001 | 2 | offset[11:5] | rs2[24:20] | rs1[19:15] | offset[4:0]
            // sw | 010 | 3 | offset[11:5] | rs2[24:20] | rs1[19:15] | offset[4:0]
            0b0100011 => {
                (
                    imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
                ) = decode_0100011(instruction);
            }

            _ => {
                panic!(
                    "invalid instruction {:032b} -> unrecoginzed [6:0]: {:07b}",
                    instruction, op_id
                )
            }
        }

        self.imm_19.push(imm_19);
        self.imm_15.push(imm_15);
        self.imm_11.push(imm_11);
        self.imm_7.push(imm_7);
        self.imm_3.push(imm_3);
        self.rs2.push(rs2);
        self.rs1.push(rs1);
        self.rd.push(rd);
        self.rd_w.push(rd_w);
        self.mem_w.push(mem_w);
        self.pc_w.push(pc_w);
    }

    pub fn assert_size(&self, size: usize) {
        assert_eq!(self.imm_19.len(), size);
        assert_eq!(self.imm_15.len(), size);
        assert_eq!(self.imm_11.len(), size);
        assert_eq!(self.imm_7.len(), size);
        assert_eq!(self.imm_3.len(), size);
        assert_eq!(self.rs2.len(), size);
        assert_eq!(self.rs1.len(), size);
        assert_eq!(self.rd.len(), size);
        assert_eq!(self.rd_w.len(), size);
        assert_eq!(self.mem_w.len(), size);
        assert_eq!(self.pc_w.len(), size);
    }

    pub fn assert_instruction(
        &self,
        idx: usize,
        imm_19: u8,
        imm_15: u8,
        imm_11: u8,
        imm_7: u8,
        imm_3: u8,
        rs2: u8,
        rs1: u8,
        rd: u8,
        rd_w: u8,
        mem_w: u8,
        pc_w: u8,
    ) {
        let number_of_instructions: usize = self.imm_19.len();
        assert!(number_of_instructions > idx);

        assert_eq!(
            self.imm_19[idx], imm_19,
            "invalid imm_19: have {:04b} want {:04b}",
            self.imm_19[idx], imm_19
        );
        assert_eq!(
            self.imm_15[idx], imm_15,
            "invalid imm_15: have {:04b} want {:04b}",
            self.imm_15[idx], imm_15
        );
        assert_eq!(
            self.imm_11[idx], imm_11,
            "invalid imm_11: have {:04b} want {:04b}",
            self.imm_11[idx], imm_11
        );
        assert_eq!(
            self.imm_7[idx], imm_7,
            "invalid imm_7: have {:04b} want {:04b}",
            self.imm_7[idx], imm_7
        );
        assert_eq!(
            self.imm_3[idx], imm_3,
            "invalid imm_3: have {:04b} want {:04b}",
            self.imm_3[idx], imm_3
        );
        assert_eq!(
            self.rs2[idx], rs2,
            "invalid rs2: have {:05b} want {:05b}",
            self.rs2[idx], rs2
        );
        assert_eq!(
            self.rs1[idx], rs1,
            "invalid rs1: have {:05b} want {:05b}",
            self.rs1[idx], rs1
        );
        assert_eq!(
            self.rd[idx], rd,
            "invalid rd: have {:05b} want {:05b}",
            self.rd[idx], rd
        );
        assert_eq!(
            self.rd_w[idx], rd_w,
            "invalid rd_w: have {} want {}",
            self.rd_w[idx], rd_w
        );
        assert_eq!(
            self.mem_w[idx], mem_w,
            "invalid mem_w: have {} want {}",
            self.mem_w[idx], mem_w
        );
        assert_eq!(
            self.pc_w[idx], pc_w,
            "invalid pc_w: have {} want {}",
            self.pc_w[idx], pc_w
        );
    }
}

#[inline(always)]
fn extract(x: u32, end: usize, start: usize) -> u32 {
    let mask: u32 = (1 << (end - start + 1)) - 1;
    (x >> start as u32) & mask
}

pub fn encode_1100011(imm_11: u8, imm_7: u8, imm_3: u8, rs2: u8, rs1: u8, id: u8) -> u32 {
    // imm[12|10:5] | rs2[24:20] | rs1[19:15] | id | imm[4:1|11] | 11000 | 11
    // imm = 0b101101111001 | 0
    // 1) split [12] [11] [10:5] [4:1]: 1  110111 1001
    // 2) Permute [12]  [10:5] [4:1] [11]: 1 110111 1001 0
    let mut rv32: u32 = (imm_11 >> 3) as u32; // imm[12]
    rv32 = (rv32 << 2) | (imm_11 & 0b11) as u32; // imm[10:9]
    rv32 = (rv32 << 4) | imm_7 as u32; // imm[8:5]
    rv32 = (rv32 << 5) | rs2 as u32;
    rv32 = (rv32 << 5) | rs1 as u32;
    rv32 = (rv32 << 3) | id as u32;
    rv32 = (rv32 << 4) | imm_3 as u32;
    rv32 = (rv32 << 1) | ((imm_11 >> 2) & 1) as u32;
    rv32 = (rv32 << 5) | 0b11000;
    rv32 = (rv32 << 2) | 0b11;
    rv32
}

pub fn decode_1100011(instruction: u32) -> (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) {
    debug_assert_eq!(extract(instruction, 6, 0), 0b1100011);

    let offset_12: u32 = extract(instruction, 31, 31);
    let offset_11: u32 = extract(instruction, 7, 7);
    let offset_10_5: u32 = extract(instruction, 30, 25);
    let offset_4_1: u32 = extract(instruction, 11, 8);

    let offset: u32 = ((offset_12 as u32) << 11
        | (offset_11 as u32) << 10
        | (offset_10_5 as u32) << 4
        | (offset_4_1 as u32)) as u32;

    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = extract(offset, 11, 8) as u8;
    let imm_7: u8 = extract(offset, 7, 4) as u8;
    let imm_3: u8 = extract(offset, 3, 0) as u8;
    let rs2: u8 = extract(instruction, 24, 20) as u8;
    let rs1: u8 = extract(instruction, 19, 15) as u8;
    let rd: u8 = 0;
    let pc_w: u8;
    let rd_w: u8 = 0;
    let mem_w: u8 = 0;

    let hi: u32 = extract(instruction, 14, 12);

    match hi {
        0b000 => pc_w = 3, // beq  | 000 | 3 |
        0b001 => pc_w = 4, // bne  | 001 | 4 |
        0b100 => pc_w = 5, // blt  | 100 | 5 |
        0b101 => pc_w = 6, // bge  | 101 | 6 |
        0b110 => pc_w = 7, // bltu | 110 | 7 |
        0b111 => pc_w = 8, // bgeu | 111 | 8 |
        _ => panic!(
            "invalid instruction {:032b} -> unrecognized [14:12]: {:05b}",
            instruction, hi
        ),
    }
    (
        imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    )
}

pub fn encode_0110011(sign: u8, rs2: u8, rs1: u8, id: u8, rd: u8) -> u32 {
    let mut rv32: u32 = sign as u32;
    rv32 = (rv32 << 2) | 0b00;
    rv32 = (rv32 << 5) | rs2 as u32;
    rv32 = (rv32 << 5) | rs1 as u32;
    rv32 = (rv32 << 3) | id as u32;
    rv32 = (rv32 << 5) | rd as u32;
    rv32 = (rv32 << 5) | 0b01100;
    rv32 = (rv32 << 2) | 0b11;
    rv32
}

pub fn decode_0110011(instruction: u32) -> (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) {
    debug_assert_eq!(extract(instruction, 6, 0), 0b0110011);

    #[cfg(debug_assertions)]
    {
        let null: u32 = extract(instruction, 26, 25);
        debug_assert_eq!(null, 0b00, "invalid instruction: {:032b} -> parsed as [add, sub, sll, slt, sltu, xor, slr, sra, or, and] but invalid [26:25]: {:02b}", instruction, null);
    }

    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0;
    let imm_7: u8 = 0;
    let imm_3: u8 = 0;
    let rs2: u8 = extract(instruction, 24, 20) as u8;
    let rs1: u8 = extract(instruction, 19, 15) as u8;
    let rd: u8 = extract(instruction, 11, 7) as u8;
    let rd_w: u8;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let hi: u32 = extract(instruction, 14, 12);
    let sign: u32 = extract(instruction, 31, 27);
    match hi {
        0b000 => {
            match sign{
                0b00000=> rd_w = 12, // add  | 00000 | 000 | 12 |
                0b01000=> rd_w = 13, // sub  | 01000 | 000 | 13 |
                _=>panic!("invalid instruction: {:032b} -> parsed as add or sub but invalid [31:27]: {:05b}", instruction, sign),
            }
        }
        0b001 => rd_w = 14, // sll  | 00000 | 001 | 14 |
        0b010 => rd_w = 15, // slt  | 00000 | 010 | 15 |
        0b011 => rd_w = 16, // sltu | 00000 | 011 | 16 |
        0b100 => rd_w = 17, // xor  | 00000 | 100 | 17 |
        0b101 => {
            match sign{
                0b00000=> rd_w = 18, // srl  | 00000 | 101 | 18 |
                0b01000=> rd_w = 19, // sra  | 01000 | 101 | 19 |
                _=>panic!("invalid instruction: {:032b} -> parsed as srl or sra but invalid [31:27]: {:05b}", instruction, sign),
            }
        }
        0b110 => rd_w = 20, // or   | 00000 | 110 | 20 |
        0b111 => rd_w = 21, // and  | 00000 | 111 | 21 |
        _ => panic!(
            "invalid instruction: {:032b} -> unrecognized [14-12]: {:03b}",
            instruction, hi
        ),
    }

    (
        imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    )
}

pub fn encode_0010011(imm_11: u8, imm_7: u8, imm_3: u8, rs1: u8, id: u8, rd: u8) -> u32 {
    let mut rv32: u32 = imm_11 as u32;
    rv32 = (rv32 << 4) | imm_7 as u32;
    rv32 = (rv32 << 4) | imm_3 as u32;
    rv32 = (rv32 << 5) | rs1 as u32;
    rv32 = (rv32 << 3) | id as u32;
    rv32 = (rv32 << 5) | rd as u32;
    rv32 = (rv32 << 5) | 0b00100;
    rv32 = (rv32 << 2) | 0b11;
    rv32
}

pub fn decode_0010011(instruction: u32) -> (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) {
    debug_assert_eq!(extract(instruction, 6, 0), 0b0010011);

    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8;
    let imm_7: u8 = extract(instruction, 27, 24) as u8;
    let imm_3: u8 = extract(instruction, 23, 20) as u8;
    let rs2: u8 = 0;
    let rs1: u8 = extract(instruction, 19, 15) as u8;
    let rd: u8 = extract(instruction, 11, 7) as u8;
    let rd_w: u8;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let hi: u32 = extract(instruction, 14, 12);

    match hi {
        0b101 => {
            imm_11 = 0;
            let sign: u32 = extract(instruction, 31, 27);

            #[cfg(debug_assertions)]
            {
                let null: u32 = extract(instruction, 26, 25);
                debug_assert_eq!(null, 0, "invalid instruction: {:032b} -> parsed as srli or srai but invalid [26:25]: {:05b} should be 00", instruction, null);
            }
            match sign{
                0b00000 => rd_w = 10, // slri  | 00000| 101 | 10 |
                0b01000 => rd_w = 11, // srai  | 01000| 101 | 11 |
                _=>panic!("invalid instruction: {:032b} -> parsed as slri or srai but invalid [31:27]: {:05b}", instruction, sign),
            }
        }

        _ => {
            imm_11 = extract(instruction, 31, 28) as u8;
            match hi {
                0b000 => rd_w = 3, // addi  | 00000| 000 |  3 |
                0b010 => rd_w = 4, // slti  | 00000| 010 |  4 |
                0b011 => rd_w = 5, // sltiu | 00000| 011 |  5 |
                0b100 => rd_w = 6, // xori  | 00000| 100 |  6 |
                0b110 => rd_w = 7, // ori   | 00000| 110 |  7 |
                0b111 => rd_w = 8, // andi  | 00000| 111 |  8 |
                0b001 => rd_w = 9, // slli  | 00000| 001 |  9 |
                _ => panic!(
                    "invalid instruction: {:032b} -> unrecognized [14-12]: {:03b}",
                    instruction, hi
                ),
            }
        }
    }

    (
        imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    )
}

pub fn encode_0100011(imm_11: u8, imm_7: u8, imm_3: u8, rs2: u8, rs1: u8, id: u8) -> u32 {
    let mut rv32: u32 = imm_11 as u32;
    rv32 = (rv32 << 3) | (imm_7 as u32) >> 1;
    rv32 = (rv32 << 5) | rs2 as u32;
    rv32 = (rv32 << 5) | rs1 as u32;
    rv32 = (rv32 << 3) | id as u32;
    rv32 = (rv32 << 5) | ((imm_7 as u32) & 1) << 4 | imm_3 as u32;
    rv32 = (rv32 << 5) | 0b01000;
    rv32 = (rv32 << 2) | 0b11;
    rv32
}

pub fn decode_0100011(instruction: u32) -> (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) {
    debug_assert_eq!(extract(instruction, 6, 0), 0b0100011);

    let offset_11_5: u32 = extract(instruction, 31, 25);
    let offset_4_0: u32 = extract(instruction, 11, 7);
    let offset: u32 = ((offset_11_5 as u32) << 5) | offset_4_0 as u32;

    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = extract(offset, 11, 8) as u8;
    let imm_7: u8 = extract(offset, 7, 4) as u8;
    let imm_3: u8 = extract(offset, 3, 0) as u8;
    let rs2: u8 = extract(instruction, 24, 20) as u8;
    let rs1: u8 = extract(instruction, 19, 15) as u8;
    let rd: u8 = 0;
    let rd_w: u8 = 0;
    let mem_w: u8;
    let pc_w: u8 = 0;

    let hi: u32 = extract(instruction, 14, 12);
    match hi {
        0b000 => mem_w = 1, // sb | 000 | 1 |
        0b001 => mem_w = 2, // sh | 001 | 2 |
        0b010 => mem_w = 3, // sw | 010 | 3 |
        _ => panic!(
            "invalid instruction {:032b} -> unrecognized [14:12]: {:05b}",
            instruction, hi
        ),
    }
    (
        imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    )
}

pub fn encode_1101111(imm_19: u8, imm_15: u8, imm_11: u8, imm_7: u8, imm_3: u8, rd: u8) -> u32 {
    let mut rv32: u32 = (imm_19 >> 3) as u32; // imm[20]
    rv32 = (rv32 << 2) | (imm_11 & 0b11) as u32; // imm[10:9]
    rv32 = (rv32 << 4) | (imm_7 as u32); // imm[8:5]
    rv32 = (rv32 << 4) | (imm_3 as u32); // imm[4:1]
    rv32 = (rv32 << 1) | ((imm_11 >> 2) & 1) as u32; // imm[11]
    rv32 = (rv32 << 3) | (imm_19 & 0b111) as u32; // imm[19:17]
    rv32 = (rv32 << 4) | imm_15 as u32; // imm[16:13]
    rv32 = (rv32 << 1) | (imm_11 >> 3) as u32; // imm[12:12]
    rv32 = (rv32 << 5) | rd as u32;
    rv32 = (rv32 << 5) | 0b11011;
    rv32 = (rv32 << 2) | 0b11;
    rv32
}

pub fn decode_1101111(instruction: u32) -> (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) {
    debug_assert_eq!(extract(instruction, 6, 0), 0b1101111);

    let offset_20: u32 = extract(instruction, 31, 31);
    let offset_19_12: u32 = extract(instruction, 19, 12);
    let offset_11: u32 = extract(instruction, 20, 20);
    let offset_10_1: u32 = extract(instruction, 30, 21);

    let offset: u32 = ((offset_20 as u32) << 19)
        | (offset_19_12 as u32) << 11
        | (offset_11 as u32) << 10
        | (offset_10_1 as u32);

    let imm_19: u8 = extract(offset, 19, 16) as u8;
    let imm_15: u8 = extract(offset, 15, 12) as u8;
    let imm_11: u8 = extract(offset, 11, 8) as u8;
    let imm_7: u8 = extract(offset, 7, 4) as u8;
    let imm_3: u8 = extract(offset, 3, 0) as u8;
    let rs2: u8 = 0;
    let rs1: u8 = 0;
    let rd: u8 = extract(instruction, 11, 7) as u8;
    let rd_w: u8 = 27;
    let mem_w: u8 = 0;
    let pc_w: u8 = 1;
    (
        imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    )
}

pub fn encode_1100111(imm_11: u8, imm_7: u8, imm_3: u8, rs1: u8, id: u8, rd: u8) -> u32 {
    let mut rv32: u32 = imm_11 as u32;
    rv32 = (rv32 << 4) | imm_7 as u32;
    rv32 = (rv32 << 4) | imm_3 as u32;
    rv32 = (rv32 << 5) | rs1 as u32;
    rv32 = (rv32 << 3) | id as u32;
    rv32 = (rv32 << 5) | rd as u32;
    rv32 = (rv32 << 5) | 0b11001;
    rv32 = (rv32 << 2) | 0b11;
    rv32
}

pub fn decode_1100111(instruction: u32) -> (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) {
    debug_assert_eq!(extract(instruction, 6, 0), 0b1100111);

    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = extract(instruction, 31, 28) as u8;
    let imm_7: u8 = extract(instruction, 27, 24) as u8;
    let imm_3: u8 = extract(instruction, 23, 20) as u8;
    let rs2: u8 = 0;
    let rs1: u8 = extract(instruction, 19, 15) as u8;
    let rd: u8 = extract(instruction, 11, 7) as u8;
    let rd_w: u8 = 28;
    let mem_w: u8 = 0;
    let pc_w: u8 = 2;
    (
        imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    )
}

pub fn encode_0010111(imm_19: u8, imm_15: u8, imm_11: u8, imm_7: u8, imm_3: u8, rd: u8) -> u32 {
    let mut rv32: u32 = imm_19 as u32;
    rv32 = (rv32 << 4) | imm_15 as u32;
    rv32 = (rv32 << 4) | imm_11 as u32;
    rv32 = (rv32 << 4) | imm_7 as u32;
    rv32 = (rv32 << 4) | imm_3 as u32;
    rv32 = (rv32 << 5) | rd as u32;
    rv32 = (rv32 << 5) | 0b00101;
    rv32 = (rv32 << 2) | 0b11;
    rv32
}

pub fn decode_0010111(instruction: u32) -> (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) {
    debug_assert_eq!(extract(instruction, 6, 0), 0b0010111);

    let imm_19: u8 = extract(instruction, 31, 28) as u8;
    let imm_15: u8 = extract(instruction, 27, 24) as u8;
    let imm_11: u8 = extract(instruction, 23, 20) as u8;
    let imm_7: u8 = extract(instruction, 19, 16) as u8;
    let imm_3: u8 = extract(instruction, 15, 12) as u8;
    let rs2: u8 = 0;
    let rs1: u8 = 0;
    let rd: u8 = extract(instruction, 11, 7) as u8;
    let rd_w: u8 = 2;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;
    (
        imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    )
}

pub fn encode_0000011(imm_11: u8, imm_7: u8, imm_3: u8, rs1: u8, id: u8, rd: u8) -> u32 {
    let mut rv32: u32 = imm_11 as u32;
    rv32 = (rv32 << 4) | imm_7 as u32;
    rv32 = (rv32 << 4) | imm_3 as u32;
    rv32 = (rv32 << 5) | rs1 as u32;
    rv32 = (rv32 << 3) | id as u32;
    rv32 = (rv32 << 5) | rd as u32;
    rv32 = (rv32 << 5) | 0b00000;
    rv32 = (rv32 << 2) | 0b11;
    rv32
}

pub fn decode_0000011(instruction: u32) -> (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) {
    debug_assert_eq!(extract(instruction, 6, 0), 0b0000011);

    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = extract(instruction, 31, 28) as u8;
    let imm_7: u8 = extract(instruction, 27, 24) as u8;
    let imm_3: u8 = extract(instruction, 23, 20) as u8;
    let rs2: u8 = 0;
    let rs1: u8 = extract(instruction, 19, 15) as u8;
    let rd: u8 = extract(instruction, 11, 7) as u8;
    let rd_w: u8;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let hi: u32 = extract(instruction, 14, 12);

    match hi {
        0b000 => rd_w = 22, // lb   | 000 | 22 |
        0b001 => rd_w = 23, // lh   | 001 | 23 |
        0b010 => rd_w = 24, // lw   | 010 | 24 |
        0b100 => rd_w = 25, // lbu  | 100 | 25 |
        0b101 => rd_w = 26, // lhu  | 101 | 26 |
        _ => panic!(
            "invalid instruction: {:032b} -> unrecognized [14-12]: {:03b}",
            instruction, hi
        ),
    }
    (
        imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    )
}

pub fn encode_0110111(imm_19: u8, imm_15: u8, imm_11: u8, imm_7: u8, imm_3: u8, rd: u8) -> u32 {
    let mut rv32: u32 = imm_19 as u32;
    rv32 = (rv32 << 4) | imm_15 as u32;
    rv32 = (rv32 << 4) | imm_11 as u32;
    rv32 = (rv32 << 4) | imm_7 as u32;
    rv32 = (rv32 << 4) | imm_3 as u32;
    rv32 = (rv32 << 5) | rd as u32;
    rv32 = (rv32 << 5) | 0b01101;
    rv32 = (rv32 << 2) | 0b11;
    rv32
}

pub fn decode_0110111(instruction: u32) -> (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) {
    debug_assert_eq!(extract(instruction, 6, 0), 0b0110111);

    let imm_19: u8 = extract(instruction, 31, 28) as u8;
    let imm_15: u8 = extract(instruction, 27, 24) as u8;
    let imm_11: u8 = extract(instruction, 23, 20) as u8;
    let imm_7: u8 = extract(instruction, 19, 16) as u8;
    let imm_3: u8 = extract(instruction, 15, 12) as u8;
    let rs2: u8 = 0;
    let rs1: u8 = 0;
    let rd: u8 = extract(instruction, 11, 7) as u8;
    let rd_w: u8 = 1;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;
    (
        imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    )
}
