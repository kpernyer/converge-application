# Development Plan - converge-application

This document outlines the mission, current status, and roadmap for `converge-application`, the distribution layer of the Converge ecosystem.

## Mission

To provide a productized, batteries-included distribution of the Converge engine that selects and composes domain packs, providers, and runtime configurations into a deployable solution.

## Current Status (v0.1.x)

- Basic distribution structure in place.
- Integration with `converge-core`, `converge-provider`, `converge-domain`, and `converge-runtime`.
- Initial CLI for starting the server and running jobs.
- Basic support for `growth-strategy` and `sdr-pipeline` domain packs.

## Priority 1: Canonical Job Families (Next 1 Month)

Deliver initial Blueprints and Packs for the 5 core job families:

- **Money** (existential): Initial "Issue Invoice" and "Collect Payment" Packs.
- **Customers** (growth): "Capture Lead" and "Qualify Deal" Blueprints (expanding on existing growth-strategy).
- **Delivery** (promise-keeping): "Track Delivery" and "Complete Delivery" invariants.
- **People** (sustainability): "Onboard Team Member" coordination loop.
- **Trust & Control** (risk): "Grant Access" authority gates.

## Priority 2: Professionalization & Ecosystem (Next 2 Months)

- [ ] Stabilize CLI interface for template management.
- [ ] Add support for dynamically loading external domain packs.
- [ ] Implement provider selection policies via configuration.
- [ ] Integrate with `converge-ledger` for audit logging persistence.

## Roadmap v1.0

- **Composition Engine**: Advanced wiring of domain packs and custom agents without code changes.
- **TUI Interface**: A terminal UI for monitoring convergence jobs in real-time.
- **Production-Ready Persistence**: First-class support for PostgreSQL/SurrealDB for job management.
- **Security Baseline**: RBAC for API endpoints and secret management for provider keys.

## Architecture Guidelines

- **Zero Semantics**: `converge-application` must not define business logic or types.
- **Composition over Inheritance**: Use configuration to wire layers, not hard-coded logic.
- **Portable by Design**: Distribution should run on bare metal, VMs, or containers identically.
