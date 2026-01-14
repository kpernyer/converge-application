# Converge CLI Contract
## Implementation of the Cross-Platform Contract for CLI

> A Converge CLI is a Truths-driven, ledger-minded, deterministic-when-needed client that treats ML as proposals, not truth.

This document adapts the [Cross-Platform Contract](./CROSS_PLATFORM_CONTRACT.md) for the CLI context. The axioms are identical; the implementation is CLI-native.

---

## 0. Converge Axioms (Non-Negotiable)

These apply to ALL Converge clients (iOS, Android, CLI).

| Axiom | CLI Manifestation |
|-------|-------------------|
| **Monotonicity** | Context only grows. Facts append to output. `--append` flag for file output. |
| **Idempotency** | Same seeds + same context → same outcome. `--deterministic` flag enforces. |
| **Determinism** | `--mock` flag runs without LLM. Core logic never depends on ML. |
| **Convergence** | CLI blocks until fixed point or budget exhausted. Exit code reflects outcome. |
| **No Hidden State** | All state flows through context. No implicit global config. |
| **Governance Separation** | Trust pack (auth) separate from domain packs. |
| **Live Convergence** | Streaming output shows progressive facts. `--watch` for live updates. |

---

## 1. CLI Architecture

### 1.1 Transport Priority

```
CLI Transport Priority:
  1. Direct Engine (in-process) ← PRIMARY for local runs
  2. gRPC over HTTP/2          ← For remote/distributed runs
  3. REST                       ← For simple queries
```

### 1.2 Output Modes

| Mode | Flag | Behavior |
|------|------|----------|
| **Streaming** | `--stream` | Facts printed as they arrive |
| **Summary** | (default) | Final summary after convergence |
| **JSON** | `--json` | Machine-readable JSON output |
| **Quiet** | `--quiet` | Exit code only |
| **Watch** | `--watch` | Live updates (for long-running jobs) |

### 1.3 Exit Codes

| Code | Meaning | Maps to Run Status |
|------|---------|-------------------|
| 0 | Converged successfully | `converged` |
| 1 | Halted (invariant violated) | `halted` |
| 2 | Budget exhausted | `budget_exceeded` |
| 3 | Error (system failure) | `error` |
| 4 | Eval failed | (eval-specific) |

---

## 2. Run Identity

Every CLI run MUST have:

### 2.1 Run ID

```bash
# Auto-generated
converge run --template growth-strategy
# Run ID: run_a1b2c3d4-e5f6-7890-abcd-ef1234567890

# User-provided (for reproducibility)
converge run --template growth-strategy --run-id "run_my-test-001"
```

### 2.2 Correlation ID

Links related operations:

```bash
# Single command
converge run --correlation-id "batch_2024-01-14_marketing"

# All facts from this run will include correlation_id
```

### 2.3 Output Format

Every fact emitted includes:

```json
{
  "run_id": "run_abc123",
  "correlation_id": "cor_def456",
  "sequence": 1,
  "timestamp": "2024-01-14T10:30:00Z",
  "actor": {
    "type": "system",
    "cli_version": "0.1.0",
    "device_id": "cli:hostname:user"
  },
  "fact": {
    "key": "Strategies",
    "id": "strategy:smb-focus",
    "content": "Target SMB segment..."
  }
}
```

---

## 3. Actor Model

### 3.1 CLI Actor Structure

```json
{
  "actor": {
    "type": "user|system|agent",
    "user_id": "kenneth@aprio.one",
    "device_id": "cli:macbook-pro:kpernyer",
    "cli_version": "0.1.0",
    "invocation": "converge run --template growth-strategy"
  }
}
```

### 3.2 Actor Types

| Type | When Used |
|------|-----------|
| `user` | Interactive CLI session |
| `system` | Cron/CI jobs, automated runs |
| `agent` | LLM-powered agents proposing facts |

### 3.3 Device ID Format

```
cli:{hostname}:{username}
cli:macbook-pro:kpernyer
cli:github-runner:ci
```

---

## 4. Commands

### 4.1 Core Commands

```bash
# Run a convergence job
converge run --template <name> [--seeds <json>] [--mock] [--stream]

# List available packs
converge packs list

# Show pack details
converge packs info <name>

# Run eval fixtures
converge eval run [--mock] [--dir evals/]
converge eval list

# Show capabilities (what's available)
converge capabilities

# Interactive TUI
converge tui
```

### 4.2 Future Commands (Contract-Aligned)

```bash
# Watch a running job (live convergence)
converge watch <run_id>

# Inject fact into running job
converge inject --run-id <id> --fact <json>

# Approve/reject pending proposal
converge approve <proposal_id> [--reason "..."]
converge reject <proposal_id> --reason "..."

# Resume from checkpoint
converge resume --run-id <id> [--from-sequence <n>]

# Export trace
converge trace <run_id> [--format json|yaml|human]
```

---

## 5. Eval Contract

### 5.1 Eval Fixture Format (Cross-Platform Compatible)

```json
{
  "eval_id": "truth.halt_on_violation",
  "description": "System halts when invariant violated",
  "pack": "growth-strategy",
  "seeds": [
    {"id": "context", "content": "..."}
  ],
  "expected": {
    "converged": true,
    "max_cycles": 10,
    "min_facts": 15,
    "must_contain_facts": ["strategy:"],
    "must_not_contain_facts": ["error:"],
    "required_context_keys": ["Strategies", "Evaluations"],
    "max_latency_ms": 500
  },
  "use_mock_llm": true
}
```

### 5.2 Core Eval Cases (Must Pass)

| ID | Name | Validates |
|----|------|-----------|
| `convergence.reaches_fixed_point` | Run halts when no more agents fire | Convergence |
| `convergence.respects_budget` | Run halts at max_cycles | Budget control |
| `invariant.multiple_strategies` | At least N strategies produced | RequireMultipleStrategies |
| `invariant.strategy_evaluations` | All strategies have evaluations | RequireStrategyEvaluations |
| `invariant.brand_safety` | No unsafe content in output | BrandSafetyInvariant |
| `determinism.mock_reproducible` | Same seeds → same facts with mock LLM | Determinism |
| `trace.run_id_present` | All facts have run_id | Traceability |
| `trace.correlation_linked` | Related facts share correlation_id | Traceability |

### 5.3 Running Evals

```bash
# Run all evals with mock LLM (fast, deterministic)
converge eval run --mock

# Run specific eval
converge eval run convergence.reaches_fixed_point

# Run with real LLM (integration test)
converge eval run --dir evals/integration/

# CI-friendly (exit code 1 if any fail)
converge eval run --mock && echo "All evals passed"
```

---

## 6. Streaming Output

### 6.1 Progressive Facts

```bash
converge run --template growth-strategy --stream

# Output (facts arrive as they're produced):
[cycle:1] fact:signal:nordic-growth | Nordic SaaS market growing 15% YoY
[cycle:1] fact:signal:nordic-competition | 3 major competitors...
[cycle:2] fact:competitor:alpha-corp | AlphaCorp: Strong in enterprise...
[cycle:3] fact:strategy:smb-focus | Target SMB segment...
[cycle:4] fact:eval:smb-focus | Score: 85/100 | RECOMMENDED
[cycle:5] fact:insight:1 | Prioritize SMB market entry...
[cycle:5] fact:risk:1 | Resource Constraint Risk...
[cycle:6] converged | 6 cycles, 21 facts
```

### 6.2 JSON Streaming

```bash
converge run --template growth-strategy --stream --json

# Output (one JSON object per line):
{"cycle":1,"type":"fact","key":"Signals","id":"signal:nordic-growth",...}
{"cycle":2,"type":"fact","key":"Competitors","id":"competitor:alpha-corp",...}
{"cycle":6,"type":"status","converged":true,"cycles":6,"facts":21}
```

---

## 7. Determinism Mode

### 7.1 Mock LLM

```bash
# Always deterministic, no API calls
converge run --template growth-strategy --mock
```

### 7.2 Reproducibility

```bash
# Save seeds for replay
converge run --template growth-strategy --seeds @seeds.json > run_001.json

# Replay should produce identical output
converge run --template growth-strategy --seeds @seeds.json --mock > run_002.json
diff run_001.json run_002.json  # Should be empty
```

### 7.3 Fixtures for CI

```yaml
# .github/workflows/test.yml
- name: Run evals
  run: cargo run -- eval run --mock
  # Exits 1 if any eval fails
```

---

## 8. Trace Output

### 8.1 Trace Format

```bash
converge run --template growth-strategy --trace

# Output includes trace entries:
{
  "trace_id": "trc_xyz789",
  "run_id": "run_abc123",
  "correlation_id": "cor_def456",
  "actor": { "type": "agent", "name": "MarketSignalAgent" },
  "action": "produce_fact",
  "input_hash": "sha256:abc...",
  "output_hash": "sha256:def...",
  "fact_ids": ["signal:nordic-growth", "signal:nordic-competition"],
  "timestamp": "2024-01-14T10:30:00.123Z"
}
```

### 8.2 Trace to File

```bash
converge run --template growth-strategy --trace-file traces/run_001.jsonl
```

---

## 9. Capabilities Command

```bash
converge capabilities

# Output:
Converge CLI v0.1.0

Packs:
  - growth-strategy (v1.0.0) [enabled]
  - sdr-pipeline (v0.1.0) [disabled]

Agents:
  - MarketSignalAgent (deterministic)
  - CompetitorAgent (deterministic)
  - StrategyAgent (deterministic)
  - EvaluationAgent (deterministic)
  - StrategicInsightAgent (LLM-powered)
  - RiskAssessmentAgent (LLM-powered)

LLM Providers:
  - Anthropic: configured (claude-sonnet-4-20250514)
  - OpenAI: configured (gpt-4o)
  - Mock: always available

Features:
  - Determinism mode: supported
  - Streaming output: supported
  - Eval fixtures: supported
  - Trace output: supported
```

---

## 10. Implementation Checklist

### Axioms
- [x] Monotonicity: Context only grows (append-only facts)
- [x] Idempotency: Same context → same outcome (with --mock)
- [x] Determinism: Can run without LLM (--mock flag)
- [x] Convergence: Runs halt at stable state
- [x] No Hidden State: All state in context (run_id in output)
- [x] Governance Separation: Trust invariants separate from domain
- [x] Live Convergence: Streaming output (--stream flag)

### Run Identity
- [x] Run ID on every invocation
- [x] Correlation ID support
- [x] Actor model in output (type, device_id, cli_version)
- [x] Device ID generation (cli:hostname:username)

### Eval System
- [x] Eval fixtures in JSON
- [x] `converge eval run` command
- [x] `converge eval list` command
- [x] Exit code 1 on failure
- [x] Mock LLM support
- [x] Cross-platform compatible format

### Output
- [x] Streaming mode (--stream)
- [x] JSON output (--json)
- [ ] Trace output (--trace)
- [x] Quiet mode (--quiet)

### Future
- [ ] Watch command for live updates
- [ ] Inject fact command
- [ ] Approve/reject commands
- [ ] Resume from checkpoint

---

## 11. One-Liner

> **The Converge CLI is a Truths-driven, ledger-minded, deterministic-when-needed client that treats ML as proposals, not truth.**

*This contract aligns with iOS and Android. The implementation is CLI-native; the behavior is identical.*
