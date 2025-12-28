//! Core basalt logic
//!
//! This module contains provider-agnostic core functionality for basalt:
//!
//! - **Environment checking** — Verify git repository, dependencies, authentication
//! - **Git operations** — Wrapper around git commands
//! - **Metadata management** — Store and retrieve stack metadata
//! - **Stack detection** — Identify and validate linear branch stacks (TODO)
//!
//! All code in this module MUST be provider-agnostic. Provider-specific
//! logic belongs in the `providers` module.

pub mod environment;
pub mod git;
pub mod metadata;

// Future modules:
// pub mod stack;
