// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Converge App - Distribution & Packaging Layer
//!
//! This is the deployable product that:
//! - Selects which domain packs are available
//! - Configures which providers are enabled
//! - Sets runtime deployment defaults (auth, tenancy, quotas)
//! - Bootstraps the converge-runtime server
//!
//! # Architecture Role
//!
//! > `converge-app` owns **packaging**, not **semantics**.
//!
//! This crate composes already-defined domain meaning from `converge-domain`.
//! It does NOT invent new business types, rules, or DSLs.
//!
//! # Usage
//!
//! ```bash
//! # Start the server with defaults
//! converge serve
//!
//! # Start with specific domain packs
//! converge serve --packs growth-strategy,sdr-pipeline
//!
//! # List available domain packs
//! converge packs list
//! ```

#![allow(dead_code)]
#![allow(unused_variables)]

mod config;
mod packs;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

/// Converge - Semantic convergence engine for agentic workflows
#[derive(Parser)]
#[command(name = "converge")]
#[command(about = "Converge Agent OS - where agents propose and the engine decides")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Converge server
    Serve {
        /// Host to bind to
        #[arg(short = 'H', long, default_value = "0.0.0.0", env = "CONVERGE_HOST")]
        host: String,

        /// Port to bind to
        #[arg(short, long, default_value = "8080", env = "CONVERGE_PORT")]
        port: u16,

        /// Domain packs to enable (comma-separated)
        #[arg(long, env = "CONVERGE_PACKS")]
        packs: Option<String>,

        /// Enable all available domain packs
        #[arg(long)]
        all_packs: bool,
    },

    /// Manage domain packs
    Packs {
        #[command(subcommand)]
        command: PacksCommands,
    },

    /// Run a job from the command line
    Run {
        /// Template to use
        #[arg(short, long)]
        template: String,

        /// Seeds as JSON (or @file.json)
        #[arg(short, long)]
        seeds: Option<String>,

        /// Max cycles budget
        #[arg(long, default_value = "50")]
        max_cycles: u32,
    },
}

#[derive(Subcommand)]
enum PacksCommands {
    /// List available domain packs
    List,
    /// Show details of a specific pack
    Info {
        /// Pack name
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            host,
            port,
            packs,
            all_packs,
        } => {
            info!("Starting Converge server on {}:{}", host, port);

            // Determine which packs to enable
            let enabled_packs = if all_packs {
                packs::available_packs()
            } else if let Some(pack_list) = packs {
                pack_list.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                // Default: enable all compiled-in packs
                packs::default_packs()
            };

            info!(packs = ?enabled_packs, "Domain packs enabled");

            // Build app configuration
            let app_config = config::AppConfig {
                host,
                port,
                enabled_packs,
                ..Default::default()
            };

            // Start server
            serve(app_config).await?;
        }

        Commands::Packs { command } => match command {
            PacksCommands::List => {
                println!("Available domain packs:\n");
                for pack in packs::available_packs() {
                    let info = packs::pack_info(&pack);
                    println!("  {} - {}", pack, info.description);
                }
            }
            PacksCommands::Info { name } => {
                let info = packs::pack_info(&name);
                println!("Pack: {}", name);
                println!("Description: {}", info.description);
                println!("Version: {}", info.version);
                println!("\nTemplates:");
                for template in &info.templates {
                    println!("  - {}", template);
                }
                println!("\nInvariants:");
                for invariant in &info.invariants {
                    println!("  - {}", invariant);
                }
            }
        },

        Commands::Run {
            template,
            seeds,
            max_cycles,
        } => {
            info!(template = %template, "Running job from CLI");

            // TODO: Parse seeds, create job request, run inline
            println!("CLI job execution not yet implemented");
            println!("Use the HTTP API: POST /api/v1/templates/jobs");
        }
    }

    Ok(())
}

/// Start the Converge server with the given configuration.
async fn serve(config: config::AppConfig) -> Result<()> {
    use converge_runtime::http::HttpServer;
    use converge_runtime::state::AppState;
    use std::net::SocketAddr;

    // Create template registry from enabled domain packs
    let templates = packs::load_templates(&config.enabled_packs)?;

    // Create app state with templates
    let state = AppState::with_templates(templates);

    // Create HTTP config
    let bind_addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let http_config = converge_runtime::config::HttpConfig {
        bind: bind_addr,
        max_body_size: 10 * 1024 * 1024, // 10 MB
    };

    // Start server
    info!(bind = %bind_addr, "Starting Converge server");
    let server = HttpServer::new(http_config, state);
    server.start().await?;

    Ok(())
}
