// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Domain pack management for Converge.
//!
//! Domain packs are defined in `converge-domain` and loaded here for
//! composition into the runtime. This module:
//!
//! - Lists available packs
//! - Loads templates from packs
//! - Provides pack metadata
//!
//! # Architecture Note
//!
//! This module does NOT define business semantics. It only selects
//! which already-defined domain packs are available in this distribution.

use anyhow::Result;
use converge_runtime::templates::TemplateRegistry;

/// Information about a domain pack.
pub struct PackInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub templates: Vec<String>,
    pub invariants: Vec<String>,
}

/// Returns all available domain packs (compiled into this distribution).
pub fn available_packs() -> Vec<String> {
    let mut packs = Vec::new();

    #[cfg(feature = "growth-strategy")]
    packs.push("growth-strategy".to_string());

    #[cfg(feature = "sdr-pipeline")]
    packs.push("sdr-pipeline".to_string());

    // Always available (core pack)
    packs.push("growth-strategy".to_string());

    // Deduplicate
    packs.sort();
    packs.dedup();
    packs
}

/// Returns the default packs to enable.
pub fn default_packs() -> Vec<String> {
    vec!["growth-strategy".to_string()]
}

/// Get information about a specific pack.
pub fn pack_info(name: &str) -> PackInfo {
    match name {
        "growth-strategy" => PackInfo {
            name: "growth-strategy".to_string(),
            description: "Multi-agent growth strategy analysis with market signals, \
                         competitor analysis, strategy synthesis, and evaluation."
                .to_string(),
            version: "1.0.0".to_string(),
            templates: vec!["growth-strategy".to_string()],
            invariants: vec![
                "BrandSafetyInvariant".to_string(),
                "RequireMultipleStrategies".to_string(),
                "RequireStrategyEvaluations".to_string(),
            ],
        },
        "sdr-pipeline" => PackInfo {
            name: "sdr-pipeline".to_string(),
            description: "SDR/sales funnel automation with lead qualification, \
                         outreach sequencing, and meeting scheduling."
                .to_string(),
            version: "0.1.0".to_string(),
            templates: vec!["sdr-qualify".to_string(), "sdr-outreach".to_string()],
            invariants: vec![
                "LeadQualificationInvariant".to_string(),
                "OutreachComplianceInvariant".to_string(),
            ],
        },
        _ => PackInfo {
            name: name.to_string(),
            description: "Unknown pack".to_string(),
            version: "0.0.0".to_string(),
            templates: vec![],
            invariants: vec![],
        },
    }
}

/// Load templates from the specified domain packs.
pub fn load_templates(packs: &[String]) -> Result<TemplateRegistry> {
    let mut registry = TemplateRegistry::new();

    for pack in packs {
        match pack.as_str() {
            "growth-strategy" => {
                // Load growth-strategy templates from converge-domain
                // For now, use the embedded default
                let default_registry = TemplateRegistry::with_defaults();
                if let Some(template) = default_registry.get("growth-strategy") {
                    registry.register((*template).clone());
                }
            }
            "sdr-pipeline" => {
                // TODO: Load SDR pipeline templates when implemented
                tracing::warn!(pack = %pack, "Pack not yet implemented");
            }
            _ => {
                tracing::warn!(pack = %pack, "Unknown pack requested");
            }
        }
    }

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_packs() {
        let packs = available_packs();
        assert!(packs.contains(&"growth-strategy".to_string()));
    }

    #[test]
    fn test_pack_info() {
        let info = pack_info("growth-strategy");
        assert_eq!(info.name, "growth-strategy");
        assert!(!info.templates.is_empty());
        assert!(!info.invariants.is_empty());
    }

    #[test]
    fn test_load_templates() {
        let registry = load_templates(&["growth-strategy".to_string()]).unwrap();
        assert!(registry.contains("growth-strategy"));
    }
}
