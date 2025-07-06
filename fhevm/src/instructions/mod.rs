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
//!  9 |slli  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] << imm[4:0]
//! 10 |srli  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] >> imm[4:0] (logical)
//! 11 |srai  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] >> imm[4:0] (arithmetic)
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
//! 24 |lw    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = M[x[rs1] + sext(imm[11:0])][31:0]
//! 25 |lbu   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = M[x[rs1] + sext(imm[11:0])][7:0]
//! 26 |lhu   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = M[x[rs1] + sext(imm[11:0])][15:0]
//!
//! 27 |jal   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + 4
//! 28 |jalr  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + 4
//!
//! 29 |mul   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1]s * x[rs2]s
//! 30 |mulh  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = (x[rs1]s * x[rs2]s)>>32
//! 31 |mulhsu| imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = (x[rs1]s * x[rs2]u)>>32
//! 32 |mulhu | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = (x[rs1]u * x[rs2]u)>>32
//! 33 |div   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1]s / x[rs2]s
//! 34 |divu  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1]u / x[rs2]u
//! 35 |rem   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1]s % x[rs2]s
//! 36 |remu  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1]u % x[rs2]u
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
//!  0 |none   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | pc += 4
//!  1 |jal   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | pc += sext(imm[19:0])
//!  2 |jalr  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | t = pc + 4; pc = (x[rs1] + sext(imm[11:0])) & ~1
//!  3 |beq   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] ==  x[rs2]), pc += sext(imm[19:0])
//!  4 |bne   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] !=  x[rs2]), pc += sext(imm[19:0])
//!  5 |blt   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] <   x[rs2]), pc += sext(imm[19:0])
//!  6 |bge   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] >=s x[rs2]), pc += sext(imm[19:0])
//!  7 |bltu  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] <u  x[rs2]), pc += sext(imm[19:0])
//!  8 |bgeu  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] >=u x[rs2]), pc += sext(imm[19:0])

pub mod b_type;
pub mod i_type;
pub mod j_type;
// pub mod memory;
pub mod r_type;
pub mod s_type;
pub mod u_type;

pub fn reconstruct(x: &[u8; 8]) -> u32 {
    let mut y: u32 = 0;
    y |= (x[7] as u32) << 28;
    y |= (x[6] as u32) << 24;
    y |= (x[5] as u32) << 20;
    y |= (x[4] as u32) << 16;
    y |= (x[3] as u32) << 12;
    y |= (x[2] as u32) << 8;
    y |= (x[1] as u32) << 4;
    y |= x[0] as u32;
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

pub enum StoreOps {
    None,
    Sb,
    Sh,
    Sw,
}

impl StoreOps {
    pub fn apply(&self, value: &[u8; 8]) -> (usize, [u8; 8]) {
        match self {
            StoreOps::None => (0, *value),
            StoreOps::Sb => (OpID::SB.1 as usize, [value[0], value[1], 0, 0, 0, 0, 0, 0]),
            StoreOps::Sh => (
                OpID::SH.1 as usize,
                [value[0], value[1], value[2], value[3], 0, 0, 0, 0],
            ),
            StoreOps::Sw => (OpID::SW.1 as usize, *value),
        }
    }
}

pub static STORE_OPS_LIST: &[StoreOps] =
    &[StoreOps::None, StoreOps::Sb, StoreOps::Sh, StoreOps::Sw];

pub enum PcOps {
    One,
    Jal,
    Jalr,
    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,
}

impl PcOps {
    pub fn apply(
        &self,
        imm: &[u8; 8],
        x_rs1: &[u8; 8],
        x_rs2: &[u8; 8],
        pc: &[u8; 8],
    ) -> (usize, [u8; 8]) {
        match self {
            PcOps::One => (0, decompose(reconstruct(pc).wrapping_add(4))),
            PcOps::Beq => (
                OpID::BEQ.2 as usize,
                b_type::beq::Beq::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Bge => (
                OpID::BGE.2 as usize,
                b_type::bge::Bge::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Bgeu => (
                OpID::BGEU.2 as usize,
                b_type::bgeu::Bgeu::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Blt => (
                OpID::BLT.2 as usize,
                b_type::blt::Blt::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Bltu => (
                OpID::BLTU.2 as usize,
                b_type::bltu::Bltu::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Bne => (
                OpID::BNE.2 as usize,
                b_type::bne::Bne::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Jal => (
                OpID::JAL.2 as usize,
                j_type::jal::Jal::apply_pc(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Jalr => (
                OpID::JALR.2 as usize,
                i_type::jalr::Jalr::apply_pc(imm, x_rs1, x_rs2, pc),
            ),
        }
    }
}

pub static PC_OPS_LIST: &[PcOps] = &[
    PcOps::One,
    PcOps::Beq,
    PcOps::Bge,
    PcOps::Bgeu,
    PcOps::Blt,
    PcOps::Bltu,
    PcOps::Bne,
    PcOps::Jal,
    PcOps::Jalr,
];

pub enum RdOps {
    None,
    Lui,
    Auipc,
    Addi,
    Slti,
    Sltiu,
    Xori,
    Ori,
    Andi,
    Slli,
    Srli,
    Srai,
    Add,
    Sub,
    Sll,
    Slt,
    Sltu,
    Xor,
    Srl,
    Sra,
    Or,
    And,
    Jal,
    Jalr,
    Mul,
    Mulh,
    Mulhsu,
    Mulhu,
    Div,
    Divu,
    Rem,
    Remu,
}

impl RdOps {
    pub fn apply(
        &self,
        imm: &[u8; 8],
        x_rs1: &[u8; 8],
        x_rs2: &[u8; 8],
        pc: &[u8; 8],
    ) -> (usize, [u8; 8]) {
        match self {
            RdOps::None => (0, [0u8; 8]),
            RdOps::Lui => (
                OpID::LUI.0 as usize,
                u_type::lui::Lui::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Auipc => (
                OpID::AUIPC.0 as usize,
                u_type::auipc::Auipc::apply(imm, x_rs1, x_rs2, pc),
            ),
            RdOps::Addi => (
                OpID::ADDI.0 as usize,
                i_type::addi::Addi::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Slti => (
                OpID::SLTI.0 as usize,
                i_type::slti::Slti::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Sltiu => (
                OpID::SLTIU.0 as usize,
                i_type::sltiu::Sltiu::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Xori => (
                OpID::XORI.0 as usize,
                i_type::xori::Xori::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Ori => (
                OpID::ORI.0 as usize,
                i_type::ori::Ori::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Andi => (
                OpID::ANDI.0 as usize,
                i_type::andi::Andi::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Slli => (
                OpID::SLLI.0 as usize,
                i_type::slli::Slli::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Srli => (
                OpID::SRLI.0 as usize,
                i_type::srli::Srli::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Srai => (
                OpID::SRAI.0 as usize,
                i_type::srai::Srai::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Add => (
                OpID::ADD.0 as usize,
                r_type::add::Add::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Sub => (
                OpID::SUB.0 as usize,
                r_type::sub::Sub::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Sll => (
                OpID::SLL.0 as usize,
                r_type::sll::Sll::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Slt => (
                OpID::SLT.0 as usize,
                r_type::slt::Slt::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Sltu => (
                OpID::SLTU.0 as usize,
                r_type::sltu::Sltu::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Xor => (
                OpID::XOR.0 as usize,
                r_type::xor::Xor::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Srl => (
                OpID::SRL.0 as usize,
                r_type::srl::Srl::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Sra => (
                OpID::SRA.0 as usize,
                r_type::sra::Sra::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Or => (
                OpID::OR.0 as usize,
                r_type::or::Or::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::And => (
                OpID::AND.0 as usize,
                r_type::and::And::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Jal => (
                OpID::JAL.0 as usize,
                j_type::jal::Jal::apply_rd(imm, x_rs1, x_rs2, pc),
            ),
            RdOps::Jalr => (
                OpID::JALR.0 as usize,
                i_type::jalr::Jalr::apply_rd(imm, x_rs1, x_rs2, pc),
            ),
            RdOps::Mul => (
                OpID::MUL.0 as usize,
                r_type::mul::Mul::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Mulh => (
                OpID::MULH.0 as usize,
                r_type::mulh::Mulh::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Mulhsu => (
                OpID::MULHSU.0 as usize,
                r_type::mulhsu::Mulhsu::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Mulhu => (
                OpID::MULHU.0 as usize,
                r_type::mulhu::Mulhu::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Div => (
                OpID::DIV.0 as usize,
                r_type::div::Div::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Divu => (
                OpID::DIVU.0 as usize,
                r_type::divu::Divu::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Rem => (
                OpID::REM.0 as usize,
                r_type::rem::Rem::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Remu => (
                OpID::REMU.0 as usize,
                r_type::remu::Remu::apply(imm, x_rs1, x_rs2),
            ),
        }
    }
}

pub static RD_OPS_LIST: &[RdOps] = &[
    RdOps::None,
    RdOps::Lui,
    RdOps::Auipc,
    RdOps::Addi,
    RdOps::Slti,
    RdOps::Sltiu,
    RdOps::Xori,
    RdOps::Ori,
    RdOps::Andi,
    RdOps::Slli,
    RdOps::Srli,
    RdOps::Srai,
    RdOps::Add,
    RdOps::Sub,
    RdOps::Sll,
    RdOps::Slt,
    RdOps::Sltu,
    RdOps::Xor,
    RdOps::Srl,
    RdOps::Sra,
    RdOps::Or,
    RdOps::And,
    RdOps::Jal,
    RdOps::Jalr,
    RdOps::Mul,
    RdOps::Mulh,
    RdOps::Mulhsu,
    RdOps::Mulhu,
    RdOps::Div,
    RdOps::Divu,
    RdOps::Rem,
    RdOps::Remu,
];

pub enum LoadOps {
    Lb,
    Lbu,
    Lh,
    Lhu,
    Lw,
}

pub static LOAD_OPS_LIST: &[LoadOps] = &[
    LoadOps::Lb,
    LoadOps::Lbu,
    LoadOps::Lh,
    LoadOps::Lhu,
    LoadOps::Lw,
];

impl LoadOps {
    pub fn apply(&self, value: &[u8; 8]) -> (usize, [u8; 8]) {
        match self {
            LoadOps::Lb => (
                OpID::LB.0 as usize,
                decompose(sext(
                    reconstruct(&[value[0], value[1], 0, 0, 0, 0, 0, 0]),
                    7,
                )),
            ),
            LoadOps::Lh => (
                OpID::LH.0 as usize,
                decompose(sext(
                    reconstruct(&[value[0], value[1], value[2], value[3], 0, 0, 0, 0]),
                    15,
                )),
            ),
            LoadOps::Lw => (OpID::LW.0 as usize, *value),
            LoadOps::Lbu => (OpID::LBU.0 as usize, [value[0], value[1], 0, 0, 0, 0, 0, 0]),
            LoadOps::Lhu => (
                OpID::LHU.0 as usize,
                [value[0], value[1], value[2], value[3], 0, 0, 0, 0],
            ),
        }
    }
}

#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct OpID(u8, u8, u8);

impl OpID {
    pub const NONE: OpID = OpID(0, 0, 0);
    pub const LUI: OpID = OpID(1, 0, 0);
    pub const AUIPC: OpID = OpID(2, 0, 0);
    pub const ADDI: OpID = OpID(3, 0, 0);
    pub const SLTI: OpID = OpID(4, 0, 0);
    pub const SLTIU: OpID = OpID(5, 0, 0);
    pub const XORI: OpID = OpID(6, 0, 0);
    pub const ORI: OpID = OpID(7, 0, 0);
    pub const ANDI: OpID = OpID(8, 0, 0);
    pub const SLLI: OpID = OpID(9, 0, 0);
    pub const SRLI: OpID = OpID(10, 0, 0);
    pub const SRAI: OpID = OpID(11, 0, 0);
    pub const ADD: OpID = OpID(12, 0, 0);
    pub const SUB: OpID = OpID(13, 0, 0);
    pub const SLL: OpID = OpID(14, 0, 0);
    pub const SLT: OpID = OpID(15, 0, 0);
    pub const SLTU: OpID = OpID(16, 0, 0);
    pub const XOR: OpID = OpID(17, 0, 0);
    pub const SRL: OpID = OpID(18, 0, 0);
    pub const SRA: OpID = OpID(19, 0, 0);
    pub const OR: OpID = OpID(20, 0, 0);
    pub const AND: OpID = OpID(21, 0, 0);
    pub const LB: OpID = OpID(22, 0, 0);
    pub const LH: OpID = OpID(23, 0, 0);
    pub const LW: OpID = OpID(24, 0, 0);
    pub const LBU: OpID = OpID(25, 0, 0);
    pub const LHU: OpID = OpID(26, 0, 0);
    pub const MUL: OpID = OpID(29, 0, 0);
    pub const MULH: OpID = OpID(30, 0, 0);
    pub const MULHSU: OpID = OpID(31, 0, 0);
    pub const MULHU: OpID = OpID(32, 0, 0);
    pub const DIV: OpID = OpID(33, 0, 0);
    pub const DIVU: OpID = OpID(34, 0, 0);
    pub const REM: OpID = OpID(35, 0, 0);
    pub const REMU: OpID = OpID(36, 0, 0);
    pub const SB: OpID = OpID(0, 1, 0);
    pub const SH: OpID = OpID(0, 2, 0);
    pub const SW: OpID = OpID(0, 3, 0);
    pub const JAL: OpID = OpID(27, 0, 1);
    pub const JALR: OpID = OpID(28, 0, 2);
    pub const BEQ: OpID = OpID(0, 0, 3);
    pub const BNE: OpID = OpID(0, 0, 4);
    pub const BLT: OpID = OpID(0, 0, 5);
    pub const BGE: OpID = OpID(0, 0, 6);
    pub const BLTU: OpID = OpID(0, 0, 7);
    pub const BGEU: OpID = OpID(0, 0, 8);
}

impl OpID {
    fn rd_w_id(&self) -> u8 {
        self.0
    }

    fn mem_w_id(&self) -> u8 {
        self.1
    }

    fn pc_w_id(&self) -> u8 {
        self.2
    }
}

#[derive(Clone, Copy)]
pub struct OpRegisters(u8, u8, u8);

impl OpRegisters {
    fn new(rs2: u8, rs1: u8, rd: u8) -> Self {
        Self(rs2, rs1, rd)
    }

    pub(crate) fn rs1(&self) -> u8 {
        self.1
    }
    pub(crate) fn rs2(&self) -> u8 {
        self.0
    }
    pub(crate) fn rsd(&self) -> u8 {
        self.2
    }
}

pub struct InstructionsParser {
    pub imm: Vec<u32>,
    pub op_registers: Vec<OpRegisters>,
    pub op_id: Vec<OpID>,
    pub instructions: Vec<u32>,
}

impl InstructionsParser {
    pub fn new() -> Self {
        InstructionsParser {
            imm: Vec::new(),
            op_id: Vec::new(),
            op_registers: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub fn max_count(&self) -> usize {
        self.imm.len()
    }

    pub fn add(&mut self, instruction: Instruction) {
        self.imm.push(instruction.get_immediate());
        self.op_registers.push(instruction.get_registers());
        self.op_id.push(instruction.get_opid());
    }

    pub fn assert_size(&self, size: usize) {
        assert_eq!(self.imm.len(), size);
        assert_eq!(self.instructions.len(), size);
    }

    /// Retreive immediate value of insturction at index `idx` (i.e. pc = idx * 4)
    pub fn get_imm(&self, idx: usize) -> u32 {
        self.instructions[idx]
    }

    pub fn get_opid(&self, idx: usize) -> OpID {
        self.op_id[idx]
    }

    pub fn get_opregisters(&self, idx: usize) -> OpRegisters {
        self.op_registers[idx]
    }

    pub fn _get(&self, idx: usize) -> (i64, i64, i64, i64, i64, i64, i64) {
        assert!(self.imm.len() > idx);
        let data = self.instructions[idx];
        (
            self.imm[idx] as i64,
            ((data >> 26) & 0x1F) as i64,
            ((data >> 21) & 0x1F) as i64,
            ((data >> 16) & 0x1F) as i64,
            ((data >> 10) & 0x3F) as i64,
            ((data >> 5) & 0x1F) as i64,
            (data & 0x1F) as i64,
        )
    }

    pub fn assert_instruction(
        &self,
        idx: usize,
        imm: u32,
        rs2: u8,
        rs1: u8,
        rd: u8,
        rd_w: u8,
        mem_w: u8,
        pc_w: u8,
    ) {
        let number_of_instructions: usize = self.imm.len();
        assert!(number_of_instructions > idx);

        let imm_have = self.get_imm(idx);
        let op_registers = self.get_opregisters(idx);
        let (rs2_have, rs1_have, rd_have) =
            (op_registers.rs2(), op_registers.rs1(), op_registers.rsd());
        let op_id = self.get_opid(idx);
        let (rd_w_have, mem_w_have, pc_w_have) =
            (op_id.rd_w_id(), op_id.mem_w_id(), op_id.pc_w_id());

        assert_eq!(
            imm_have, imm,
            "invalid imm: have {:032b} want {:032b}",
            imm_have, imm
        );
        assert_eq!(
            rs2_have, rs2,
            "invalid rs2: have {:05b} want {:05b}",
            rs2_have, rs2
        );
        assert_eq!(
            rs1_have, rs1,
            "invalid rs1: have {:05b} want {:05b}",
            rs1_have, rs1
        );
        assert_eq!(
            rd_have, rd,
            "invalid rd: have {:05b} want {:05b}",
            rd_have, rd
        );
        assert_eq!(
            rd_w_have, rd_w,
            "invalid rd_w: have {} want {}",
            rd_w_have, rd_w
        );
        assert_eq!(
            mem_w_have, mem_w,
            "invalid mem_w: have {} want {}",
            mem_w_have, mem_w
        );
        assert_eq!(
            pc_w_have, pc_w,
            "invalid pc_w: have {} want {}",
            pc_w_have, pc_w
        );
    }
}

pub struct Instruction(pub u32);

pub const RS1MASK: u32 = 0x000F_8000;
pub const RS2MASK: u32 = 0x01F0_0000;
pub const FUNCT3MASK: u32 = 0x0000_7000;
pub const FUNCT7MASK: u32 = 0xFE00_0000;
pub const SHAMTMASK: u32 = 0x01F0_0000;
pub const RDMASK: u32 = 0x0000_0F80;
pub const OPCODEMASK: u32 = 0x0000_007F;

pub const RS1SHIFT: u32 = 15;
pub const RS2SHIFT: u32 = 20;
pub const FUNCT3SHIFT: u32 = 12;
pub const FUNCT7SHIFT: u32 = 25;
pub const SHAMTSHIFT: u32 = 20;
pub const RDSHIFT: u32 = 7;
pub const OPCODESHIFT: u32 = 0;

pub enum Type {
    NONE,
    R,
    I,
    S,
    B,
    U,
    J,
}

impl Instruction {
    #[inline(always)]
    pub fn new(instruction: u32) -> Self {
        Self(instruction)
    }

    pub fn print(&self) {
        println!("{:032b}", self.0);
    }

    #[inline(always)]
    pub fn get_type(&self) -> Type {
        let opcode: u8 = self.get_opcode();
        match opcode {
            0b0110111 | 0b0010111 => Type::U,
            0b0010011 | 0b0000011 | 0b1100111 => Type::I,
            0b0110011 => Type::R,
            0b0100011 => Type::S,
            0b1101111 => Type::J,
            0b1100011 => Type::B,
            0b1110011 => Type::NONE,
            _ => panic!("unrecognized opcode: {:07b}", opcode),
        }
    }

    #[inline(always)]
    pub fn get_funct3(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::U | Type::J => panic!(
                    "cannot get funct3 on Type::(U, J) instructions: {:032b}",
                    self.0
                ),
                _ => {}
            }
        }
        ((self.0 & FUNCT3MASK) >> FUNCT3SHIFT) as u8
    }

    #[inline(always)]
    pub fn set_funct3(&mut self, funct3: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::U | Type::J => panic!(
                    "cannot set funct3 on Type::(U, J) instructions: {:032b}",
                    self.0
                ),
                _ => {}
            }
        }
        self.0 =
            (self.0 & (0xFFFF_FFFF ^ FUNCT3MASK)) | ((funct3 as u32) << FUNCT3SHIFT) & FUNCT3MASK
    }

    #[inline(always)]
    pub fn get_funct7(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::I => {}
                _ => panic!(
                    "cannot get funct7 on Type::(S, B, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        ((self.0 & FUNCT7MASK) >> FUNCT7SHIFT) as u8
    }

    #[inline(always)]
    pub fn set_funct7(&mut self, funct7: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R => {}
                _ => panic!(
                    "cannot set funct7 on Type::(I, S, B, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        self.0 =
            (self.0 & (0xFFFF_FFFF ^ FUNCT7MASK)) | ((funct7 as u32) << FUNCT7SHIFT) & FUNCT7MASK
    }

    #[inline(always)]
    pub fn get_rs1(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::I | Type::S | Type::B => {}
                _ => panic!(
                    "cannot get rs1 on Type::(U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        ((self.0 & RS1MASK) >> RS1SHIFT) as u8
    }

    #[inline(always)]
    pub fn set_rs1(&mut self, rs1: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::I | Type::S | Type::B => {}
                _ => panic!(
                    "cannot set rs1 on Type::(U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        self.0 = (self.0 & (0xFFFF_FFFF ^ RS1MASK)) | ((rs1 as u32) << RS1SHIFT) & RS1MASK
    }

    #[inline(always)]
    pub fn get_rs2(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::S | Type::B => {}
                _ => panic!(
                    "cannot get rs2 on Type::(I, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        ((self.0 & RS2MASK) >> RS2SHIFT) as u8
    }

    #[inline(always)]
    pub fn set_rs2(&mut self, rs2: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::S | Type::B => {}
                _ => panic!(
                    "cannot set rs2 on Type::(I, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        self.0 = (self.0 & (0xFFFF_FFFF ^ RS2MASK)) | ((rs2 as u32) << RS2SHIFT) & RS2MASK
    }

    #[inline(always)]
    pub fn get_rd(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::I | Type::U | Type::J => {}
                _ => panic!(
                    "cannot get rd on Type::(S, B) instructions: {:032b}",
                    self.0
                ),
            }
        }
        ((self.0 & RDMASK) >> RDSHIFT) as u8
    }

    #[inline(always)]
    pub fn set_rd(&mut self, rd: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::I | Type::U | Type::J => {}
                _ => panic!(
                    "cannot set rd on Type::(S, B) instructions: {:032b}",
                    self.0
                ),
            }
        }
        self.0 = (self.0 & (0xFFFF_FFFF ^ RDMASK)) | ((rd as u32) << RDSHIFT) & RDMASK
    }

    #[inline(always)]
    pub fn get_shamt(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::I => {}
                _ => panic!(
                    "cannot get shamt on Type::(R, S, B, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        ((self.0 & SHAMTMASK) >> SHAMTSHIFT) as u8
    }

    #[inline(always)]
    pub fn set_shamt(&mut self, shamt: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::I => {}
                _ => panic!(
                    "cannot set shamt on Type::(R, S, B, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        self.0 = (self.0 & (0xFFFF_FFFF ^ SHAMTMASK)) | ((shamt as u32) << SHAMTSHIFT) & SHAMTMASK
    }

    #[inline(always)]
    pub fn get(&self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub fn get_opcode(&self) -> u8 {
        ((self.0 & OPCODEMASK) >> OPCODESHIFT) as u8
    }

    #[inline(always)]
    pub fn set_opcode(&mut self, opcode: u8) {
        self.0 =
            (self.0 & (0xFFFF_FFFF ^ OPCODEMASK)) | ((opcode as u32) << OPCODESHIFT) & OPCODEMASK
    }

    #[inline(always)]
    pub fn set_immediate(&mut self, immediate: u32) {
        match self.get_type() {
            Type::R => panic!("cannot encode immediate on type R instruction"),
            Type::I => match (self.get_funct3(), self.get_opcode()) {
                (0b001, 0b0010011) | (0b101, 0b0010011) => self.set_shamt(immediate as u8),
                _ => i_type::set_immediate(&mut self.0, immediate),
            },
            Type::S => s_type::set_immediate(&mut self.0, immediate),
            Type::B => b_type::set_immediate(&mut self.0, immediate),
            Type::U => u_type::set_immediate(&mut self.0, immediate),
            Type::J => j_type::set_immediate(&mut self.0, immediate),
            Type::NONE => {
                panic!("cannot encode immediate on type NONE instruction")
            }
        }
    }

    #[inline(always)]
    pub fn get_immediate(&self) -> u32 {
        match self.get_type() {
            Type::R => 0,
            Type::I => match (self.get_funct3(), self.get_opcode()) {
                (0b001, 0b0010011) | (0b101, 0b0010011) => self.get_shamt() as u32,
                _ => i_type::get_immediate(&self.0),
            },
            Type::S => s_type::get_immediate(&self.0),
            Type::B => b_type::get_immediate(&self.0),
            Type::U => u_type::get_immediate(&self.0),
            Type::J => j_type::get_immediate(&self.0),
            Type::NONE => 0,
        }
    }

    #[inline(always)]
    pub fn get_registers(&self) -> OpRegisters {
        match self.get_type() {
            Type::R => OpRegisters::new(self.get_rs2(), self.get_rs1(), self.get_rd()),
            Type::I => OpRegisters::new(0, self.get_rs1(), self.get_rd()),
            Type::S | Type::B => OpRegisters::new(self.get_rs2(), self.get_rs1(), 0),
            Type::U | Type::J => OpRegisters::new(0, 0, self.get_rd()),
            Type::NONE => OpRegisters::new(0, 0, 0),
        }
    }

    #[inline(always)]
    pub fn get_opid(&self) -> OpID {
        let opcode: u8 = self.get_opcode();
        match self.get_type() {
            Type::R => match (self.get_funct7(), self.get_funct3()) {
                (0b0000000, 0b000) => OpID::ADD,
                (0b0100000, 0b000) => OpID::SUB,
                (0b0000000, 0b001) => OpID::SLL,
                (0b0000000, 0b010) => OpID::SLT,
                (0b0000000, 0b011) => OpID::SLTU,
                (0b0000000, 0b100) => OpID::XOR,
                (0b0000000, 0b101) => OpID::SRL,
                (0b0100000, 0b101) => OpID::SRA,
                (0b0000000, 0b110) => OpID::OR,
                (0b0000000, 0b111) => OpID::AND,
                (0b0000001, 0b000) => OpID::MUL,
                (0b0000001, 0b001) => OpID::MULH,
                (0b0000001, 0b010) => OpID::MULHSU,
                (0b0000001, 0b011) => OpID::MULHU,
                (0b0000001, 0b100) => OpID::DIV,
                (0b0000001, 0b101) => OpID::DIVU,
                (0b0000001, 0b110) => OpID::REM,
                (0b0000001, 0b111) => OpID::REMU,
                _ => panic!(
                    "invalid funct3 {:03b} or funct7 {:07b}: {:032b}",
                    self.get_funct3(),
                    self.get_funct7(),
                    self.0
                ),
            },
            Type::I => {
                let funct3: u8 = self.get_funct3();
                match opcode {
                    0b0010011 => match funct3 {
                        0b000 => OpID::ADDI,
                        0b010 => OpID::SLTI,
                        0b011 => OpID::SLTIU,
                        0b100 => OpID::XORI,
                        0b110 => OpID::ORI,
                        0b111 => OpID::ANDI,
                        0b001 => OpID::SLLI,
                        0b101 => match self.get_funct7() {
                            0b0000000 => OpID::SRLI,
                            0b0100000 => OpID::SRAI,
                            _ => panic!("invalid funct7: {:032b}", self.0),
                        },
                        _ => panic!("invalid funct3: {:032b}", self.0),
                    },
                    0b0000011 => match funct3 {
                        0b000 => OpID::LB,
                        0b001 => OpID::LH,
                        0b010 => OpID::LW,
                        0b100 => OpID::LBU,
                        0b101 => OpID::LHU,
                        _ => panic!("invalid funct3: {:032b}", self.0),
                    },
                    0b1100111 => OpID::JALR,
                    _ => panic!("invalid instruction: {:032b}", self.0),
                }
            }
            Type::S => match self.get_funct3() {
                0b000 => OpID::SB,
                0b001 => OpID::SH,
                0b010 => OpID::SW,
                _ => panic!("invalid funct3: {:032b}", self.0),
            },
            Type::B => match self.get_funct3() {
                0b000 => OpID::BEQ,
                0b001 => OpID::BNE,
                0b100 => OpID::BLT,
                0b101 => OpID::BGE,
                0b110 => OpID::BLTU,
                0b111 => OpID::BGEU,
                _ => panic!("invalid funct3: {:032b}", self.0),
            },
            Type::U => match opcode {
                0b0110111 => OpID::LUI,
                0b0010111 => OpID::AUIPC,
                _ => panic!("invalid instruction: {:032b}", self.0),
            },
            Type::J => OpID::JAL,
            Type::NONE => OpID::NONE,
        }
    }
}
