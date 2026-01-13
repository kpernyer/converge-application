# Converge Application

Converge distribution - packages domain packs, providers, and runtime into a deployable product.

## Overview

`converge-application` is the distribution layer of the Converge ecosystem. It composes:

- **Domain packs** from `converge-domain` (business use-cases and templates)
- **Providers** from `converge-provider` (LLM adapters, integrations)
- **Runtime** from `converge-runtime` (HTTP API, job lifecycle)
- **Core engine** from `converge-core` (semantic convergence)

> **Architecture principle**: `converge-application` owns **packaging**, not **semantics**.

## Installation

```bash
cargo install converge-application
```

Or build from source:

```bash
git clone https://github.com/kpernyer/converge-application
cd converge-application
cargo build --release
```

## Usage

### Start the server

```bash
# Start with defaults
converge serve

# Start with specific host and port
converge serve -H 127.0.0.1 -p 3000

# Start with specific domain packs
converge serve --packs growth-strategy,sdr-pipeline

# Start with all available packs
converge serve --all-packs
```

### Manage domain packs

```bash
# List available domain packs
converge packs list

# Show details of a specific pack
converge packs info growth-strategy
```

### Run a job from CLI

```bash
converge run --template growth-strategy --seeds @seeds.json
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CONVERGE_HOST` | Server bind address | `0.0.0.0` |
| `CONVERGE_PORT` | Server port | `8080` |
| `CONVERGE_PACKS` | Enabled domain packs (comma-separated) | `growth-strategy` |

## Architecture

See [docs/ARCHITECTURE_LAYERS.md](docs/ARCHITECTURE_LAYERS.md) for the full architecture overview.

```
converge-core        <- Semantic engine (the "kernel")
   ^
   |
converge-provider    <- Adapters (capabilities without semantics)
   ^
   |
converge-domain      <- Use-cases (meaning, rules, outcomes)
   ^
   |
converge-runtime     <- Execution service (API + job lifecycle)
   ^
   |
converge-application <- Distribution (packaging + composition)
```

## Available Domain Packs

| Pack | Description |
|------|-------------|
| `growth-strategy` | Multi-agent growth strategy analysis with market signals, competitor analysis, strategy synthesis, and evaluation |
| `sdr-pipeline` | SDR/sales funnel automation with lead qualification, outreach sequencing, and meeting scheduling |

## Features

Enable features in your build:

```toml
[dependencies]
converge-application = { version = "0.1", features = ["full"] }
```

Available features:
- `growth-strategy` - Growth strategy domain pack
- `sdr-pipeline` - SDR pipeline domain pack
- `full` - All domain packs

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read our [Code of Conduct](CODE_OF_CONDUCT.md) first.

## Author

Kenneth Pernyer - [kenneth@aprio.one](mailto:kenneth@aprio.one)

Copyright 2024-2025 Aprio One AB, Sweden
