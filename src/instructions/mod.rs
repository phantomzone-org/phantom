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

#[non_exhaustive]
pub struct OpID;

impl OpID {
    pub const LUI: (u8, u8, u8) = (1, 0, 0);
    pub const AUIPC: (u8, u8, u8) = (2, 0, 0);
    pub const ADDI: (u8, u8, u8) = (3, 0, 0);
    pub const SLTI: (u8, u8, u8) = (4, 0, 0);
    pub const SLTIU: (u8, u8, u8) = (5, 0, 0);
    pub const XORI: (u8, u8, u8) = (6, 0, 0);
    pub const ORI: (u8, u8, u8) = (7, 0, 0);
    pub const ANDI: (u8, u8, u8) = (8, 0, 0);
    pub const SLLI: (u8, u8, u8) = (9, 0, 0);
    pub const SRLI: (u8, u8, u8) = (10, 0, 0);
    pub const SRAI: (u8, u8, u8) = (11, 0, 0);
    pub const ADD: (u8, u8, u8) = (12, 0, 0);
    pub const SUB: (u8, u8, u8) = (13, 0, 0);
    pub const SLL: (u8, u8, u8) = (14, 0, 0);
    pub const SLT: (u8, u8, u8) = (15, 0, 0);
    pub const SLTU: (u8, u8, u8) = (16, 0, 0);
    pub const XOR: (u8, u8, u8) = (17, 0, 0);
    pub const SRL: (u8, u8, u8) = (18, 0, 0);
    pub const SRA: (u8, u8, u8) = (19, 0, 0);
    pub const OR: (u8, u8, u8) = (20, 0, 0);
    pub const AND: (u8, u8, u8) = (21, 0, 0);
    pub const LB: (u8, u8, u8) = (22, 0, 0);
    pub const LH: (u8, u8, u8) = (23, 0, 0);
    pub const LW: (u8, u8, u8) = (24, 0, 0);
    pub const LBU: (u8, u8, u8) = (25, 0, 0);
    pub const LHU: (u8, u8, u8) = (26, 0, 0);
    pub const SB: (u8, u8, u8) = (0, 1, 0);
    pub const SH: (u8, u8, u8) = (0, 2, 0);
    pub const SW: (u8, u8, u8) = (0, 3, 0);
    pub const JAL: (u8, u8, u8) = (27, 0, 1);
    pub const JALR: (u8, u8, u8) = (28, 0, 2);
    pub const BEQ: (u8, u8, u8) = (0, 0, 3);
    pub const BNE: (u8, u8, u8) = (0, 0, 4);
    pub const BLT: (u8, u8, u8) = (0, 0, 5);
    pub const BGE: (u8, u8, u8) = (0, 0, 6);
    pub const BLTU: (u8, u8, u8) = (0, 0, 7);
    pub const BGEU: (u8, u8, u8) = (0, 0, 8);
}

pub struct Instructions {
    imm_31: Vec<u8>,
    imm_27: Vec<u8>,
    imm_23: Vec<u8>,
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
            imm_31: Vec::new(),
            imm_27: Vec::new(),
            imm_23: Vec::new(),
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
        let i: Instruction = Instruction::new(instruction);
        let (imm_31, imm_27, imm_23, imm_19, imm_15, imm_11, imm_7, imm_3) = i.decode_immediate();
        let (rs2, rs1, rd) = i.decode_registers();
        let (rd_w, mem_w, pc_w) = i.decode_opcode();
        self.imm_31.push(imm_31);
        self.imm_27.push(imm_27);
        self.imm_23.push(imm_23);
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
        imm_31: u8,
        imm_27: u8,
        imm_23: u8,
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
            self.imm_31[idx], imm_31,
            "invalid imm_31: have {:04b} want {:04b}",
            self.imm_31[idx], imm_31
        );

        assert_eq!(
            self.imm_27[idx], imm_27,
            "invalid imm_27: have {:04b} want {:04b}",
            self.imm_27[idx], imm_27
        );

        assert_eq!(
            self.imm_23[idx], imm_23,
            "invalid imm_19: have {:04b} want {:04b}",
            self.imm_23[idx], imm_23
        );

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

pub enum Instruction {
    R(u32),
    I(u32),
    S(u32),
    B(u32),
    U(u32),
    J(u32),
}

impl Instruction {
    #[inline(always)]
    pub fn new(instruction: u32) -> Instruction {
        match instruction & 0x7f {
            0b0110011 => Instruction::R(instruction),

            // Type I-immedate:
            // [31-11] [10- 5] [4 - 1] [ 0]
            // [  31 ] [30:25] [24:21] [20]
            0b0010011 | 0b0000011 => Instruction::I(instruction),

            // Type S-immediate
            //
            // [31-11] [10- 5] [4 - 1] [ 0]
            // [  31 ] [30:25] [11: 8] [ 7]
            0b0100011 => Instruction::S(instruction),

            // Type B-immediate
            //
            // [31-12] [11] [10- 5] [4- 1] [0]
            // [  31 ] [ 7] [30:25] [11:8] [-]
            0b1100011 => Instruction::B(instruction),

            // Type U-immediate
            //
            // [31-12] [11-0]
            // [31:12] [  - ]
            0b0110111 | 0b0010111 | 0b1100111 => Instruction::U(instruction),

            // Type J-immediate
            //
            // [31-20] [19-12] [11] [10- 5] [4 - 1] [0]
            // [  31 ] [19:12] [20] [30:25] [24:21] [-]
            0b1101111 => Instruction::J(instruction),

            _ => {
                panic!(
                    "invalid instruction {:032b} -> unrecoginzed opcode: {:07b}",
                    instruction,
                    instruction & 0x7f
                )
            }
        }
    }

    #[inline(always)]
    pub fn decode_registers(&self) -> (u8, u8, u8) {
        match self {
            Instruction::R(instruction) => (
                ((instruction >> 20) & 0x1f) as u8,
                ((instruction >> 15) & 0x1f) as u8,
                ((instruction >> 7) & 0x1f) as u8,
            ),
            Instruction::I(instruction) => (
                0,
                ((instruction >> 15) & 0x1f) as u8,
                ((instruction >> 7) & 0x1f) as u8,
            ),
            Instruction::S(instruction) | Instruction::B(instruction) => (
                ((instruction >> 20) & 0x1f) as u8,
                ((instruction >> 15) & 0x1f) as u8,
                0,
            ),
            Instruction::U(instruction) | Instruction::J(instruction) => {
                (0, 0, ((instruction >> 15) & 0x1f) as u8)
            }
        }
    }

    #[inline(always)]
    pub fn decode_opcode(&self) -> (u8, u8, u8) {
        match self {
            Instruction::R(instruction) => {
                let funct7: u32 = (instruction >> 25) & 0x7f;
                let funct3: u32 = (instruction >> 12) & 0x7;
                match funct7 << 7 | funct3 {
                    0b0000000000 => OpID::ADD,
                    0b0100000000 => OpID::SUB,
                    0b0000000001 => OpID::SLL,
                    0b0000000010 => OpID::SLT,
                    0b0000000011 => OpID::SLTU,
                    0b0000000100 => OpID::XOR,
                    0b0000000101 => OpID::SRL,
                    0b0100000101 => OpID::SRA,
                    0b0000000110 => OpID::OR,
                    0b0000000111 => OpID::AND,
                    _ => panic!("invalid instruction R-type: {:032b}", instruction),
                }
            }
            Instruction::I(instruction) => {
                let opcode: u32 = instruction & 0x7F;
                let funct3: u32 = (instruction >> 12) & 0x7;
                match opcode {
                    0b0010011 => {
                        let funct7: u32 = (instruction >> 25) & 0x7f;
                        match (funct7, funct3) {
                            (_, 0b000) => OpID::ADDI,
                            (_, 0b010) => OpID::SLTI,
                            (_, 0b011) => OpID::SLTIU,
                            (_, 0b100) => OpID::XORI,
                            (_, 0b110) => OpID::ORI,
                            (_, 0b111) => OpID::ANDI,
                            (_, 0b001) => OpID::SLLI,
                            (0b00000, 0b101) => OpID::SRLI,
                            (0b01000, 0b101) => OpID::SRAI,
                            _ => panic!(
                                "invalid instruction, parsed as I-type: {:032b}",
                                instruction
                            ),
                        }
                    }
                    0b0000011 => match funct3 {
                        0b000 => OpID::LB,
                        0b001 => OpID::LH,
                        0b010 => OpID::LW,
                        0b100 => OpID::LBU,
                        0b101 => OpID::LHU,
                        _ => panic!(
                            "invalid instruction, parsed as I-type: {:032b}",
                            instruction
                        ),
                    },
                    _ => panic!(
                        "invalid instruction, parsed as I-type: {:032b}",
                        instruction
                    ),
                }
            }
            Instruction::S(instruction) => {
                let funct3: u32 = (instruction >> 12) & 0x7;
                match funct3 {
                    0b000 => OpID::SB,
                    0b001 => OpID::SH,
                    0b010 => OpID::SW,
                    _ => panic!(
                        "invalid instruction, parsed as S-type: {:032b}",
                        instruction
                    ),
                }
            }
            Instruction::B(instruction) => {
                let funct3: u32 = (instruction >> 12) & 0x7;
                match funct3 {
                    0b000 => OpID::BEQ,
                    0b001 => OpID::BNE,
                    0b100 => OpID::BLT,
                    0b101 => OpID::BGE,
                    0b110 => OpID::BLTU,
                    0b111 => OpID::BGEU,
                    _ => panic!(
                        "invalid instruction, parsed as B-type: {:032b}",
                        instruction
                    ),
                }
            }
            Instruction::U(instruction) => {
                let opid: u32 = instruction & 0x7f;
                match opid {
                    0b0110111 => OpID::LUI,
                    0b1100111 => OpID::JALR,
                    0b0010111 => OpID::AUIPC,
                    _ => panic!(
                        "invalid instruction, parsed as U-type: {:032b}",
                        instruction
                    ),
                }
            }
            Instruction::J(_) => OpID::JAL,
        }
    }

    #[inline(always)]
    pub fn decode_immediate(&self) -> (u8, u8, u8, u8, u8, u8, u8, u8) {
        let immediate: u32;

        match self {
            Instruction::R(_) => immediate = 0,

            // Type I-immedate:
            // [31-11] [10- 5] [4 - 1] [ 0]
            // [  31 ] [30:25] [24:21] [20]
            Instruction::I(data) => immediate = (data >> 20) | ((data >> 31) & 1) * 0xFFFF_F000,

            // Type S-immediate
            //
            // [31-11] [10- 5] [4 - 1] [ 0]
            // [  31 ] [30:25] [11: 8] [ 7]
            Instruction::S(data) => {
                immediate = (data >> 20) & 0x0000_07E0
                    | (data >> 7) & 0x0000_001F
                    | ((data >> 31) & 1) * 0xFFFF_F000
            }

            // Type B-immediate
            //
            // [31-12] [11] [10- 5] [4- 1] [0]
            // [  31 ] [ 7] [30:25] [11:8] [-]
            Instruction::B(data) => {
                immediate = (data << 4) & 0x0000_0800
                    | (data >> 20) & 0x0000_07E0
                    | (data >> 7) & 0x0000_001E
                    | ((data >> 31) & 1) * 0xFFFF_F000
            }

            // Type U-immediate
            //
            // [31-12] [11-0]
            // [31:12] [  - ]
            Instruction::U(data) => immediate = data & 0xFFFF_F000,

            // Type J-immediate
            //
            // [31-20] [19-12] [11] [10- 5] [4 - 1] [0]
            // [  31 ] [19:12] [20] [30:25] [24:21] [-]
            Instruction::J(data) => {
                immediate = data & 0xff000
                    | (data >> 9) & 0x0000_0800
                    | (data >> 20) & 0x0000_07E0
                    | (data >> 20) & 0x0000_001E
                    | ((data >> 31) & 1) * 0xFFFF_F000
            }
        }

        (
            ((immediate >> 28) & 0xFF) as u8,
            ((immediate >> 24) & 0xFF) as u8,
            ((immediate >> 20) & 0xFF) as u8,
            ((immediate >> 16) & 0xFF) as u8,
            ((immediate >> 12) & 0xFF) as u8,
            ((immediate >> 8) & 0xFF) as u8,
            ((immediate >> 4) & 0xFF) as u8,
            ((immediate >> 0) & 0xFF) as u8,
        )
    }

    pub fn encode_immediate(&mut self, immediate: u32) {
        match self {
            Instruction::R(_) => {
                panic!("invalid instruction: cannot encode immediate on type R")
            }

            // [31-20]
            // [11: 0]
            Instruction::I(data) => *data = (*data & 0x000F_FFFF) | (immediate << 20),

            // [31-25] [11-7]
            // [11: 5] [4: 0]
            Instruction::S(data) => {
                *data = (*data & 0x01FF_F07F)
                    | (immediate << 20) & 0xFE00_0000
                    | (immediate << 6) & 0x0000_0F80
            }

            // [31] [30-25] [11-8] [ 7]
            // [12] [10: 5] [4: 1] [11]
            Instruction::B(data) => {
                *data = (*data & 0x01FF_F07F)
                    | (immediate << 19) & 0x8000_0000
                    | (immediate << 21) & 0x7E00_0000
                    | (immediate << 6) & 0x0000_0F00
                    | (immediate >> 5) & 0x0000_0080
            }

            Instruction::U(data) => *data = (*data & 0x0000_0FFF) | (immediate & 0xFFFF_F000),

            Instruction::J(data) => {
                *data = (*data & 0x0000_0FFF)
                    | (immediate << 12) & 0x8000_0000
                    | (immediate << 20) & 0x7FE0_0000
                    | (immediate << 9) & 0x0010_0000
                    | (immediate >> 12) & 0x000F_F000
            }
        }
    }
}
