//! Scoped computational memory experiment, not the full native Shadow pipeline.
pub mod continuity;
pub mod knowledge;
pub mod ffi_gate;
pub mod pd_memory;
pub mod pd_spine_contract;
pub mod progressive_debatic;
pub mod zero_noi;
mod fnv;
mod framing;
mod projection;
pub use continuity::{input, summary, uninterrupted, Store};
pub use fnv::fnv1a64;
