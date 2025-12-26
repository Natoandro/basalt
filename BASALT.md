# basalt — Multi-Provider Stacked Change Workflow Tool

## Overview

**basalt** is a planned Rust rewrite of [Charcoal](https://github.com/danerwilliams/charcoal) that brings a stacked change workflow to **multiple Git hosting providers**, starting with **GitLab**.

The current repository contains **Charcoal**, a TypeScript-based CLI tool for GitHub stacked pull requests (forked from the open-source Graphite CLI). **basalt** represents the next evolution: a ground-up Rust rewrite with multi-provider support.

The command-line tool is invoked as **`bt`** for brevity and ease of use.

The tool is intentionally:

* **CLI-only**
* **Local-first** (stack state lives in Git / local metadata)
* **Provider-agnostic** (pluggable backends for GitLab, GitHub, and potentially others)
* **Fast and portable** (leveraging Rust's performance and cross-platform capabilities)

---

## Why Rust?

The rewrite to Rust provides:

* **Performance**: Faster execution for large repositories and complex stacks
* **Portability**: Single binary distribution with no runtime dependencies
* **Type safety**: Robust compile-time guarantees for stack manipulation logic
* **Maintainability**: Clear abstractions for multi-provider support

---

## Current State

This repository currently contains:

* **Charcoal** (TypeScript/Node.js)
  * GitHub-focused stacked PR workflow tool
  * Fork of the open-source Graphite CLI
  * Fully functional for GitHub workflows
  * Located in `apps/cli/`

The Rust rewrite has not yet begun.

---

## Core Concept

* A **stack** is a linear sequence of Git branches
* Each branch maps to **exactly one code review** (MR for GitLab, PR for GitHub, etc.)
* Stacks are submitted **bottom → top**
* Rebasing/restacking is explicit and predictable

---

## Multi-Provider Architecture

### Provider Abstraction

basalt will implement a clean provider interface that abstracts:

* **Authentication** (delegated to provider CLIs: `glab`, `gh`, etc.)
* **Review creation** (MRs, PRs, etc.)
* **Review updates** (descriptions, target branches, etc.)
* **Metadata storage** (branch → review ID mapping)

### Planned Providers

1. **GitLab** (MVP target)
   * Uses `glab` CLI for all GitLab API interactions
   * No direct API usage
   * Draft MR support
   * MR dependency relationships

2. **GitHub** (leveraging Charcoal knowledge)
   * Uses `gh` CLI for all GitHub API interactions
   * Maintains parity with current Charcoal features
   * Draft PR support
   * Stack information in PR descriptions

3. **Future providers** (post-MVP)
   * Gitea / Forgejo
   * Bitbucket
   * Azure DevOps
   * Self-hosted solutions

### Provider Detection

basalt will auto-detect the provider based on:

1. Git remote URL inspection
2. Explicit configuration override
3. Available provider CLI tools

---

## Architectural Differences vs Charcoal/Graphite

### What stays conceptually the same

* Linear stacked branches
* One review per branch
* Bottom-up submission order
* Rebase-driven restacking
* Local-first metadata

### What changes

* **Language**: TypeScript → Rust
* **Scope**: GitHub-only → Multi-provider
* **API approach**: Direct API calls → Provider CLI delegation
* **Distribution**: npm package → Single binary
* **Dependencies**: Node.js runtime → No runtime dependencies

### Hard Dependencies

* `git` (always required)
* Provider-specific CLI (per provider):
  * `glab` for GitLab
  * `gh` for GitHub
  * etc.

---

## MVP Scope (GitLab Provider)

**MVP success criteria:**

> From the top branch of a stack, run one command and have all stacked branches pushed and represented as draft GitLab MRs, using a Rust implementation with a pluggable provider architecture.

Anything not strictly required to reach this outcome is **deferred** for post-MVP.

---

# TODO — MVP (Rust Rewrite + GitLab Provider)

## 1. Project Setup

* [ ] Initialize Rust project structure (`cargo init`)
* [ ] Set up CLI framework (clap or similar)
* [ ] Define binary name: `bt`
* [ ] Configure build tooling and CI
* [ ] Define provider trait/interface

## 2. Provider Abstraction Layer

* [ ] Define `Provider` trait
* [ ] Define review metadata structures
* [ ] Define authentication interface
* [ ] Create mock provider for testing
* [ ] Implement provider detection logic

## 3. GitLab Provider Implementation

* [ ] Implement GitLab provider
* [ ] Verify `glab` is installed and configured
* [ ] Verify `glab auth status` succeeds
* [ ] Implement MR creation via `glab mr create`
* [ ] Implement MR update logic
* [ ] Parse JSON output from `glab` commands
* [ ] Store branch → MR ID mappings

## 4. Environment & Dependency Checks

* [ ] Verify execution inside a Git repository
* [ ] Verify required provider CLI is available
* [ ] Verify provider authentication
* [ ] Provide clear, actionable error messages

## 5. Repository Initialization (`bt init`)

* [ ] Create config directory (e.g. `.basalt/`)
* [ ] Auto-detect Git provider from remote
* [ ] Detect and store default base branch
* [ ] Persist minimal config (TOML or JSON)
* [ ] Support provider override flag

## 6. Stack Detection & Validation

* [ ] Detect current branch
* [ ] Walk linear ancestry up to base branch
* [ ] Validate stack is linear (no merges)
* [ ] Build in-memory stack representation
* [ ] Abort on ambiguous or unsupported graphs

## 7. Stack Submission (`bt submit`)

* [ ] Enumerate stack bottom → top
* [ ] Checkout each branch
* [ ] Push branch (create upstream if needed)
* [ ] Create review via provider
* [ ] Set appropriate metadata (draft status, etc.)
* [ ] Update review if it already exists
* [ ] Store review ID and URL
* [ ] Print submission summary

## 8. Restacking (`bt restack`)

* [ ] Rebase first stack branch onto base
* [ ] Rebase each subsequent branch onto its parent
* [ ] Surface conflicts directly from git
* [ ] Force-push rebased branches
* [ ] Handle rebase failures gracefully

## 9. Metadata Storage

* [ ] Design metadata format (per-provider)
* [ ] Store branch → review ID mapping
* [ ] Persist provider information
* [ ] Load metadata on subsequent runs
* [ ] Handle metadata migration/versioning

## 10. Output & UX

* [ ] Deterministic, readable CLI output
* [ ] Success and failure indicators
* [ ] Display review URLs after submission
* [ ] Progress indicators for multi-step operations
* [ ] Colored output (optional, respects NO_COLOR)

## 11. Documentation

* [ ] Write comprehensive README
* [ ] Installation instructions (cargo install, binaries)
* [ ] Provider-specific setup guides
* [ ] End-to-end workflow examples
* [ ] Migration guide from Charcoal

## 12. Testing

* [ ] Unit tests for stack logic
* [ ] Provider trait tests with mock provider
* [ ] Integration tests with real git repos
* [ ] CI pipeline setup

---

# TODO — Post-MVP Features

## A. GitHub Provider

* [ ] Implement GitHub provider using `gh` CLI
* [ ] Feature parity with current Charcoal
* [ ] Migration tooling from Charcoal metadata
* [ ] GitHub-specific optimizations

## B. Stack Intelligence

* [ ] Automatic stack inference
* [ ] Support non-linear branch graphs
* [ ] Stack reordering
* [ ] Partial stack submission
* [ ] Stack visualization

## C. Review Enhancements

* [ ] Encode stack relationships in review descriptions
* [ ] Provider-native dependency features (GitLab MR dependencies)
* [ ] Automatic title templating
* [ ] Parent / child review navigation links
* [ ] Custom description templates

## D. CI & Merge Awareness

* [ ] Inspect CI/pipeline status via provider CLI
* [ ] Merge readiness checks
* [ ] Merge train awareness (GitLab)
* [ ] Merge queue integration (GitHub)
* [ ] Optional CI gating on submission

## E. Merge & Cleanup Workflow

* [ ] Sequential merge command
* [ ] Auto-rebase remaining stack after merge
* [ ] Automatic remote branch deletion
* [ ] Local stack cleanup
* [ ] Stack archival

## F. UX Improvements

* [ ] `bt status` command with rich output
* [ ] Interactive prompts for ambiguous operations
* [ ] Dry-run mode (`--dry-run` flag)
* [ ] Shell completion scripts (bash, zsh, fish)
* [ ] Man pages

## G. Configuration & Policy

* [ ] Configurable base branch per stack
* [ ] Draft vs ready review policy
* [ ] Force-push safety controls
* [ ] Team-wide configuration support (`.basalt.toml` in repo)
* [ ] Per-provider configuration overrides

## H. Additional Providers

* [ ] Gitea/Forgejo provider
* [ ] Bitbucket provider
* [ ] Azure DevOps provider
* [ ] Generic git provider (local-only mode)

## I. Advanced Stack Operations

* [ ] Branch insertion into existing stacks
* [ ] Branch removal from stacks
* [ ] Stack splitting
* [ ] Stack merging
* [ ] Cherry-pick across stacks

## J. Performance & Optimization

* [ ] Parallel review operations where possible
* [ ] Caching of provider CLI outputs
* [ ] Incremental stack validation
* [ ] Optimistic UI updates

## K. Distribution & Installation

* [ ] Prebuilt binaries for major platforms
* [ ] Homebrew formula
* [ ] APT/RPM packages
* [ ] cargo-binstall support
* [ ] Windows installer
* [ ] Docker image

## L. Observability & Debugging

* [ ] Verbose logging mode (`-v`, `-vv`, `-vvv`)
* [ ] Debug output for provider interactions
* [ ] Git operation tracing
* [ ] Performance profiling hooks

---

## Guiding Principles

* **Explicit over implicit**: Prefer explicit behavior over clever inference
* **Fail fast and loudly**: Clear errors are better than silent failures
* **Defer complexity**: Don't build features until workflows are proven
* **Leverage existing tools**: Let Git and provider CLIs do what they do well
* **Provider parity**: All providers should support the same core operations
* **Local-first**: Never require network access for read-only operations

---

## Non-goals

* Web UI or desktop application
* Replacing provider-native features (CI, merge queues, etc.)
* Background services or daemons
* Supporting every possible Git workflow
* Direct provider API usage (always delegate to CLIs)
* Being a Git wrapper (use git directly for non-stack operations)

---

## Migration from Charcoal

For existing Charcoal users on GitHub:

1. basalt will provide a migration command to convert metadata
2. GitHub provider will maintain workflow compatibility
3. No changes required to existing Git history
4. Can run basalt and Charcoal side-by-side during transition

---

## Project Timeline

1. **Phase 1**: Rust foundation + GitLab provider (MVP)
2. **Phase 2**: GitHub provider + migration tooling
3. **Phase 3**: Enhanced stack operations
4. **Phase 4**: Additional providers
5. **Phase 5**: Advanced features & optimizations

---

*This document defines the vision and execution plan for building **basalt**, a Rust-based, multi-provider stacked change workflow tool invoked via the `bt` command.*