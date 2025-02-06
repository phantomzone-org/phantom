//! AUIPC: add upper immediate to pc
//! 
//! #Format 
//! 
//! auipc rd,imm
//! 
//! # Description Build pc-relative addresses and uses the U-type format. 
//! AUIPC forms a 32-bit offset from the 20-bit U-immediate, filling in 
//! the lowest 12 bits with zeros, adds this offset to the pc, then places 
//! the result in register rd.
//! 
//! # Implementation
//! 
//! x[rd] = pc + sext(immediate[31:12] << 12)
