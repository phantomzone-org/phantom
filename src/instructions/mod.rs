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

pub fn reconstruct(x: &[u8; 8]) -> u32 {
    let mut y: u32 = 0;
    y |= (x[7] as u32) << 28;
    y |= (x[6] as u32) << 24;
    y |= (x[5] as u32) << 20;
    y |= (x[4] as u32) << 16;
    y |= (x[3] as u32) << 12;
    y |= (x[2] as u32) << 8;
    y |= (x[1] as u32) << 4;
    y |= (x[0] as u32);
    y
}

pub fn decompose(x: u32) -> [u8; 8] {
    let mut y: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    y[0] = ((x >> 0) & 0xF) as u8;
    y[1] = ((x >> 4) & 0xF) as u8;
    y[2] = ((x >> 8) & 0xF) as u8;
    y[3] = ((x >> 12) & 0xF) as u8;
    y[4] = ((x >> 16) & 0xF) as u8;
    y[5] = ((x >> 20) & 0xF) as u8;
    y[6] = ((x >> 24) & 0xF) as u8;
    y[7] = ((x >> 28) & 0xF) as u8;
    y
}

pub fn sext(x: u32, bits: usize) -> u32 {
    x | ((x >> bits) & 1) * (0xFFFF_FFFF & (0xFFFF_FFFF << bits))
}

#[allow(dead_code)]
fn test_r_type(funct7: u8, funct3: u8, op_code: u8, opid: (u8, u8, u8)) {
    // 00000 | 00 | rs2[24:20] | rs1[19:15] | funct3 | rd[11:7] |
    let imm: u32 = 0;
    let rs2: u8 = 0b11011;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_funct3(funct3);
    instruction.encode_funct7(funct7);
    instruction.encode_rs2(rs2);
    instruction.encode_rs1(rs1);
    instruction.encode_rd(rd);
    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(0, decompose(imm), rs2, rs1, rd, opid.0, opid.1, opid.2);
}

#[allow(dead_code)]
fn test_i_type(funct3: u8, op_code: u8, opid: (u8, u8, u8)) {
    // imm[31:20] | rs1[19:15] | funct3 | rd[11:7] | op_code
    // imm[11: 0]
    let funct3: u8 = funct3;
    let imm: u32 = 0xABC;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_immediate(imm);
    instruction.encode_funct3(funct3);
    instruction.encode_rs1(rs1);
    instruction.encode_rd(rd);
    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0,
        decompose(sext(imm, 11)),
        rs2,
        rs1,
        rd,
        opid.0,
        opid.1,
        opid.2,
    );
}

#[allow(dead_code)]
fn test_i_shamt_type(imm: u32, funct3: u8, opid: (u8, u8, u8)) {
    // 0000000 | shamt[24:20] | rs1[19:15] | funct3 | rd[11:7] | 0010011
    let op_code: u8 = 0b0010011;
    let funct3: u8 = funct3;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_immediate(imm);
    instruction.encode_funct3(funct3);
    instruction.encode_rs1(rs1);
    instruction.encode_rd(rd);
    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0,
        decompose(imm & 0x1F),
        rs2,
        rs1,
        rd,
        opid.0,
        opid.1,
        opid.2,
    );
}

#[allow(dead_code)]
fn test_s_type(funct3: u8, op_code: u8, op_id: (u8, u8, u8)) {
    // imm[11:5] | rs2[24:20] | rs1[19:15] | 000 | imm[4:0] | 0100011
    let imm: u32 = 0xABC;
    let rs2: u8 = 0b11011;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_immediate(imm);
    instruction.encode_funct3(funct3);
    instruction.encode_rs2(rs2);
    instruction.encode_rs1(rs1);
    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0,
        decompose(sext(imm, 12)),
        rs2,
        rs1,
        rd,
        op_id.0,
        op_id.1,
        op_id.1,
    );
}

#[allow(dead_code)]
fn test_b_type(funct3: u8, op_code: u8, op_id: (u8, u8, u8)) {
    let imm: u32 = 0xABC<<1;
    let rs2: u8 = 0b11011;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_immediate(imm);
    instruction.encode_funct3(funct3);
    instruction.encode_rs2(rs2);
    instruction.encode_rs1(rs1);
    let mut m: Instructions = Instructions::new();
    println!("instruction: {:032b}", instruction.get());
    println!("imm : {:032b}", imm);
    println!("sext: {:032b}", sext(imm, 12));
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0,
        decompose(sext(imm, 12)),
        rs2,
        rs1,
        rd,
        op_id.0,
        op_id.1,
        op_id.2,
    );
}

#[allow(dead_code)]
fn test_u_type(op_code: u8, op_id: (u8, u8, u8)) {
    let imm: u32 = 0xABCD_E000;
    let rs2: u8 = 0;
    let rs1: u8 = 0;
    let rd: u8 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_immediate(imm);
    instruction.encode_rd(rd);
    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(0, decompose(imm), rs2, rs1, rd, op_id.0, op_id.1, op_id.2);
}

#[allow(dead_code)]
fn test_j_type(op_code: u8, op_id: (u8, u8, u8)) {
    let imm: u32 = 0xABCDE << 1;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_immediate(imm);
    instruction.encode_rd(rd);
    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(0, decompose(imm), rs2, rs1, rd, op_id.0, op_id.1, op_id.2);
}

pub trait Arithmetic {
    fn apply(&self, _imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8];
}

pub trait PcUpdates {
    fn apply(
        &self,
        imm: &[u8; 8],
        x_rs1: &[u8; 8],
        x_rs2: &[u8; 8],
        pc: &[u8; 8],
    ) -> ([u8; 8], [u8; 8]);
}

pub trait Store {
    fn apply(
        &self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm: &[u8; 8],
        x_rs1: &[u8; 8],
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
        imm: &[u8; 8],
        x_rs1: &[u8; 8],
        memory: &mut Memory,
        circuit_btp: &CircuitBootstrapper,
        address: &mut Address,
        tmp_bytes: &mut [u8],
    ) -> [u8; 8];
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

    pub fn add(&mut self, instruction: Instruction) {
        let imm_u8: [u8; 8] = decompose(instruction.decode_immediate());
        let (rs2, rs1, rd) = instruction.decode_registers();
        let (rd_w, mem_w, pc_w) = instruction.decode_opcode();
        self.imm_31.push(imm_u8[7]);
        self.imm_27.push(imm_u8[6]);
        self.imm_23.push(imm_u8[5]);
        self.imm_19.push(imm_u8[4]);
        self.imm_15.push(imm_u8[3]);
        self.imm_11.push(imm_u8[2]);
        self.imm_7.push(imm_u8[1]);
        self.imm_3.push(imm_u8[0]);
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
        imm: [u8; 8],
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
            self.imm_31[idx], imm[7],
            "invalid imm_31: have {:04b} want {:04b}",
            self.imm_31[idx], imm[7]
        );

        assert_eq!(
            self.imm_27[idx], imm[6],
            "invalid imm_27: have {:04b} want {:04b}",
            self.imm_27[idx], imm[6]
        );

        assert_eq!(
            self.imm_23[idx], imm[5],
            "invalid imm_19: have {:04b} want {:04b}",
            self.imm_23[idx], imm[5]
        );

        assert_eq!(
            self.imm_19[idx], imm[4],
            "invalid imm_19: have {:04b} want {:04b}",
            self.imm_19[idx], imm[4]
        );
        assert_eq!(
            self.imm_15[idx], imm[3],
            "invalid imm_15: have {:04b} want {:04b}",
            self.imm_15[idx], imm[3]
        );
        assert_eq!(
            self.imm_11[idx], imm[2],
            "invalid imm_11: have {:04b} want {:04b}",
            self.imm_11[idx], imm[2]
        );
        assert_eq!(
            self.imm_7[idx], imm[1],
            "invalid imm_7: have {:04b} want {:04b}",
            self.imm_7[idx], imm[1]
        );
        assert_eq!(
            self.imm_3[idx], imm[0],
            "invalid imm_3: have {:04b} want {:04b}",
            self.imm_3[idx], imm[0]
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

    pub fn get(&self) -> u32 {
        match self {
            Instruction::R(instruction)
            | Instruction::I(instruction)
            | Instruction::S(instruction)
            | Instruction::B(instruction)
            | Instruction::U(instruction)
            | Instruction::J(instruction) => *instruction,
        }
    }

    #[inline(always)]
    pub fn encode_funct3(&mut self, funct3: u8) {
        match self {
            Instruction::R(instruction)
            | Instruction::I(instruction)
            | Instruction::S(instruction)
            | Instruction::B(instruction) => {
                *instruction = (*instruction & 0xFFFF_8FFF) | ((funct3 & 0x7) as u32) << 12
            }
            _ => panic!("can only encode funct3 on R, I, S and B type instruction"),
        }
    }

    #[inline(always)]
    pub fn encode_funct7(&mut self, funct7: u8) {
        match self {
            Instruction::R(instruction) => {
                *instruction = (*instruction & 0x01FF_FFFF) | ((funct7 & 0x7f) as u32) << 25
            }
            _ => panic!("can only encode funct3 on R type instruction"),
        }
    }

    #[inline(always)]
    pub fn encode_rs2(&mut self, rs2: u8) {
        match self {
            Instruction::R(instruction)
            | Instruction::S(instruction)
            | Instruction::B(instruction) => {
                *instruction = (*instruction & 0xFE0F_FFFF) | ((rs2 & 0x7f) as u32) << 20
            }
            _ => panic!("can only encode rs2 on R, S and B type instruction"),
        }
    }

    #[inline(always)]
    pub fn encode_rs1(&mut self, rs1: u8) {
        match self {
            Instruction::R(instruction)
            | Instruction::I(instruction)
            | Instruction::S(instruction)
            | Instruction::B(instruction) => {
                *instruction = (*instruction & 0xFFF0_7FFF) | ((rs1 & 0x7f) as u32) << 15
            }
            _ => panic!("can only encode rs2 on R, I, S and B type instruction"),
        }
    }

    #[inline(always)]
    pub fn encode_rd(&mut self, rd: u8) {
        match self {
            Instruction::R(instruction)
            | Instruction::I(instruction)
            | Instruction::U(instruction)
            | Instruction::J(instruction) => {
                *instruction = (*instruction & 0xFFFF_F07F) | ((rd & 0x7f) as u32) << 7
            }
            _ => panic!("can only encode rs2 on R, I, U and J type instruction"),
        }
    }

    pub fn encode_shamt(&mut self, shamt: u8) {
        match self {
            Instruction::I(instruction) => {
                *instruction = (*instruction & 0xFE0F_FFFF) | ((shamt & 0x7f) as u32) << 20
            }
            _ => panic!("can only encode shamt on slli, srai, srli"),
        }
    }

    #[inline(always)]
    pub fn encode_immediate(&mut self, immediate: u32) {
        match self {
            Instruction::R(_) => {
                panic!("cannot encode immediate on type R instruction")
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
                let imm_shift: u32 = immediate>>1;
                *data = (*data & 0x01FF_F07F)
                    | (imm_shift & 0x0000_0800) << 20
                    | (imm_shift & 0x0000_0400) >> 3
                    | (imm_shift & 0x0000_03F0) << 21
                    | (imm_shift & 0x0000_000F) << 8;
            }

            Instruction::U(data) => *data = (*data & 0x0000_0FFF) | (immediate & 0xFFFF_F000),

            Instruction::J(data) => {
                *data = (*data & 0x0000_0FFF)
                    | (immediate << 12) & 0x8000_0000
                    | (immediate << 20) & 0xFC00_0000
                    | (immediate << 9) & 0x0010_0000
                    | (immediate >> 12) & 0x000F_F000
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
                (0, 0, ((instruction >> 7) & 0x1f) as u8)
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
    pub fn decode_immediate(&self) -> u32 {
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
                immediate = ((data>>20) & 0x0000_0800
                | (data >> 3) & 0x0000_0400
                | (data>>21) & 0x0000_03F0
                | (data>>8) & 0x0000_000F
                | ((data >> 31) & 1) * 0xFFFF_F800)<<1;
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

        immediate
    }
}
