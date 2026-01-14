// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT

//! Eval Fixtures for Converge
//!
//! This module implements reproducible evaluation testing based on the
//! cross-platform contract pattern from iOS/Android implementations.
//!
//! Eval fixtures define:
//! - Input seeds for a convergence run
//! - Expected outcomes (convergence, cycle count, required facts)
//! - Pass/fail criteria with specific thresholds
//!
//! # Usage
//!
//! ```bash
//! # Run all evals
//! converge eval run
//!
//! # Run specific eval
//! converge eval run growth_strategy_smb_001
//!
//! # List available evals
//! converge eval list
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

use converge_core::{Context as ConvergeContext, ContextKey, Engine, Fact};
use converge_core::llm::LlmProvider;
use converge_provider::{AnthropicProvider, OpenAiProvider};
use strum::IntoEnumIterator;

use crate::agents::{MockInsightProvider, RiskAssessmentAgent, StrategicInsightAgent};
use converge_domain::growth_strategy::{
    BrandSafetyInvariant, CompetitorAgent, EvaluationAgent, MarketSignalAgent,
    RequireEvaluationRationale, RequireMultipleStrategies, RequireStrategyEvaluations,
    StrategyAgent,
};

/// A seed fact for the eval fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedFact {
    pub id: String,
    pub content: String,
}

/// Expected outcomes for an eval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalExpectation {
    /// Must converge (reach fixed point)
    #[serde(default)]
    pub converged: Option<bool>,

    /// Maximum allowed cycles
    #[serde(default)]
    pub max_cycles: Option<u32>,

    /// Minimum number of facts produced
    #[serde(default)]
    pub min_facts: Option<usize>,

    /// Facts that must be present (by ID prefix)
    #[serde(default)]
    pub must_contain_facts: Vec<String>,

    /// Facts that must NOT be present (by ID prefix)
    #[serde(default)]
    pub must_not_contain_facts: Vec<String>,

    /// Minimum number of strategies generated
    #[serde(default)]
    pub min_strategies: Option<usize>,

    /// Minimum number of evaluations generated
    #[serde(default)]
    pub min_evaluations: Option<usize>,

    /// Maximum latency in milliseconds
    #[serde(default)]
    pub max_latency_ms: Option<u64>,

    /// Context keys that must have facts
    #[serde(default)]
    pub required_context_keys: Vec<String>,
}

/// An eval fixture defining a test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalFixture {
    /// Unique identifier for this eval
    pub eval_id: String,

    /// Human-readable description
    pub description: String,

    /// Pack to use for this eval
    pub pack: String,

    /// Input seeds
    pub seeds: Vec<SeedFact>,

    /// Expected outcomes
    pub expected: EvalExpectation,

    /// Whether to use mock LLM (faster, deterministic)
    #[serde(default)]
    pub use_mock_llm: bool,
}

/// Result of running an eval
#[derive(Debug, Clone)]
pub struct EvalResult {
    /// The eval that was run
    pub eval_id: String,

    /// Unique run ID for tracing
    pub run_id: Uuid,

    /// Whether the eval passed all expectations
    pub passed: bool,

    /// Individual check results
    pub checks: Vec<EvalCheck>,

    /// Actual cycle count
    pub cycles: u32,

    /// Actual fact count
    pub fact_count: usize,

    /// Whether convergence was reached
    pub converged: bool,

    /// Total run duration
    pub duration: Duration,

    /// Error message if run failed
    pub error: Option<String>,
}

/// Individual check within an eval
#[derive(Debug, Clone)]
pub struct EvalCheck {
    pub name: String,
    pub passed: bool,
    pub expected: String,
    pub actual: String,
}

impl EvalResult {
    /// Create a failed result due to error
    pub fn error(eval_id: &str, run_id: Uuid, error: String, duration: Duration) -> Self {
        Self {
            eval_id: eval_id.to_string(),
            run_id,
            passed: false,
            checks: vec![],
            cycles: 0,
            fact_count: 0,
            converged: false,
            duration,
            error: Some(error),
        }
    }
}

/// Load an eval fixture from a JSON file
pub fn load_fixture(path: &Path) -> Result<EvalFixture> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read fixture file: {}", path.display()))?;

    let fixture: EvalFixture = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse fixture JSON: {}", path.display()))?;

    Ok(fixture)
}

/// Load all fixtures from a directory
pub fn load_fixtures_from_dir(dir: &Path) -> Result<Vec<EvalFixture>> {
    let mut fixtures = Vec::new();

    if !dir.exists() {
        return Ok(fixtures);
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            match load_fixture(&path) {
                Ok(fixture) => fixtures.push(fixture),
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "Failed to load fixture");
                }
            }
        }
    }

    // Sort by eval_id for consistent ordering
    fixtures.sort_by(|a, b| a.eval_id.cmp(&b.eval_id));

    Ok(fixtures)
}

/// Run a single eval fixture
pub fn run_eval(fixture: &EvalFixture) -> EvalResult {
    let run_id = Uuid::new_v4();
    let start = Instant::now();

    tracing::info!(
        eval_id = %fixture.eval_id,
        run_id = %run_id,
        pack = %fixture.pack,
        "Starting eval run"
    );

    // Build context from seeds
    let mut context = ConvergeContext::new();
    for seed in &fixture.seeds {
        let fact = Fact::new(ContextKey::Seeds, &seed.id, &seed.content);
        if let Err(e) = context.add_fact(fact) {
            return EvalResult::error(
                &fixture.eval_id,
                run_id,
                format!("Failed to add seed: {}", e),
                start.elapsed(),
            );
        }
    }

    // Create engine and register agents
    let mut engine = Engine::new();
    if let Err(e) = register_pack_agents(&mut engine, &fixture.pack, fixture.use_mock_llm) {
        return EvalResult::error(
            &fixture.eval_id,
            run_id,
            format!("Failed to register agents: {}", e),
            start.elapsed(),
        );
    }

    // Run convergence
    let result = match engine.run(context) {
        Ok(r) => r,
        Err(e) => {
            return EvalResult::error(
                &fixture.eval_id,
                run_id,
                format!("Engine run failed: {}", e),
                start.elapsed(),
            );
        }
    };

    let duration = start.elapsed();

    // Collect facts
    let all_facts: Vec<_> = ContextKey::iter()
        .flat_map(|key| result.context.get(key).to_vec())
        .collect();

    let fact_count = all_facts.len();
    let strategy_count = result.context.get(ContextKey::Strategies).len();
    let evaluation_count = result.context.get(ContextKey::Evaluations).len();

    // Run checks
    let mut checks = Vec::new();
    let expected = &fixture.expected;

    // Check: converged
    if let Some(expected_converged) = expected.converged {
        checks.push(EvalCheck {
            name: "converged".to_string(),
            passed: result.converged == expected_converged,
            expected: expected_converged.to_string(),
            actual: result.converged.to_string(),
        });
    }

    // Check: max_cycles
    if let Some(max_cycles) = expected.max_cycles {
        checks.push(EvalCheck {
            name: "max_cycles".to_string(),
            passed: result.cycles <= max_cycles,
            expected: format!("<= {}", max_cycles),
            actual: result.cycles.to_string(),
        });
    }

    // Check: min_facts
    if let Some(min_facts) = expected.min_facts {
        checks.push(EvalCheck {
            name: "min_facts".to_string(),
            passed: fact_count >= min_facts,
            expected: format!(">= {}", min_facts),
            actual: fact_count.to_string(),
        });
    }

    // Check: min_strategies
    if let Some(min_strategies) = expected.min_strategies {
        checks.push(EvalCheck {
            name: "min_strategies".to_string(),
            passed: strategy_count >= min_strategies,
            expected: format!(">= {}", min_strategies),
            actual: strategy_count.to_string(),
        });
    }

    // Check: min_evaluations
    if let Some(min_evaluations) = expected.min_evaluations {
        checks.push(EvalCheck {
            name: "min_evaluations".to_string(),
            passed: evaluation_count >= min_evaluations,
            expected: format!(">= {}", min_evaluations),
            actual: evaluation_count.to_string(),
        });
    }

    // Check: max_latency_ms
    if let Some(max_latency_ms) = expected.max_latency_ms {
        let actual_ms = duration.as_millis() as u64;
        checks.push(EvalCheck {
            name: "max_latency_ms".to_string(),
            passed: actual_ms <= max_latency_ms,
            expected: format!("<= {}ms", max_latency_ms),
            actual: format!("{}ms", actual_ms),
        });
    }

    // Check: must_contain_facts
    for fact_prefix in &expected.must_contain_facts {
        let found = all_facts.iter().any(|f| f.id.starts_with(fact_prefix));
        checks.push(EvalCheck {
            name: format!("contains:{}", fact_prefix),
            passed: found,
            expected: format!("fact with prefix '{}'", fact_prefix),
            actual: if found { "found".to_string() } else { "not found".to_string() },
        });
    }

    // Check: must_not_contain_facts
    for fact_prefix in &expected.must_not_contain_facts {
        let found = all_facts.iter().any(|f| f.id.starts_with(fact_prefix));
        checks.push(EvalCheck {
            name: format!("excludes:{}", fact_prefix),
            passed: !found,
            expected: format!("no fact with prefix '{}'", fact_prefix),
            actual: if found { "found (unexpected)".to_string() } else { "not found (good)".to_string() },
        });
    }

    // Check: required_context_keys
    for key_name in &expected.required_context_keys {
        let key = match key_name.as_str() {
            "Seeds" => Some(ContextKey::Seeds),
            "Signals" => Some(ContextKey::Signals),
            "Competitors" => Some(ContextKey::Competitors),
            "Strategies" => Some(ContextKey::Strategies),
            "Evaluations" => Some(ContextKey::Evaluations),
            "Hypotheses" => Some(ContextKey::Hypotheses),
            "Constraints" => Some(ContextKey::Constraints),
            _ => None,
        };

        if let Some(context_key) = key {
            let has_facts = !result.context.get(context_key).is_empty();
            checks.push(EvalCheck {
                name: format!("has_key:{}", key_name),
                passed: has_facts,
                expected: format!("{} has facts", key_name),
                actual: if has_facts { "has facts".to_string() } else { "empty".to_string() },
            });
        }
    }

    // Determine overall pass/fail
    let passed = checks.iter().all(|c| c.passed);

    tracing::info!(
        eval_id = %fixture.eval_id,
        run_id = %run_id,
        passed = passed,
        cycles = result.cycles,
        facts = fact_count,
        duration_ms = duration.as_millis(),
        "Eval run completed"
    );

    EvalResult {
        eval_id: fixture.eval_id.clone(),
        run_id,
        passed,
        checks,
        cycles: result.cycles,
        fact_count,
        converged: result.converged,
        duration,
        error: None,
    }
}

/// Run multiple eval fixtures
pub fn run_evals(fixtures: &[EvalFixture]) -> Vec<EvalResult> {
    fixtures.iter().map(run_eval).collect()
}

/// Creates an LLM provider (real or mock based on flag)
fn create_llm_provider(use_mock: bool) -> Arc<dyn LlmProvider> {
    if use_mock {
        return Arc::new(MockInsightProvider::default_insights()) as Arc<dyn LlmProvider>;
    }

    tokio::task::block_in_place(|| {
        // Try Anthropic first
        if let Ok(provider) = AnthropicProvider::from_env("claude-sonnet-4-20250514") {
            return Arc::new(provider) as Arc<dyn LlmProvider>;
        }

        // Try OpenAI second
        if let Ok(provider) = OpenAiProvider::from_env("gpt-4o") {
            return Arc::new(provider) as Arc<dyn LlmProvider>;
        }

        // Fall back to mock provider
        Arc::new(MockInsightProvider::default_insights()) as Arc<dyn LlmProvider>
    })
}

/// Register agents for a pack
fn register_pack_agents(engine: &mut Engine, pack_name: &str, use_mock_llm: bool) -> Result<()> {
    match pack_name {
        "growth-strategy" => {
            // Register deterministic agents
            engine.register(MarketSignalAgent);
            engine.register(CompetitorAgent);
            engine.register(StrategyAgent);
            engine.register(EvaluationAgent);

            // Create LLM provider
            let llm_provider = create_llm_provider(use_mock_llm);

            // Register LLM-powered agents
            engine.register(StrategicInsightAgent::new(llm_provider.clone()));
            engine.register(RiskAssessmentAgent::new(llm_provider));

            // Register Invariants
            engine.register_invariant(BrandSafetyInvariant::default());
            engine.register_invariant(RequireMultipleStrategies);
            engine.register_invariant(RequireStrategyEvaluations);
            engine.register_invariant(RequireEvaluationRationale);
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown pack: {}", pack_name));
        }
    }
    Ok(())
}

/// Print eval results in a formatted way
pub fn print_results(results: &[EvalResult]) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    println!("\n=== Eval Results ===\n");

    for result in results {
        let status = if result.passed { "PASS" } else { "FAIL" };
        let status_color = if result.passed { "\x1b[32m" } else { "\x1b[31m" };
        let reset = "\x1b[0m";

        println!(
            "[{}{}{}] {} ({}ms, {} cycles, {} facts)",
            status_color, status, reset,
            result.eval_id,
            result.duration.as_millis(),
            result.cycles,
            result.fact_count,
        );

        if let Some(ref error) = result.error {
            println!("      Error: {}", error);
        }

        // Show failed checks
        for check in &result.checks {
            if !check.passed {
                println!(
                    "      {}FAIL{}: {} - expected {}, got {}",
                    "\x1b[31m", reset,
                    check.name,
                    check.expected,
                    check.actual
                );
            }
        }
    }

    println!("\n===================");
    println!(
        "Total: {} | {}Passed: {}{} | {}Failed: {}{}",
        total,
        "\x1b[32m", passed, "\x1b[0m",
        if failed > 0 { "\x1b[31m" } else { "\x1b[0m" }, failed, "\x1b[0m"
    );
    println!("===================\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_parsing() {
        let json = r#"{
            "eval_id": "test_001",
            "description": "Test fixture",
            "pack": "growth-strategy",
            "seeds": [
                {"id": "seed1", "content": "Test seed"}
            ],
            "expected": {
                "converged": true,
                "max_cycles": 10
            },
            "use_mock_llm": true
        }"#;

        let fixture: EvalFixture = serde_json::from_str(json).unwrap();
        assert_eq!(fixture.eval_id, "test_001");
        assert_eq!(fixture.seeds.len(), 1);
        assert_eq!(fixture.expected.converged, Some(true));
        assert!(fixture.use_mock_llm);
    }

    #[test]
    fn test_eval_check_logic() {
        let check = EvalCheck {
            name: "test".to_string(),
            passed: true,
            expected: "foo".to_string(),
            actual: "foo".to_string(),
        };
        assert!(check.passed);
    }
}
