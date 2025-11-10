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

pub(crate) mod b_type;
pub(crate) mod i_type;
pub(crate) mod j_type;
pub(crate) mod r_type;
pub(crate) mod s_type;
pub(crate) mod u_type;

pub(crate) fn sext(x: u32, bits: u32) -> u32 {
    (x << (u32::BITS - bits) >> (u32::BITS - bits))
        | ((x >> bits) & 1) * (0xFFFF_FFFF & (0xFFFF_FFFF << bits))
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub(crate) enum RAM_UPDATE {
    NONE = 0b00,
    SB = 0b01,
    SH = 0b10,
    SW = 0b11,
}

impl RAM_UPDATE {
    pub(crate) fn id(&self) -> u32 {
        *self as u32
    }

    pub(crate) fn eval_plain(&self, rs2: u32, ram: u32, offset: u32) -> u32 {
        match self {
            RAM_UPDATE::NONE => ram,
            RAM_UPDATE::SW => rs2,

            RAM_UPDATE::SB => match offset {
                0 => (rs2 & 0xFF) | (ram & 0xFFFF_FF00),
                1 => ((rs2 & 0xFF) << 8) | (ram & 0xFFFF_00FF),
                2 => ((rs2 & 0xFF) << 16) | (ram & 0xFF00_FFFF),
                3 => ((rs2 & 0xFF) << 24) | (ram & 0x00FF_FFFF),
                _ => panic!("invalid offset: {} != [0, 1, 2, 3]", offset),
            },
            RAM_UPDATE::SH => match offset {
                0 => (rs2 & 0xFFFF) | (ram & 0xFFFF_0000),
                2 => ((rs2 & 0xFFFF) << 16) | (ram & 0x_FFFF),
                _ => panic!("invalid offset: {} != [0, 2]", offset),
            },
        }
    }
}

pub(crate) static RAM_UPDATE_OP_LIST: &[RAM_UPDATE] = &[
    RAM_UPDATE::NONE,
    RAM_UPDATE::SB,
    RAM_UPDATE::SH,
    RAM_UPDATE::SW,
];

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub(crate) enum PC_UPDATE {
    NONE = 0b0001,
    JAL = 0b0011,
    JALR = 0b0111,
    BEQ = 0b0000,
    BNE = 0b0100,
    BLT = 0b0110,
    BGE = 0b1110,
    BLTU = 0b0010,
    BGEU = 0b1010,
}

impl PC_UPDATE {
    pub(crate) fn id(&self) -> u32 {
        *self as u32
    }

    pub(crate) fn eval_plain(&self, imm: u32, rs1: u32, rs2: u32, pc: u32) -> u32 {
        match self {
            PC_UPDATE::NONE => pc.wrapping_add(4),
            PC_UPDATE::BEQ => {
                if rs1 == rs2 {
                    pc.wrapping_add(imm)
                } else {
                    pc.wrapping_add(4)
                }
            }
            PC_UPDATE::BGE => {
                if (rs1 as i32) >= (rs2 as i32) {
                    pc.wrapping_add(imm)
                } else {
                    pc.wrapping_add(4)
                }
            }
            PC_UPDATE::BGEU => {
                if rs1 >= rs2 {
                    pc.wrapping_add(imm)
                } else {
                    pc.wrapping_add(4)
                }
            }
            PC_UPDATE::BLT => {
                if (rs1 as i32) < (rs2 as i32) {
                    pc.wrapping_add(imm)
                } else {
                    pc.wrapping_add(4)
                }
            }
            PC_UPDATE::BLTU => {
                if rs1 < rs2 {
                    pc.wrapping_add(imm)
                } else {
                    pc.wrapping_add(4)
                }
            }
            PC_UPDATE::BNE => {
                if rs1 != rs2 {
                    pc.wrapping_add(imm)
                } else {
                    pc.wrapping_add(4)
                }
            }
            PC_UPDATE::JAL => pc.wrapping_add(imm),
            PC_UPDATE::JALR => rs1.wrapping_add(imm) & 0xFFFF_FFFE,
        }
    }
}

pub(crate) static PC_UPDATE_OP_LIST: &[PC_UPDATE] = &[
    PC_UPDATE::NONE,
    PC_UPDATE::BEQ,
    PC_UPDATE::BGE,
    PC_UPDATE::BGEU,
    PC_UPDATE::BLT,
    PC_UPDATE::BLTU,
    PC_UPDATE::BNE,
    PC_UPDATE::JAL,
    PC_UPDATE::JALR,
];
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub(crate) enum RD_UPDATE {
    NONE = 0,
    LUI = 1,
    AUIPC = 2,
    ADDI = 3,
    SLTI = 4,
    SLTIU = 5,
    XORI = 6,
    ORI = 7,
    ANDI = 8,
    SLLI = 9,
    SRLI = 10,
    SRAI = 11,
    ADD = 12,
    SUB = 13,
    SLL = 14,
    SLT = 15,
    SLTU = 16,
    XOR = 17,
    SRL = 18,
    SRA = 19,
    OR = 20,
    AND = 21,
    LB = 22,
    LBU = 23,
    LH = 24,
    LHU = 25,
    LW = 26,
    JAL = 27,
    JALR = 28,
    MUL = 29,
    MULH = 30,
    MULHSU = 31,
    MULHU = 32,
    DIV = 33,
    DIVU = 34,
    REM = 35,
    REMU = 36,
}

impl RD_UPDATE {
    pub(crate) fn id(&self) -> u32 {
        *self as u32
    }

    pub(crate) fn eval_plain(&self, imm: u32, rs1: u32, rs2: u32, pc: u32, ram: u32) -> u32 {
        match self {
            RD_UPDATE::NONE => 0,
            RD_UPDATE::LUI => imm.wrapping_shl(12),
            RD_UPDATE::AUIPC => pc.wrapping_add(imm.wrapping_shl(12)),
            RD_UPDATE::ADDI => rs1.wrapping_add(imm),
            RD_UPDATE::SLTI => {
                if (rs1 as i32) < (imm as i32) {
                    return 1;
                } else {
                    return 0;
                }
            }
            RD_UPDATE::SLTIU => {
                if rs1 < imm {
                    return 1;
                } else {
                    return 0;
                }
            }
            RD_UPDATE::XORI => rs1 ^ imm,
            RD_UPDATE::ORI => rs1 | imm,
            RD_UPDATE::ANDI => rs1 & imm,
            RD_UPDATE::SLLI => rs1.wrapping_shl(imm & 0x1F),
            RD_UPDATE::SRLI => rs1.wrapping_shr(imm & 0x1F),
            RD_UPDATE::SRAI => (rs1 as i32).wrapping_shr(imm & 0x1F) as u32,
            RD_UPDATE::ADD => rs1.wrapping_add(rs2),
            RD_UPDATE::SUB => rs1.wrapping_sub(rs2),
            RD_UPDATE::SLL => rs1.wrapping_shl(rs2 & 0x1F),
            RD_UPDATE::SLT => {
                if (rs1 as i32) < (rs2 as i32) {
                    return 1;
                } else {
                    return 0;
                };
            }
            RD_UPDATE::SLTU => {
                if rs1 < rs2 {
                    return 1;
                } else {
                    return 0;
                };
            }
            RD_UPDATE::XOR => rs1 ^ rs2,
            RD_UPDATE::SRL => rs1 >> (rs2 & 0x1F),
            RD_UPDATE::SRA => (rs1 as i32 >> (rs2 & 0x1F)) as u32,
            RD_UPDATE::OR => rs1 | rs2,
            RD_UPDATE::AND => rs1 & rs2,
            RD_UPDATE::LB => sext(ram & 0xFF, 7),
            RD_UPDATE::LH => sext(ram & 0xFFFF, 15),
            RD_UPDATE::LW => ram,
            RD_UPDATE::LBU => ram & 0xFF,
            RD_UPDATE::LHU => ram & 0xFFFF,
            RD_UPDATE::JAL => pc.wrapping_add(4),
            RD_UPDATE::JALR => pc.wrapping_add(4),
            RD_UPDATE::MUL => rs1.wrapping_mul(rs2),
            RD_UPDATE::MULH => ((rs1 as i32 as i64).wrapping_mul(rs2 as i32 as i64) >> 32) as u32,
            RD_UPDATE::MULHSU => ((rs1 as i32 as i64).wrapping_mul(rs2 as i64) >> 32) as u32,
            RD_UPDATE::MULHU => ((rs1 as i64).wrapping_mul(rs2 as i64) >> 32) as u32,
            RD_UPDATE::DIV => {
                if rs2 == 0 {
                    return i32::MAX as u32;
                } else if (rs1 as i32 == i32::MIN) && (rs2 as i32 == -1) {
                    return i32::MIN as u32;
                }
                (rs1 as i32 / rs2 as i32) as u32
            }
            RD_UPDATE::DIVU => {
                if rs2 == 0 {
                    return rs1;
                }
                rs1 / rs2
            }
            RD_UPDATE::REM => {
                if rs1 == 0 {
                    rs1
                } else if rs1 as i32 == i32::MIN && rs2 as i32 == -1 {
                    0
                } else {
                    (rs1 as i32 % rs2 as i32) as u32
                }
            }
            RD_UPDATE::REMU => {
                if rs2 == 0 {
                    return rs1;
                }
                rs1 % rs2
            }
        }
    }
}

#[allow(dead_code)]
pub(crate) static RD_UPDATE_RV32M_OP_LIST: &[RD_UPDATE] = &[
    RD_UPDATE::NONE,
    RD_UPDATE::LUI,
    RD_UPDATE::AUIPC,
    RD_UPDATE::ADDI,
    RD_UPDATE::SLTI,
    RD_UPDATE::SLTIU,
    RD_UPDATE::XORI,
    RD_UPDATE::ORI,
    RD_UPDATE::ANDI,
    RD_UPDATE::SLLI,
    RD_UPDATE::SRLI,
    RD_UPDATE::SRAI,
    RD_UPDATE::ADD,
    RD_UPDATE::SUB,
    RD_UPDATE::SLL,
    RD_UPDATE::SLT,
    RD_UPDATE::SLTU,
    RD_UPDATE::XOR,
    RD_UPDATE::SRL,
    RD_UPDATE::SRA,
    RD_UPDATE::OR,
    RD_UPDATE::AND,
    RD_UPDATE::LB,
    RD_UPDATE::LBU,
    RD_UPDATE::LH,
    RD_UPDATE::LHU,
    RD_UPDATE::LW,
    RD_UPDATE::JAL,
    RD_UPDATE::JALR,
    RD_UPDATE::MUL,
    RD_UPDATE::MULH,
    RD_UPDATE::MULHSU,
    RD_UPDATE::MULHU,
    RD_UPDATE::DIV,
    RD_UPDATE::DIVU,
    RD_UPDATE::REM,
    RD_UPDATE::REMU,
];

pub(crate) static RD_UPDATE_RV32I_OP_LIST: &[RD_UPDATE] = &[
    RD_UPDATE::NONE,
    RD_UPDATE::LUI,
    RD_UPDATE::AUIPC,
    RD_UPDATE::ADDI,
    RD_UPDATE::SLTI,
    RD_UPDATE::SLTIU,
    RD_UPDATE::XORI,
    RD_UPDATE::ORI,
    RD_UPDATE::ANDI,
    RD_UPDATE::SLLI,
    RD_UPDATE::SRLI,
    RD_UPDATE::SRAI,
    RD_UPDATE::ADD,
    RD_UPDATE::SUB,
    RD_UPDATE::SLL,
    RD_UPDATE::SLT,
    RD_UPDATE::SLTU,
    RD_UPDATE::XOR,
    RD_UPDATE::SRL,
    RD_UPDATE::SRA,
    RD_UPDATE::OR,
    RD_UPDATE::AND,
    RD_UPDATE::LB,
    RD_UPDATE::LBU,
    RD_UPDATE::LH,
    RD_UPDATE::LHU,
    RD_UPDATE::LW,
    RD_UPDATE::JAL,
    RD_UPDATE::JALR,
];

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

    pub(crate) fn assert_size(&self, size: usize) {
        assert_eq!(self.imm.len(), size);
        assert_eq!(self.instructions.len(), size);
    }

    pub(crate) fn get_raw(&self, idx: usize) -> Instruction {
        self.instructions_raw[idx].clone()
    }

    pub(crate) fn get(&self, idx: usize) -> (i64, i64, i64, i64, i64, i64, i64) {
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

    pub(crate) fn assert_instruction(
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

pub(crate) const RS1MASK: u32 = 0x000F_8000;
pub(crate) const RS2MASK: u32 = 0x01F0_0000;
pub(crate) const FUNCT3MASK: u32 = 0x0000_7000;
pub(crate) const FUNCT7MASK: u32 = 0xFE00_0000;
pub(crate) const SHAMTMASK: u32 = 0x01F0_0000;
pub(crate) const RDMASK: u32 = 0x0000_0F80;
pub(crate) const OPCODEMASK: u32 = 0x0000_007F;

pub(crate) const RS1SHIFT: u32 = 15;
pub(crate) const RS2SHIFT: u32 = 20;
pub(crate) const FUNCT3SHIFT: u32 = 12;
pub(crate) const FUNCT7SHIFT: u32 = 25;
pub(crate) const SHAMTSHIFT: u32 = 20;
pub(crate) const RDSHIFT: u32 = 7;
pub(crate) const OPCODESHIFT: u32 = 0;

pub(crate) enum Type {
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
    pub(crate) fn get_type(&self) -> Type {
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
    pub(crate) fn get_funct3(&self) -> u32 {
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
    pub(crate) fn set_funct3(&mut self, funct3: u32) {
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
    pub(crate) fn get_funct7(&self) -> u32 {
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
    pub(crate) fn set_funct7(&mut self, funct7: u32) {
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
    pub(crate) fn get_rs1(&self) -> u32 {
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
    pub(crate) fn set_rs1(&mut self, rs1: u32) {
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
    pub(crate) fn get_rs2(&self) -> u32 {
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
    pub(crate) fn set_rs2(&mut self, rs2: u32) {
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
    pub(crate) fn get_rd(&self) -> u32 {
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
    pub(crate) fn set_rd(&mut self, rd: u32) {
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
    pub(crate) fn get_shamt(&self) -> u32 {
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
    pub(crate) fn set_shamt(&mut self, shamt: u32) {
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
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub(crate) fn get_opcode(&self) -> u32 {
        ((self.0 & OPCODEMASK) >> OPCODESHIFT) as u32
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub(crate) fn set_opcode(&mut self, opcode: u32) {
        self.0 =
            (self.0 & (0xFFFF_FFFF ^ OPCODEMASK)) | ((opcode as u32) << OPCODESHIFT) & OPCODEMASK
    }

    #[inline(always)]
    pub(crate) fn set_immediate(&mut self, immediate: u32) {
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
    pub(crate) fn get_immediate(&self) -> u32 {
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
    pub(crate) fn get_registers(&self) -> (u32, u32, u32) {
        match self.get_type() {
            Type::R => (self.get_rs2(), self.get_rs1(), self.get_rd()),
            Type::I => (0, self.get_rs1(), self.get_rd()),
            Type::S | Type::B => (self.get_rs2(), self.get_rs1(), 0),
            Type::U | Type::J => (0, 0, self.get_rd()),
            Type::NONE => (0, 0, 0),
        }
    }

    #[inline(always)]
    pub(crate) fn get_opid(&self) -> (RD_UPDATE, RAM_UPDATE, PC_UPDATE) {
        let opcode: u32 = self.get_opcode();
        match self.get_type() {
            Type::R => match (self.get_funct7(), self.get_funct3()) {
                (0b0000000, 0b000) => (RD_UPDATE::ADD, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0100000, 0b000) => (RD_UPDATE::SUB, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000000, 0b001) => (RD_UPDATE::SLL, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000000, 0b010) => (RD_UPDATE::SLT, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000000, 0b011) => (RD_UPDATE::SLTU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000000, 0b100) => (RD_UPDATE::XOR, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000000, 0b101) => (RD_UPDATE::SRL, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0100000, 0b101) => (RD_UPDATE::SRA, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000000, 0b110) => (RD_UPDATE::OR, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000000, 0b111) => (RD_UPDATE::AND, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000001, 0b000) => (RD_UPDATE::MUL, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000001, 0b001) => (RD_UPDATE::MULH, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000001, 0b010) => (RD_UPDATE::MULHSU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000001, 0b011) => (RD_UPDATE::MULHU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000001, 0b100) => (RD_UPDATE::DIV, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000001, 0b101) => (RD_UPDATE::DIVU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000001, 0b110) => (RD_UPDATE::REM, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                (0b0000001, 0b111) => (RD_UPDATE::REMU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
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
                        0b000 => (RD_UPDATE::ADDI, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        0b010 => (RD_UPDATE::SLTI, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        0b011 => (RD_UPDATE::SLTIU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        0b100 => (RD_UPDATE::XORI, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        0b110 => (RD_UPDATE::ORI, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        0b111 => (RD_UPDATE::ANDI, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        0b001 => (RD_UPDATE::SLLI, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        0b101 => match self.get_funct7() {
                            0b0000000 => (RD_UPDATE::SRLI, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                            0b0100000 => (RD_UPDATE::SRAI, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                            _ => panic!("invalid funct7: {:032b}", self.0),
                        },
                        _ => panic!("invalid funct3: {:032b}", self.0),
                    },
                    0b0000011 => match funct3 {
                        0b000 => (RD_UPDATE::LB, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        0b001 => (RD_UPDATE::LH, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        0b010 => (RD_UPDATE::LW, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        0b100 => (RD_UPDATE::LBU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        0b101 => (RD_UPDATE::LHU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                        _ => panic!("invalid funct3: {:032b}", self.0),
                    },
                    0b1100111 => (RD_UPDATE::JALR, RAM_UPDATE::NONE, PC_UPDATE::JALR),
                    _ => panic!("invalid instruction: {:032b}", self.0),
                }
            }
            Type::S => match self.get_funct3() {
                0b000 => (RD_UPDATE::NONE, RAM_UPDATE::SB, PC_UPDATE::NONE),
                0b001 => (RD_UPDATE::NONE, RAM_UPDATE::SH, PC_UPDATE::NONE),
                0b010 => (RD_UPDATE::NONE, RAM_UPDATE::SW, PC_UPDATE::NONE),
                _ => panic!("invalid funct3: {:032b}", self.0),
            },
            Type::B => match self.get_funct3() {
                0b000 => (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BEQ),
                0b001 => (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BNE),
                0b100 => (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BLT),
                0b101 => (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BGE),
                0b110 => (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BLTU),
                0b111 => (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BGEU),
                _ => panic!("invalid funct3: {:032b}", self.0),
            },
            Type::U => match opcode {
                0b0110111 => (RD_UPDATE::LUI, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                0b0010111 => (RD_UPDATE::AUIPC, RAM_UPDATE::NONE, PC_UPDATE::NONE),
                _ => panic!("invalid instruction: {:032b}", self.0),
            },
            Type::J => (RD_UPDATE::JAL, RAM_UPDATE::NONE, PC_UPDATE::JAL),
            Type::NONE => (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        }
    }
}
