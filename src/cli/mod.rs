//! CLI command implementations
//!
//! This module contains the implementation of all basalt CLI commands.
//! Each command has its own submodule with a `run_*` function.
//!
//! # Architecture
//!
//! - Command definitions live in `main.rs` using clap
//! - Command implementations live here in individual modules
//! - Commands delegate to core logic in `crate::core`
//! - Commands use providers through the provider abstraction

pub mod init;

// Future command modules:
// pub mod submit;
// pub mod restack;
// pub mod status;
