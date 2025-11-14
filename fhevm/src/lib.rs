pub(crate) mod codegen;
pub(crate) mod debug;
pub mod instructions;
pub mod interpreter;
pub mod keys;
pub mod parameters;
pub(crate) mod pc_update;
pub(crate) mod ram;
pub(crate) mod ram_offset;
pub(crate) mod ram_update;
pub(crate) mod rd_update;

// Re-export the main functionality
pub(crate) use instructions::*;
pub use interpreter::*;
pub(crate) use pc_update::*;
pub(crate) use ram::*;

#[cfg(test)]
mod tests;
