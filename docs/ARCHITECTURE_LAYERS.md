# Converge Architecture Layers

Converge is intentionally layered to keep **semantic authority explicit**, preserve **transparent determinism**, and avoid “agent framework drift”.

> **`converge-domain` owns meaning. `converge-runtime` owns execution. `converge-app` owns packaging.**

---

## North Star

**Stop agent drift. Converge to an explainable result.**

Converge is built around:
- **Safety by Construction** (invalid states unrepresentable)
- **Zero‑Trust Agency** (agents propose; engine decides)
- **Transparent Determinism** (every outcome is traceable and reproducible)

---

## The Layers

### 1) `converge-core` — Semantic Engine (the “kernel”)

**Responsibility**
- The convergence loop and its invariants:
  - eligibility (dependency-indexed)
  - parallel compute / serialized commit
  - merge semantics
  - monotonicity + budgets + termination
  - provenance, audit events, determinism guarantees
- Typed state model: `Context`, `Fact`, `ProposedFact`, `AgentEffect`, `HaltReason`, etc.

**Non‑goals**
- No business semantics
- No provider/connectors
- No HTTP/gRPC servers
- No workflow engines or message buses

**Rule**
> `converge-core` must remain portable, deterministic, and “boring”.  
> It is the semantic authority per root intent/job.

---

### 2) `converge-provider` — Adapters (capabilities without semantics)

**Responsibility**
- Integrations that expose *capabilities* as typed interfaces:
  - LLM providers (OpenAI/Anthropic/Gemini/local/Ollama)
  - retrieval / vector stores (LanceDB/Qdrant)
  - graph stores
  - web/search adapters
  - mail/calendar/CRM/Slack/Zapier/Shopify/Stripe/etc (when added)

**Non‑goals**
- No domain rules
- No convergence logic
- No “workflow” logic

**Rule**
> Providers can be powerful, but they are **not authoritative**.  
> They are tools called by agents; they do not define correctness.

---

### 3) `converge-domain` — Use‑cases (meaning, rules, outcomes)

**Responsibility**
- Domain packs: reusable, opinionated “templates” for real business outcomes:
  - CRM/SDR funnel automation
  - HR policy rollouts & acknowledgements
  - growth strategy pipelines
  - catalog updates, sourcing, inventory checks, etc.
- Gherkin as **business intent** and invariants:
  - acceptance criteria (“done means…”)
  - governance rules (promotion, gating, HITL)
  - semantic constraints (brand safety, compliance)
- Compilation/translation into **typed** rules/agents/invariants:
  - Gherkin → Rust types (no runtime magic)
  - domain-specific invariants registered into engine

**Non‑goals**
- No deployment concerns
- No API gateways
- No secret management / tenancy

**Rule**
> Domain is where **meaning lives** and where Converge becomes a product.  
> If it affects “what should happen”, it belongs here.

---

### 4) `converge-runtime` — Execution Service (API + job lifecycle)

**Responsibility**
- Expose Converge to external callers (web/apps/CLI/TUI/other services):
  - REST/OpenAPI now; gRPC later; GraphQL/TUI optional
- Job lifecycle:
  - create/validate job
  - run inline convergence (short jobs)
  - persist / resume (HITL waits)
  - stream progress and results
- Template registry + override merging (hybrid model)
- Observability surfaces:
  - cycle traces
  - audit/provenance export
  - invariant violation reporting
  - model usage metrics (tokens, provider, latency) as telemetry (non-authoritative)

**Non‑goals**
- No business semantics (runtime must not invent “CRM objects”)
- No domain-specific branching logic
- No “hidden workflows” or background orchestration that breaks determinism

**Rule**
> Runtime is a **thin execution boundary** around domain + core.  
> It should wire jobs, not define meaning.

---

### 5) `converge-app` — Distribution (packaging + composition)

**Responsibility**
- A productized distribution that selects:
  - which domain packs are available
  - which providers are enabled
  - runtime deployment configs (auth, tenancy, quotas, storage)
- Composition, not semantics:
  - choose which templates exist
  - choose safe defaults
  - configure policy and limits

**Hard rule**
> `converge-app` must not invent business types, business rules, or new DSLs.  
> It composes **already defined** domain meaning.

---

## Dependency Direction (must remain acyclic)

```
converge-core
   ▲
   │ depends on core
converge-provider
   ▲
   │ depends on core + provider
converge-domain
   ▲
   │ depends on core + provider + domain
converge-runtime
   ▲
   │ depends on runtime + domain (+ provider) + core
converge-app
```

Recommended “imports”:
- `converge-provider` → `converge-core` (types & error model)
- `converge-domain` → `converge-core` + `converge-provider`
- `converge-runtime` → `converge-core` + `converge-domain` (+ `converge-provider` if needed for config discovery)
- `converge-app` → `converge-runtime` (plus domain/provider selection)

---

## Producer / Consumer Model for Specs

**Producer**
- Humans (founders/operators/customers)
- Domain authors
- LLM-assisted drafting tools

**Consumer**
- `converge-domain` compilers (Gherkin → typed invariants/agents)
- `converge-runtime` planners (templates + overrides → executable job)
- `converge-core` engine (job → convergence)

> Gherkin is produced by humans and consumed by **domain** compilers—not by runtime directly.

---

## Gherkin vs YAML/JSON

### Use **Gherkin** for semantics
Use it when expressing:
- invariants (“must always hold”)
- acceptance criteria (“good enough means…”)
- governance rules (promotion, rejection, HITL gating)
- compliance / brand safety / audit requirements

**Why**
- readable by humans
- composable into tests
- compiles into typed predicates (not “interpreted vibes”)

### Use **YAML/JSON** for wiring (surrounding configuration)
Use it for:
- selecting templates
- budgets/timeouts/limits
- provider selection policy (prefer/exclude)
- deployment-level config (ports, storage backend)
- feature flags and enabled packs

**Rule of thumb**
> **Gherkin defines what must be true.**  
> **YAML defines how this instance is wired.**

---

## The Compilation Pipeline

1) **Gherkin** (business intent)
2) `converge-domain` **compile** → typed invariants + agent configs + validators
3) `converge-runtime` **plan** → load template + apply overrides + resolve providers
4) `converge-core` **execute** → converge, enforce invariants, halt states, provenance

---

## Minimal Example: “Template + Gherkin + Execution”

### A) Domain intent in Gherkin (semantic contract)

```gherkin
Feature: Growth strategy must be safe and evaluable

  Scenario: Strategy generation produces multiple strategies
    Given a market seed "market:nordic-b2b"
    And a product seed "product:converge"
    When the system converges
    Then there must be at least 2 Strategies
    And every Strategy must have an Evaluation
    And no Strategy may contain forbidden terms
```

### B) Runtime job request (wiring)

```json
{
  "template": "growth-strategy",
  "overrides": {
    "budget": { "max_cycles": 60, "max_facts": 800 },
    "seeds": [
      { "id": "market:nordic-b2b", "content": "Nordic B2B SaaS landscape" },
      { "id": "product:converge", "content": "Convergence engine for agentic workflows" }
    ],
    "validation": { "min_confidence": 0.75 }
  },
  "providers": {
    "prefer": ["anthropic", "openai"],
    "exclude": ["perplexity"]
  }
}
```

### C) What the runtime does (high level)
- loads template `"growth-strategy"` from registry
- merges overrides (deep merge rules)
- resolves provider selection policy
- instantiates agents (from domain pack) with provider handles (from provider layer)
- runs `Engine::run_until_halt()` (core)
- returns result + provenance + usage metrics

---

## Anti‑Patterns (things to avoid)

### 1) App inventing types
❌ `converge-app` defines `Lead`, `DealStage`, `Invoice` structs  
✅ those belong in `converge-domain` (or a dedicated domain pack crate)

### 2) Runtime inventing semantics
❌ runtime has branching logic: “if stage=SQL then schedule meeting”  
✅ runtime should expose “run template X with overrides Y”; domain owns the logic

### 3) Providers making decisions
❌ provider decides validity or “final truth”  
✅ provider returns data; domain + core validate and merge

### 4) Hidden state in agents
❌ `has_run` flags, counters, internal caches controlling behavior  
✅ idempotency derived from context facts

### 5) YAML as a new semantic language
❌ complex “rules” expressed via YAML condition trees  
✅ prefer Gherkin → typed compilation when expressing meaning

---

## Versioning & Stability

### `converge-core`
- Treat as the kernel: strict semver, conservative changes
- Backward compatibility is a feature

### `converge-provider`
- Can evolve faster; still stable contracts
- Provider adapters should be testable with fakes

### `converge-domain`
- Evolve rapidly; it is the product surface
- Keep packs versioned and traceable

### `converge-runtime`
- Stable API contract (OpenAPI), versioned endpoints
- Job templates are versioned artifacts

---

## Testing & “Bones”

Each layer should prove different things:

- **core:** axioms (determinism, idempotency, no hidden state, starvation contracts)
- **provider:** boundary tests + fake providers
- **domain:** end-to-end convergence for use cases + semantic evals
- **runtime:** API contract tests + template registry tests + persistence/resume tests
- **app:** smoke tests for packaged distribution + golden paths

---

## One-line Definition (for docs & homepage)

> Converge is a semantic convergence engine where **agents propose**, **the engine decides**, and **every result can be explained**.
