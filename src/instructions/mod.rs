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
pub mod ebreak;
pub mod ecall;
pub mod fence;
pub mod fencetso;
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
pub mod pause;
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

//!         

//! # ARITHMETIC
//! 
//!         |31-27|26-25|24-20|19-15|14-12|11-7|6 - 2|1-0|
//! add     |00000|00   | rs2 | rs1 | 000 | rd |01100| 11| x[rd] = x[rs1] + x[rs2]
//! sub     |01000|00   | rs2 | rs1 | 000 | rd |01100| 11| x[rd] = x[rs1] - x[rs2]
//! xor     |00000|00   | rs2 | rs1 | 100 | rd |01100| 11| x[rd] = x[rs1] ^ x[rs2]
//! and     |00000|00   | rs2 | rs1 | 111 | rd |01100| 11| x[rd] = x[rs1] & x[rs2]
//! or      |00000|00   | rs2 | rs1 | 110 | rd |01100| 11| x[rd] = x[rs1] | x[rs2]
//! addi    |    imm[11:0]    | rs1 | 000 | rd |00100| 11| x[rd] = x[rs1] + sext(imm)
//! xori    |    imm[11:0]    | rs1 | 100 | rd |00100| 11| x[rd] = x[rs1] ^ sext(imm)
//! ori     |    imm[11:0]    | rs1 | 110 | rd |00100| 11| x[rd] = x[rs1] | sext(imm)
//! andi    |    imm[11:0]    | rs1 | 111 | rd |00100| 11| x[rd] = x[rs1] & sext(imm)
//! sll     |00000|00   | rs2 | rs1 | 001 | rd |01100| 11| x[rd] = x[rs1] << x[rs2]
//! sra     |01000|00   | rs2 | rs1 | 101 | rd |01100| 11| x[rd] = x[rs1] >> x[rs2] (arithmetic)
//! srl     |00000|00   | rs2 | rs1 | 101 | rd |01100| 11| x[rd] = x[rs1] >> x[rs2] (logical)
//! srli    |00000|00   |shamt| rs1 | 101 | rd |00100| 11| x[rd] = x[rs1] >> shamt (logical)
//! srai    |01000|00   |shamt| rs1 | 101 | rd |00100| 11| x[rd] = x[rs1] >> shamt (arithmetic) 
//! slli    |00000|00   |shamt| rs1 | 001 | rd |00100| 11| x[rd] = x[rs1] << shamt
//! 
//! 
//! # COMPARISONS
//! 
//!         |31-27|26-25|24-20|19-15|14-12|11-7|6 - 2|1-0|
//! slti    |imm[11:0]        | rs1 | 010 | rd |00100| 11| x[rd] = (x[rs1] < sext(imm)) ? 1 : 0
//! sltiu   |imm[11:0]        | rs1 | 011 | rd |00100| 11| x[rd] = (x[rs1] <u sext(imm)) ? 1 : 0 
//! sltu    |00000|00   | rs2 | rs1 | 011 | rd |01100| 11| x[rd] = (x[rs1] <u x[rs2]) ? 1 : 0
//! slt     |00000|00   | rs2 | rs1 | 010 | rd |01100| 11| x[rd] = (x[rs1] < x[rs2]) ? 1 : 0
//! 
//! 
//! # PC UPDATES
//! 
//!         |    31   |30-27|26-25|24-20|19-15|   14-12   |        11-7        |6 - 2|1-0|
//! jal     | imm[20] |imm[10:1]|imm[11]|   imm[19:12]    |         rd         |11011| 11| x[rd] = pc + 4; pc += sext(imm)
//! jalr    |             imm[11:0]           | rs1 | 000 |         rd         |11001| 11| t = pc + 4; pc = (x[rs1] + sext(imm)) & ~1; x[rd] = t
//! auipc   |             imm[31:12]                      |         rd         |00101| 11| x[rd] = pc + sext(imm[31:12] << 12)
//! beq     | imm[12] |   imm[10:5]     | rs2 | rs1 | 000 | imm[4:1] | imm[11] |11000| 11| if (x[rs1] ==  x[rs2]) pc += sext(imm)
//! bge     | imm[12] |   imm[10:5]     | rs2 | rs1 | 101 | imm[4:1] | imm[11] |11000| 11| if (x[rs1] >=  x[rs2]) pc += sext(imm)
//! bgeu    | imm[12] |   imm[10:5]     | rs2 | rs1 | 111 | imm[4:1] | imm[11] |11000| 11| if (x[rs1] >=u x[rs2]) pc += sext(imm)
//! blt     | imm[12] |   imm[10:5]     | rs2 | rs1 | 100 | imm[4:1] | imm[11] |11000| 11| if (x[rs1] <   x[rs2]) pc += sext(imm)
//! bltu    | imm[12] |   imm[10:5]     | rs2 | rs1 | 110 | imm[4:1] | imm[11] |11000| 11| if (x[rs1] <u  x[rs2]) pc += sext(imm)
//! bne     | imm[12] |   imm[10:5]     | rs2 | rs1 | 001 | imm[4:1] | imm[11] |11000| 11| if (x[rs1] !=  x[rs2]) pc += sext(imm)
//! 
//! 
//! # LOAD
//! 
//!         |  31 - 12   |11-7|6 - 2|1-0|
//! lui     | imm[31:12] | rd |01101| 11| x[rd] = sext(imm[31:12] << 12)
//! 
//! # MEMORY WRITE
//!         |     31 - 25     |24-20|19-15|14-12|   11-7   |6 - 2|1-0|
//! sw      |    imm[11:5]    | rs2 | rs1 | 010 | imm[4:0] |01000| 11| M[x[rs1] + sext(imm)] = x[rs2][31:0]
//! sb      |    imm[11:5]    | rs2 | rs1 | 000 | imm[4:0] |01000| 11| M[x[rs1] + sext(imm)] = x[rs2][7:0]
//! sh      |    imm[11:5]    | rs2 | rs1 | 001 | imm[4:0] |01000| 11| M[x[rs1] + sext(imm)] = x[rs2][15:0] 
//! 
//! 
//! # MEMORY READ
//!         |  31  -  20  |19-15|14-12|11-7|6 - 2|1-0|
//! lbu     | imm[11:0]   | rs1 | 100 | rd |00000| 11| x[rd] = M[x[rs1] + sext(imm)][7:0]
//! lhu     | imm[11:0]   | rs1 | 101 | rd |00000| 11| x[rd] = M[x[rs1] + sext(imm)][15:0]
//! lb      | imm[11:0]   | rs1 | 000 | rd |00000| 11| x[rd] = sext(M[x[rs1] + sext(imm)][7:0])
//! lh      | imm[11:0]   | rs1 | 001 | rd |00000| 11| x[rd] = sext(M[x[rs1] + sext(imm)][15:0])
//! lw      | imm[11:0]   | rs1 | 010 | rd |00000| 11| x[rd] = sext(M[x[rs1] + sext(imm)][31:0])
//! 
//! 
//! # OTHERS
//! 
//! fence   | pred | succ |00000| rs1 | 000 |   00000  |00011| 11| Order memory operations as specified by pred and succ fields
//! simple  |00000|00     | rs2 | rs1 | 000 |    rd    |01100| 11| simple instruction example

pub trait Instruction{
    fn apply(&self, rs1: u32, rs2: u32, imm: u32) -> u32;
}

