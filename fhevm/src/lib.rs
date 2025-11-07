pub mod arithmetic;
pub(crate) mod codegen;
pub mod instructions;
pub mod interpreter;
pub mod keys;
pub mod load;
pub mod parameters;
pub mod pc_update;
pub mod ram;
pub(crate) mod ram_offset;
pub mod store;

// Re-export the main functionality
pub(crate) use instructions::*;
pub use interpreter::*;
pub(crate) use load::*;
pub(crate) use pc_update::*;
pub(crate) use ram::*;

#[cfg(test)]
mod tests;
