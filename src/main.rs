mod error;
mod providers;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "bt")]
#[command(author, version, about, long_about = None)]
#[command(
    about = "A multi-provider CLI for managing stacked Git workflows",
    long_about = "basalt (bt) is a Rust-based CLI tool for managing stacked changes across multiple Git hosting providers like GitLab and GitHub."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize basalt in the current repository
    Init {
        /// Override auto-detected provider (gitlab, github)
        #[arg(short, long)]
        provider: Option<String>,

        /// Set the base branch (defaults to main/master)
        #[arg(short, long)]
        base_branch: Option<String>,
    },

    /// Submit the current stack as reviews (MRs/PRs)
    Submit {
        /// Submit as ready instead of draft
        #[arg(short, long)]
        ready: bool,
    },

    /// Restack (rebase) all branches in the current stack
    Restack {
        /// Continue after resolving conflicts
        #[arg(long)]
        r#continue: bool,

        /// Abort the restack operation
        #[arg(long)]
        abort: bool,
    },

    /// Show the status of the current stack
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Init {
            provider,
            base_branch,
        }) => run_init(provider, base_branch),
        Some(Commands::Submit { ready }) => run_submit(ready),
        Some(Commands::Restack { r#continue, abort }) => run_restack(r#continue, abort),
        Some(Commands::Status { json }) => run_status(json),
        None => {
            eprintln!("No command provided. Use --help for usage information.");
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_init(provider: Option<String>, base_branch: Option<String>) -> anyhow::Result<()> {
    println!("🚧 Initializing basalt repository...");
    if let Some(p) = provider {
        println!("   Provider: {}", p);
    }
    if let Some(b) = base_branch {
        println!("   Base branch: {}", b);
    }
    println!("\n⚠️  Not yet implemented - this is a placeholder");
    Ok(())
}

fn run_submit(ready: bool) -> anyhow::Result<()> {
    println!("🚧 Submitting stack...");
    if ready {
        println!("   Mode: Ready for review");
    } else {
        println!("   Mode: Draft");
    }
    println!("\n⚠️  Not yet implemented - this is a placeholder");
    Ok(())
}

fn run_restack(r#continue: bool, abort: bool) -> anyhow::Result<()> {
    println!("🚧 Restacking branches...");
    if r#continue {
        println!("   Continuing after conflict resolution");
    }
    if abort {
        println!("   Aborting restack operation");
    }
    println!("\n⚠️  Not yet implemented - this is a placeholder");
    Ok(())
}

fn run_status(json: bool) -> anyhow::Result<()> {
    println!("🚧 Checking stack status...");
    if json {
        println!("   Output format: JSON");
    }
    println!("\n⚠️  Not yet implemented - this is a placeholder");
    Ok(())
}
