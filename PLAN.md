# Development Plan - converge-application

This document outlines the mission, current status, and roadmap for `converge-application`, the distribution layer of the Converge ecosystem.

## Mission

To provide a productized, batteries-included distribution of the Converge engine that selects and composes domain packs, providers, and runtime configurations into a deployable solution.

## Current Status (v0.1.x)

- Basic distribution structure in place.
- Integration with `converge-core`, `converge-provider`, `converge-domain`, and `converge-runtime`.
- Initial CLI for starting the server and running jobs.
- Basic support for `growth-strategy` and `sdr-pipeline` domain packs.

## Priority 1: Professionalization (Next 2 Weeks)

- [ ] Stabilize CLI interface for template management.
- [ ] Improve error reporting for deployment-level configuration issues.
- [ ] Implement robust health checks for all composed layers.
- [ ] Standardize logging and telemetry across the distribution.

## Priority 2: Ecosystem Integration (Next 1 Month)

- [ ] Add support for dynamically loading external domain packs.
- [ ] Implement provider selection policies via configuration.
- [ ] Add Docker-based reference deployment for various cloud targets.
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
