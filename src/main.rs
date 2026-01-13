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
//! # Run a job from the command line
//! converge run --template growth-strategy --seeds '[]'
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
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use converge_core::{Context, ContextKey, Engine, Fact};
use converge_domain::growth_strategy::{
    BrandSafetyInvariant, CompetitorAgent, EvaluationAgent, MarketSignalAgent,
    RequireEvaluationRationale, RequireMultipleStrategies, RequireStrategyEvaluations,
    StrategyAgent,
};
use strum::IntoEnumIterator;

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

            // Load templates from enabled packs
            let enabled_packs = packs::available_packs();
            let registry = packs::load_templates(&enabled_packs)?;
            
            // Resolve template
            let template_arc = registry.get(&template).ok_or_else(|| {
                anyhow::anyhow!("Template '{}' not found in any enabled pack", template)
            })?;
            
            // Parse seeds
            let mut context = Context::new();
            if let Some(seeds_raw) = seeds {
                let seeds_json = if seeds_raw.starts_with('@') {
                    let path = &seeds_raw[1..];
                    std::fs::read_to_string(path)
                        .map_err(|e| anyhow::anyhow!("Failed to read seed file '{}': {}", path, e))?
                } else {
                    seeds_raw
                };
                
                let seed_facts: Vec<converge_runtime::templates::SeedFact> = serde_json::from_str(&seeds_json)
                    .map_err(|e| anyhow::anyhow!("Failed to parse seeds JSON: {}", e))?;
                
                for seed in seed_facts {
                    let fact = Fact::new(
                        ContextKey::Seeds,
                        seed.id,
                        seed.content,
                    );
                    context.add_fact(fact).map_err(|e| {
                        anyhow::anyhow!("Failed to add seed fact: {}", e)
                    })?;
                }
            }

            // Report total facts across all keys
            let total_facts: usize = ContextKey::iter()
                .map(|key| context.get(key).len())
                .sum();
            info!(facts = total_facts, "Context initialized with seeds");

            // Run convergence loop inline
            let mut engine = Engine::new();
            
            // Register agents from template (Bridge to domain packs)
            register_pack_agents(&mut engine, template.as_str())?;
            
            info!("Starting convergence loop...");
            let result = engine.run(context)?;
            
            if result.converged {
                info!(cycles = result.cycles, "Job reached fixed point");
            } else {
                warn!(cycles = result.cycles, "Job halted without reaching fixed point (budget exhausted)");
            }

            // Print summary
            let final_facts: usize = ContextKey::iter()
                .map(|key| result.context.get(key).len())
                .sum();
                
            println!("\n=== Convergence Result ===");
            println!("Converged: {}", result.converged);
            println!("Total Cycles: {}", result.cycles);
            println!("Total Facts: {}", final_facts);
            println!("==========================\n");
        }
    }

    Ok(())
}

/// Register agents and invariants for a specific domain pack.
///
/// This acts as the bridge between the distribution layer and the domain packs.
fn register_pack_agents(engine: &mut Engine, pack_name: &str) -> Result<()> {
    match pack_name {
        "growth-strategy" => {
            info!(pack = %pack_name, "Registering growth-strategy agents and invariants");
            
            // Register Agents
            engine.register(MarketSignalAgent);
            engine.register(CompetitorAgent);
            engine.register(StrategyAgent);
            engine.register(EvaluationAgent);

            // Register Invariants
            engine.register_invariant(BrandSafetyInvariant::default());
            engine.register_invariant(RequireMultipleStrategies);
            engine.register_invariant(RequireStrategyEvaluations);
            engine.register_invariant(RequireEvaluationRationale);
        }
        _ => {
            warn!(pack = %pack_name, "No specific agent registration for pack");
        }
    }
    Ok(())
}

