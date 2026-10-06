//! warmups-core: every rule of the warm-up and exit-ticket tool.
//!
//! The crate performs no input or output and uses no floats, hash maps or
//! clocks, so the same inputs always give the same result. The page and the
//! wasm shim only pass text in and draw what comes back.

pub mod bank;
pub mod fraction;
