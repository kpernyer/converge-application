# AGENTS.md — Converge Application Guide

> Converge is a vision for **semantic governance**. We move from fragmented intent to unified, converged states through a deterministic alignment engine. Our mission is to provide a stable foundation for complex decision-making where human authority and AI agency coexist in a transparent, explainable ecosystem.

**For AI coding assistants (Claude, Gemini, Codex, Cursor, etc.)**

This repository handles the **distribution and CLI** aspects of the Converge ecosystem.

## Project Specifics

- **Distribution**: This is the entry point for users. It packages the platform, runtime, and domain packs.
- **CLI Design**: Follow the [application-CLI_CONTRACT.md](../converge-business/knowledgebase/application-CLI_CONTRACT.md).
- **JTBD**: Focus on business outcomes (Jobs To Be Done) rather than raw technical primitives.

## Development Patterns

- **Command Implementation**: Use `clap` for CLI command definitions.
- **Integration**: Ensure new domain packs from `converge-platform` are properly wired into the CLI.
- **Cross-Platform**: Adhere to the [application-CROSS_PLATFORM_CONTRACT.md](../converge-business/knowledgebase/application-CROSS_PLATFORM_CONTRACT.md).

---

## Consolidated Documentation (converge-business)

- **Knowledgebase**: [converge-business/knowledgebase/](../converge-business/knowledgebase/)
- **Application Architecture**: [converge-business/knowledgebase/application-ARCHITECTURE.md](../converge-business/knowledgebase/application-ARCHITECTURE.md)
