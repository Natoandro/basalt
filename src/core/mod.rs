//! Core basalt logic
//!
//! This module contains provider-agnostic core functionality for basalt:
//!
//! - **Environment checking** — Verify git repository, dependencies, authentication
//! - **Stack detection** — Identify and validate linear branch stacks (TODO)
//! - **Git operations** — Wrapper around git commands (TODO)
//! - **Metadata management** — Store and retrieve stack metadata (TODO)
//!
//! All code in this module MUST be provider-agnostic. Provider-specific
//! logic belongs in the `providers` module.

pub mod environment;

// Future modules:
// pub mod stack;
// pub mod git;
// pub mod metadata;
