# Converge: Semantic Governance & Alignment

> Converge is a vision for **semantic governance**. We move from fragmented intent to unified, converged states through a deterministic alignment engine. Our mission is to provide a stable foundation for complex decision-making where human authority and AI agency coexist in a transparent, explainable ecosystem.

## Converge Application

## Quick Start

```bash
# Install
cargo install converge-application

# Start server
converge serve

# Run a job
converge run --template growth-strategy --seeds @seeds.json
```

---

## What This Is

`converge-application` is the **distribution layer** of the Converge ecosystem. It packages:

- Domain packs (growth-strategy, sdr-pipeline, etc.)
- LLM providers (Anthropic, OpenAI, etc.)
- Runtime server (HTTP/gRPC APIs)
- CLI tools

Built around the **Jobs To Be Done (JTBD)** philosophy: refocusing from tool-centric automation to business-centric outcomes.

---

## Documentation

- **Knowledgebase:** See [converge-business/knowledgebase/](../converge-business/knowledgebase/)
- **Architecture:** See [converge-business/knowledgebase/application-ARCHITECTURE.md](../converge-business/knowledgebase/application-ARCHITECTURE.md)
- **CLI Contract:** See [converge-business/knowledgebase/application-CLI_CONTRACT.md](../converge-business/knowledgebase/application-CLI_CONTRACT.md)
- **For LLMs:** See [AGENTS.md](AGENTS.md)

---

## Usage

### Start the server

```bash
# Start with defaults
converge serve

# Start with specific host and port
converge serve -H 127.0.0.1 -p 3000

# Start with specific domain packs
converge serve --packs growth-strategy,sdr-pipeline
```

### Run a job from CLI

```bash
converge run --template growth-strategy --seeds @seeds.json
```

---

## Related Projects

- [converge-platform](../converge-platform) - Core platform
- [converge-runtime](../converge-runtime) - Runtime server
- [converge-business](../converge-business) - Documentation and strategy

---

## License

MIT License - see [LICENSE](LICENSE) for details.
