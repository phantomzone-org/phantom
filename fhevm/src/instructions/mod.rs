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
//! 22 |lb    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(lbu)
//! 23 |lh    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(lhu)
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
//!  0 |one   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | pc += 4
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
pub mod r_type;
pub mod s_type;
pub mod u_type;

pub fn reconstruct(x: &[u32; 8]) -> u32 {
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

pub fn decompose(x: u32) -> [u32; 8] {
    let mut y: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    y[0] = ((x >> 0) & 0xF) as u32;
    y[1] = ((x >> 4) & 0xF) as u32;
    y[2] = ((x >> 8) & 0xF) as u32;
    y[3] = ((x >> 12) & 0xF) as u32;
    y[4] = ((x >> 16) & 0xF) as u32;
    y[5] = ((x >> 20) & 0xF) as u32;
    y[6] = ((x >> 24) & 0xF) as u32;
    y[7] = ((x >> 28) & 0xF) as u32;
    y
}

pub fn sext(x: u32, bits: u32) -> u32 {
    x | ((x >> bits) & 1) * (0xFFFF_FFFF & (0xFFFF_FFFF << bits))
}

pub enum StoreOps {
    None,
    Sb,
    Sh,
    Sw,
}

impl StoreOps {
    pub fn apply(&self, value: &[u32; 8]) -> (u32, [u32; 8]) {
        match self {
            StoreOps::None => (0, *value),
            StoreOps::Sb => (OpIDStore::SB, [value[0], value[1], 0, 0, 0, 0, 0, 0]),
            StoreOps::Sh => (
                OpIDStore::SH,
                [value[0], value[1], value[2], value[3], 0, 0, 0, 0],
            ),
            StoreOps::Sw => (OpIDStore::SW, *value),
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
        imm: &[u32; 8],
        x_rs1: &[u32; 8],
        x_rs2: &[u32; 8],
        pc: &[u32; 8],
    ) -> (u32, [u32; 8]) {
        match self {
            PcOps::One => (0, decompose(reconstruct(pc).wrapping_add(4))),
            PcOps::Beq => (
                OpIDPCUpdate::BEQ,
                b_type::beq::Beq::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Bge => (
                OpIDPCUpdate::BGE,
                b_type::bge::Bge::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Bgeu => (
                OpIDPCUpdate::BGEU,
                b_type::bgeu::Bgeu::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Blt => (
                OpIDPCUpdate::BLT,
                b_type::blt::Blt::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Bltu => (
                OpIDPCUpdate::BLTU,
                b_type::bltu::Bltu::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Bne => (
                OpIDPCUpdate::BNE,
                b_type::bne::Bne::apply(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Jal => (
                OpIDPCUpdate::JAL,
                j_type::jal::Jal::apply_pc(imm, x_rs1, x_rs2, pc),
            ),
            PcOps::Jalr => (
                OpIDPCUpdate::JALR,
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
        imm: &[u32; 8],
        x_rs1: &[u32; 8],
        x_rs2: &[u32; 8],
        pc: &[u32; 8],
    ) -> (u32, [u32; 8]) {
        match self {
            RdOps::None => (0, [0u32; 8]),
            RdOps::Lui => (OpIDRd::LUI, u_type::lui::Lui::apply(imm, x_rs1, x_rs2)),
            RdOps::Auipc => (
                OpIDRd::AUIPC,
                u_type::auipc::Auipc::apply(imm, x_rs1, x_rs2, pc),
            ),
            RdOps::Addi => (OpIDRd::ADDI, i_type::addi::Addi::apply(imm, x_rs1, x_rs2)),
            RdOps::Slti => (OpIDRd::SLTI, i_type::slti::Slti::apply(imm, x_rs1, x_rs2)),
            RdOps::Sltiu => (
                OpIDRd::SLTIU,
                i_type::sltiu::Sltiu::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Xori => (OpIDRd::XORI, i_type::xori::Xori::apply(imm, x_rs1, x_rs2)),
            RdOps::Ori => (OpIDRd::ORI, i_type::ori::Ori::apply(imm, x_rs1, x_rs2)),
            RdOps::Andi => (OpIDRd::ANDI, i_type::andi::Andi::apply(imm, x_rs1, x_rs2)),
            RdOps::Slli => (OpIDRd::SLLI, i_type::slli::Slli::apply(imm, x_rs1, x_rs2)),
            RdOps::Srli => (OpIDRd::SRLI, i_type::srli::Srli::apply(imm, x_rs1, x_rs2)),
            RdOps::Srai => (OpIDRd::SRAI, i_type::srai::Srai::apply(imm, x_rs1, x_rs2)),
            RdOps::Add => (OpIDRd::ADD, r_type::add::Add::apply(imm, x_rs1, x_rs2)),
            RdOps::Sub => (OpIDRd::SUB, r_type::sub::Sub::apply(imm, x_rs1, x_rs2)),
            RdOps::Sll => (OpIDRd::SLL, r_type::sll::Sll::apply(imm, x_rs1, x_rs2)),
            RdOps::Slt => (OpIDRd::SLT, r_type::slt::Slt::apply(imm, x_rs1, x_rs2)),
            RdOps::Sltu => (OpIDRd::SLTU, r_type::sltu::Sltu::apply(imm, x_rs1, x_rs2)),
            RdOps::Xor => (OpIDRd::XOR, r_type::xor::Xor::apply(imm, x_rs1, x_rs2)),
            RdOps::Srl => (OpIDRd::SRL, r_type::srl::Srl::apply(imm, x_rs1, x_rs2)),
            RdOps::Sra => (OpIDRd::SRA, r_type::sra::Sra::apply(imm, x_rs1, x_rs2)),
            RdOps::Or => (OpIDRd::OR, r_type::or::Or::apply(imm, x_rs1, x_rs2)),
            RdOps::And => (OpIDRd::AND, r_type::and::And::apply(imm, x_rs1, x_rs2)),
            RdOps::Jal => (
                OpIDRd::JAL,
                j_type::jal::Jal::apply_rd(imm, x_rs1, x_rs2, pc),
            ),
            RdOps::Jalr => (
                OpIDRd::JALR,
                i_type::jalr::Jalr::apply_rd(imm, x_rs1, x_rs2, pc),
            ),
            RdOps::Mul => (OpIDRd::MUL, r_type::mul::Mul::apply(imm, x_rs1, x_rs2)),
            RdOps::Mulh => (OpIDRd::MULH, r_type::mulh::Mulh::apply(imm, x_rs1, x_rs2)),
            RdOps::Mulhsu => (
                OpIDRd::MULHSU,
                r_type::mulhsu::Mulhsu::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Mulhu => (
                OpIDRd::MULHU,
                r_type::mulhu::Mulhu::apply(imm, x_rs1, x_rs2),
            ),
            RdOps::Div => (OpIDRd::DIV, r_type::div::Div::apply(imm, x_rs1, x_rs2)),
            RdOps::Divu => (OpIDRd::DIVU, r_type::divu::Divu::apply(imm, x_rs1, x_rs2)),
            RdOps::Rem => (OpIDRd::REM, r_type::rem::Rem::apply(imm, x_rs1, x_rs2)),
            RdOps::Remu => (OpIDRd::REMU, r_type::remu::Remu::apply(imm, x_rs1, x_rs2)),
        }
    }
}

pub static RD_RV32_OP_LIST: &[RdOps] = &[
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

pub static RD_RV32I_OP_LIST: &[RdOps] = &[
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
    pub fn apply(&self, value: &[u32; 8]) -> (u32, [u32; 8]) {
        match self {
            LoadOps::Lb => (
                OpIDRd::LB,
                decompose(sext(
                    reconstruct(&[value[0], value[1], 0, 0, 0, 0, 0, 0]),
                    7,
                )),
            ),
            LoadOps::Lh => (
                OpIDRd::LH,
                decompose(sext(
                    reconstruct(&[value[0], value[1], value[2], value[3], 0, 0, 0, 0]),
                    15,
                )),
            ),
            LoadOps::Lw => (OpIDRd::LW, *value),
            LoadOps::Lbu => (OpIDRd::LBU, [value[0], value[1], 0, 0, 0, 0, 0, 0]),
            LoadOps::Lhu => (
                OpIDRd::LHU,
                [value[0], value[1], value[2], value[3], 0, 0, 0, 0],
            ),
        }
    }
}

#[non_exhaustive]
pub struct OpIDRd;

impl OpIDRd {
    pub const NONE: u32 = 0;
    pub const LUI: u32 = 1;
    pub const AUIPC: u32 = 2;
    pub const ADDI: u32 = 3;
    pub const SLTI: u32 = 4;
    pub const SLTIU: u32 = 5;
    pub const XORI: u32 = 6;
    pub const ORI: u32 = 7;
    pub const ANDI: u32 = 8;
    pub const SLLI: u32 = 9;
    pub const SRLI: u32 = 10;
    pub const SRAI: u32 = 11;
    pub const ADD: u32 = 12;
    pub const SUB: u32 = 13;
    pub const SLL: u32 = 14;
    pub const SLT: u32 = 15;
    pub const SLTU: u32 = 16;
    pub const XOR: u32 = 17;
    pub const SRL: u32 = 18;
    pub const SRA: u32 = 19;
    pub const OR: u32 = 20;
    pub const AND: u32 = 21;
    pub const LB: u32 = 22;
    pub const LH: u32 = 23;
    pub const LW: u32 = 24;
    pub const LBU: u32 = 25;
    pub const LHU: u32 = 26;
    pub const MUL: u32 = 29;
    pub const MULH: u32 = 30;
    pub const MULHSU: u32 = 31;
    pub const MULHU: u32 = 32;
    pub const DIV: u32 = 33;
    pub const DIVU: u32 = 34;
    pub const REM: u32 = 35;
    pub const REMU: u32 = 36;
    pub const JAL: u32 = 27;
    pub const JALR: u32 = 28;
}

#[non_exhaustive]
pub struct OpIDStore;

impl OpIDStore {
    pub const NONE: u32 = 0b00;
    pub const SB: u32 = 0b01;
    pub const SH: u32 = 0b10;
    pub const SW: u32 = 0b11;
}

#[non_exhaustive]
pub struct OpIDPCUpdate;

impl OpIDPCUpdate {
    pub const NONE: u32 = 0b0001;
    pub const JAL: u32 = 0b0011;
    pub const JALR: u32 = 0b0111;
    pub const BNE: u32 = 0b0100;
    pub const BEQ: u32 = 0b0000;
    pub const BLT: u32 = 0b0110;
    pub const BGE: u32 = 0b1110;
    pub const BLTU: u32 = 0b0010;
    pub const BGEU: u32 = 0b1010;
}

pub struct InstructionsParser {
    pub imm: Vec<i64>,
    pub instructions: Vec<i64>,
    pub instructions_raw: Vec<Instruction>,
}

impl InstructionsParser {
    pub fn new() -> Self {
        InstructionsParser {
            imm: Vec::new(),
            instructions: Vec::new(),
            instructions_raw: Vec::new(),
        }
    }

    pub fn add(&mut self, instruction: Instruction) {
        let (rs2, rs1, rd) = instruction.get_registers();
        let (rd_w, mem_w, pc_w) = instruction.get_opid();
        self.imm.push(instruction.get_immediate() as i64);
        self.instructions.push(
            (rs2 as i64) << 26
                | (rs1 as i64) << 21
                | (rd as i64) << 16
                | (rd_w as i64) << 10
                | (mem_w as i64) << 5
                | (pc_w as i64),
        );
        self.instructions_raw.push(instruction);
    }

    pub fn assert_size(&self, size: usize) {
        assert_eq!(self.imm.len(), size);
        assert_eq!(self.instructions.len(), size);
    }

    pub fn get_raw(&self, idx: usize) -> Instruction {
        self.instructions_raw[idx].clone()
    }

    pub fn get(&self, idx: usize) -> (i64, i64, i64, i64, i64, i64, i64) {
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
        imm: i64,
        rs2: i64,
        rs1: i64,
        rd: i64,
        rd_w: i64,
        mem_w: i64,
        pc_w: i64,
    ) {
        let (imm_have, rs2_have, rs1_have, rd_have, rd_w_have, mem_w_have, pc_w_have) =
            self.get(idx);

        let number_of_instructions: usize = self.imm.len();
        assert!(number_of_instructions > idx);
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

#[derive(Clone, Debug)]
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
        let opcode: u32 = self.get_opcode();
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
    pub fn get_funct3(&self) -> u32 {
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
        ((self.0 & FUNCT3MASK) >> FUNCT3SHIFT) as u32
    }

    #[inline(always)]
    pub fn set_funct3(&mut self, funct3: u32) {
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
    pub fn get_funct7(&self) -> u32 {
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
        ((self.0 & FUNCT7MASK) >> FUNCT7SHIFT) as u32
    }

    #[inline(always)]
    pub fn set_funct7(&mut self, funct7: u32) {
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
    pub fn get_rs1(&self) -> u32 {
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
        ((self.0 & RS1MASK) >> RS1SHIFT) as u32
    }

    #[inline(always)]
    pub fn get_rs1_or_zero(&self) -> u32 {
        match self.get_type() {
            Type::R | Type::I | Type::S | Type::B => self.get_rs1(),
            _ => 0,
        }
    }

    #[inline(always)]
    pub fn set_rs1(&mut self, rs1: u32) {
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
    pub fn get_rs2(&self) -> u32 {
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
        ((self.0 & RS2MASK) >> RS2SHIFT) as u32
    }

    #[inline(always)]
    pub fn get_rs2_or_zero(&self) -> u32 {
        match self.get_type() {
            Type::R | Type::S | Type::B => self.get_rs2(),
            _ => 0,
        }
    }

    #[inline(always)]
    pub fn set_rs2(&mut self, rs2: u32) {
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
    pub fn get_rd(&self) -> u32 {
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
        ((self.0 & RDMASK) >> RDSHIFT) as u32
    }

    #[inline(always)]
    pub fn get_rd_or_zero(&self) -> u32 {
        match self.get_type() {
            Type::R | Type::I | Type::U | Type::J => self.get_rd(),
            _ => 0,
        }
    }

    #[inline(always)]
    pub fn set_rd(&mut self, rd: u32) {
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
    pub fn get_shamt(&self) -> u32 {
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
        ((self.0 & SHAMTMASK) >> SHAMTSHIFT) as u32
    }

    #[inline(always)]
    pub fn set_shamt(&mut self, shamt: u32) {
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
    pub fn get_opcode(&self) -> u32 {
        ((self.0 & OPCODEMASK) >> OPCODESHIFT) as u32
    }

    #[inline(always)]
    pub fn set_opcode(&mut self, opcode: u32) {
        self.0 =
            (self.0 & (0xFFFF_FFFF ^ OPCODEMASK)) | ((opcode as u32) << OPCODESHIFT) & OPCODEMASK
    }

    #[inline(always)]
    pub fn set_immediate(&mut self, immediate: u32) {
        match self.get_type() {
            Type::R => panic!("cannot encode immediate on type R instruction"),
            Type::I => match (self.get_funct3(), self.get_opcode()) {
                (0b001, 0b0010011) | (0b101, 0b0010011) => self.set_shamt(immediate as u32),
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
    pub fn get_registers(&self) -> (u32, u32, u32) {
        match self.get_type() {
            Type::R => (self.get_rs2(), self.get_rs1(), self.get_rd()),
            Type::I => (0, self.get_rs1(), self.get_rd()),
            Type::S | Type::B => (self.get_rs2(), self.get_rs1(), 0),
            Type::U | Type::J => (0, 0, self.get_rd()),
            Type::NONE => (0, 0, 0),
        }
    }

    #[inline(always)]
    pub fn get_opid(&self) -> (u32, u32, u32) {
        let opcode: u32 = self.get_opcode();
        match self.get_type() {
            Type::R => match (self.get_funct7(), self.get_funct3()) {
                (0b0000000, 0b000) => (OpIDRd::ADD, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0100000, 0b000) => (OpIDRd::SUB, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000000, 0b001) => (OpIDRd::SLL, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000000, 0b010) => (OpIDRd::SLT, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000000, 0b011) => (OpIDRd::SLTU, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000000, 0b100) => (OpIDRd::XOR, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000000, 0b101) => (OpIDRd::SRL, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0100000, 0b101) => (OpIDRd::SRA, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000000, 0b110) => (OpIDRd::OR, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000000, 0b111) => (OpIDRd::AND, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000001, 0b000) => (OpIDRd::MUL, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000001, 0b001) => (OpIDRd::MULH, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000001, 0b010) => (OpIDRd::MULHSU, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000001, 0b011) => (OpIDRd::MULHU, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000001, 0b100) => (OpIDRd::DIV, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000001, 0b101) => (OpIDRd::DIVU, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000001, 0b110) => (OpIDRd::REM, OpIDStore::NONE, OpIDPCUpdate::NONE),
                (0b0000001, 0b111) => (OpIDRd::REMU, OpIDStore::NONE, OpIDPCUpdate::NONE),
                _ => panic!(
                    "invalid funct3 {:03b} or funct7 {:07b}: {:032b}",
                    self.get_funct3(),
                    self.get_funct7(),
                    self.0
                ),
            },
            Type::I => {
                let funct3: u32 = self.get_funct3();
                match opcode {
                    0b0010011 => match funct3 {
                        0b000 => (OpIDRd::ADDI, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        0b010 => (OpIDRd::SLTI, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        0b011 => (OpIDRd::SLTIU, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        0b100 => (OpIDRd::XORI, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        0b110 => (OpIDRd::ORI, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        0b111 => (OpIDRd::ANDI, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        0b001 => (OpIDRd::SLLI, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        0b101 => match self.get_funct7() {
                            0b0000000 => (OpIDRd::SRLI, OpIDStore::NONE, OpIDPCUpdate::NONE),
                            0b0100000 => (OpIDRd::SRAI, OpIDStore::NONE, OpIDPCUpdate::NONE),
                            _ => panic!("invalid funct7: {:032b}", self.0),
                        },
                        _ => panic!("invalid funct3: {:032b}", self.0),
                    },
                    0b0000011 => match funct3 {
                        0b000 => (OpIDRd::LB, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        0b001 => (OpIDRd::LH, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        0b010 => (OpIDRd::LW, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        0b100 => (OpIDRd::LBU, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        0b101 => (OpIDRd::LHU, OpIDStore::NONE, OpIDPCUpdate::NONE),
                        _ => panic!("invalid funct3: {:032b}", self.0),
                    },
                    0b1100111 => (OpIDRd::JALR, OpIDStore::NONE, OpIDPCUpdate::JALR),
                    _ => panic!("invalid instruction: {:032b}", self.0),
                }
            }
            Type::S => match self.get_funct3() {
                0b000 => (OpIDRd::NONE, OpIDStore::SB, OpIDPCUpdate::NONE),
                0b001 => (OpIDRd::NONE, OpIDStore::SH, OpIDPCUpdate::NONE),
                0b010 => (OpIDRd::NONE, OpIDStore::SW, OpIDPCUpdate::NONE),
                _ => panic!("invalid funct3: {:032b}", self.0),
            },
            Type::B => match self.get_funct3() {
                0b000 => (OpIDRd::NONE, OpIDStore::NONE, OpIDPCUpdate::BEQ),
                0b001 => (OpIDRd::NONE, OpIDStore::NONE, OpIDPCUpdate::BNE),
                0b100 => (OpIDRd::NONE, OpIDStore::NONE, OpIDPCUpdate::BLT),
                0b101 => (OpIDRd::NONE, OpIDStore::NONE, OpIDPCUpdate::BGE),
                0b110 => (OpIDRd::NONE, OpIDStore::NONE, OpIDPCUpdate::BLTU),
                0b111 => (OpIDRd::NONE, OpIDStore::NONE, OpIDPCUpdate::BGEU),
                _ => panic!("invalid funct3: {:032b}", self.0),
            },
            Type::U => match opcode {
                0b0110111 => (OpIDRd::LUI, OpIDStore::NONE, OpIDPCUpdate::NONE),
                0b0010111 => (OpIDRd::AUIPC, OpIDStore::NONE, OpIDPCUpdate::NONE),
                _ => panic!("invalid instruction: {:032b}", self.0),
            },
            Type::J => (OpIDRd::JAL, OpIDStore::NONE, OpIDPCUpdate::JAL),
            Type::NONE => (OpIDRd::NONE, OpIDStore::NONE, OpIDPCUpdate::NONE),
        }
    }
}
