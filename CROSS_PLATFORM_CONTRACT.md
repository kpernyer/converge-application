# Converge Mobile Contract
## A Truths-Driven, Ledger-Minded Client

> A Converge mobile app is a Truths-driven, ledger-minded, deterministic-when-needed client that treats ML as proposals, not truth.

This document defines how mobile apps (iOS, Android, CLI) integrate with Converge. It is **not** generic mobile advice. It is explicitly grounded in the Converge axioms.

---

## 0. Converge Axioms

These are non-negotiable. Everything else derives from them.

### 0.1 Monotonicity
Context only grows. Facts accumulate. We never delete truth—we append corrections.

### 0.2 Idempotency via Context
Given the same context, the same run produces the same outcome. Replays are safe.

### 0.3 Determinism Mode
The system can run without LLMs. Truths + Rules → Outcome. ML proposes; Truths decide.

### 0.4 Convergence Fixed Point
Runs halt when no more Truths can fire. The system finds a stable state, or explains why it can't.

### 0.5 No Hidden State
If it affects business state, it exists in context. UI state is derived, never authoritative.

### 0.6 Governance Separation
Who can do what is separate from what should happen. Trust pack governs access; other packs govern outcomes.

### 0.7 Live Convergence
Convergence is not request/response. It is a **live process** where facts arrive progressively, other actors can inject input mid-run, and "done" means stable state reached.

---

## 1. Stream-First Architecture

**This is non-negotiable.** Converge is not request/response. It is live convergence.

### 1.1 The Fundamental Shift

| Traditional App | Converge App |
|-----------------|--------------|
| Request → Wait → Response → Done | Proposal → Stream opens → Facts arrive progressively → Stable state |
| Single actor (me) | Multiple actors (me, colleagues, agents, system) |
| "Done" = response received | "Done" = convergence halted at stable state |
| Poll for updates | Subscribe to context stream |
| UI shows loading spinner | UI shows **partial/early answers**, refines progressively |

### 1.2 Why Streaming is Mandatory

In a Converge run:
1. You submit a proposal
2. **Early facts arrive** — partial answers, preliminary decisions
3. **Other humans inject input** — approvals, corrections, additional context
4. **Truths fire progressively** — each fact may trigger more truths
5. **Convergence halts** — stable state reached, or explanation why not

**The mobile app must handle all of this live.**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Time →                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  You          Other Human       System           Convergence                 │
│   │                │               │                  │                      │
│   │  proposal      │               │                  │                      │
│   ├───────────────────────────────────────────────────►                      │
│   │                │               │                  │                      │
│   │                │               │    early_fact    │                      │
│   ◄───────────────────────────────────────────────────┤                      │
│   │                │               │                  │                      │
│   │                │  approval     │                  │                      │
│   │                ├──────────────────────────────────►                      │
│   │                │               │                  │                      │
│   │                │               │   more_facts     │                      │
│   ◄───────────────────────────────────────────────────┤                      │
│   │                │               │                  │                      │
│   │                │               │  stable_state    │                      │
│   ◄───────────────────────────────────────────────────┤                      │
│   │                │               │                  │                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.3 Stream-First Rules

| Rule | Meaning |
|------|---------|
| **Subscribe, don't poll** | Open a stream on session start, keep it open |
| **Handle partial answers** | UI must render incomplete state gracefully |
| **Expect other actors** | Facts can arrive from humans you didn't invoke |
| **Stable state, not response** | "Done" is when run.status = `converged` or `halted` |
| **Progressive refinement** | Early answers may be revised by later facts |

### 1.4 Transport Priority

```
1. gRPC bidirectional streaming  ← PRIMARY
2. Server-Sent Events (SSE)      ← FALLBACK for restricted networks
3. REST + polling                ← DEGRADED MODE (offline, or SSE blocked)
```

**REST is always available as fallback.** But it is degraded mode. The app must clearly indicate when operating in degraded mode.

### 1.5 Connection States

| State | Meaning | UI Indication |
|-------|---------|---------------|
| `streaming` | Live connection, receiving facts | Green indicator |
| `reconnecting` | Temporary disconnect, reconnecting | Yellow indicator |
| `degraded` | REST fallback, polling | Orange indicator + "Limited mode" |
| `offline` | No connection, queue only | Red indicator + "Offline" |

### 1.6 Early Answers

The app **must** handle early/partial answers:

```swift
// WRONG: Wait for final answer
let result = await api.createInvoice(data)  // Blocks until "done"
showResult(result)

// RIGHT: Subscribe and handle progressively
context.watch(correlationId: proposal.correlationId)
    .sink { entry in
        switch entry.type {
        case .fact:
            // Update UI with new fact (may be partial)
            updateUI(with: entry)
        case .decision where entry.payload.status == "converged":
            // Convergence complete
            showFinalState()
        case .decision where entry.payload.status == "halted":
            // Halted, show explanation
            showHaltExplanation(entry.payload.reason)
        default:
            break
        }
    }
```

### 1.7 Multi-Actor Input

Facts can arrive from actors you didn't invoke:

| Actor | Example |
|-------|---------|
| Colleague | Approves your invoice while you wait |
| Agent | Background process completes a prerequisite |
| System | Scheduled job closes a period |
| External | Webhook from payment provider |

**Your UI must react to facts from any actor**, not just responses to your requests.

### 1.8 Run Lifecycle

A run is not a request. It is a convergence process:

```json
{
  "run_id": "run_abc123",
  "status": "running|converged|halted|waiting",
  "facts_count": 12,
  "pending_proposals": 2,
  "waiting_for": ["approval from @manager"],
  "last_activity": "ISO8601"
}
```

| Status | Meaning |
|--------|---------|
| `running` | Truths still firing, facts still arriving |
| `converged` | Stable state reached, no more truths to fire |
| `halted` | Invariant violated, explanation available |
| `waiting` | Blocked on external input (human approval, etc.) |

### 1.9 Handling "Waiting"

When a run is `waiting`, the UI should:
1. Show what it's waiting for: "Waiting for approval from @manager"
2. Allow the current user to act if they can
3. Subscribe to updates so UI refreshes when input arrives

```swift
if run.status == .waiting {
    showWaitingState(
        reason: run.waitingFor,
        canIAct: run.waitingFor.contains(currentUser.id)
    )
}

// Meanwhile, keep streaming — approval may arrive any moment
```

---

## 2. The Core Rule

**Truths are the boundary, not the UI.**

Mobile must never implement "business logic" as implicit UI flows. It must implement:

```
Truths → Run → Trace → Outcome
```

| Wrong | Right |
|-------|-------|
| `if (amount > 1000) showApproval()` | Truth: `invoice.amount > threshold → requires_approval` |
| `localStorage.set('status', 'approved')` | Context Entry: `fact.invoice.approved` |
| `await api.createInvoice(data)` | Proposal → Policy Check → Fact → Trace |

**Rule:** If it affects business state, it must map to a Truth/Invariant and emit a trace.

---

## 3. Terms (Converge Vocabulary)

| Term | Definition |
|------|------------|
| **Truth** | A declarative statement that must hold (`.truths` file) |
| **Invariant** | A structural constraint that cannot be violated |
| **Proposal** | A suggested change (from ML, user, or system) that must be accepted |
| **Fact** | An accepted proposal that is now part of context |
| **Context** | The append-only ledger of all facts, traces, and state |
| **Context Entry** | A single entry in the context ledger |
| **Run** | A single execution toward an outcome, producing traces |
| **Trace** | An auditable record of what happened and why |
| **Pack** | Reusable business truth library (Money, Customers, Delivery, People, Trust) |
| **Blueprint** | Composition of Packs for an outcome |
| **JTBD** | Jobs-to-be-Done ("what good feels like") |

---

## 4. Append-Only Context Thinking

The mobile app behaves like it's writing to a ledger.

### 4.1 Rules
- **No silent mutation** — Every change is a Context Entry
- **Explicit events** — Nothing happens without a trace
- **Replayable sequences** — Given context, reproduce outcome
- **Idempotency as default** — Same input → same result

### 4.2 Context Entry Format

Every entry in the context ledger:

```json
{
  "entry_id": "ctx_abc123",
  "entry_type": "fact|proposal|trace|decision",
  "timestamp": "ISO8601",
  "correlation_id": "uuid (links related entries)",
  "run_id": "uuid (the run that produced this)",
  "actor": {
    "type": "user|agent|system",
    "user_id": "string",
    "device_id": "string",
    "org_id": "string",
    "roles": ["string"]
  },
  "truth_id": "string|null (which Truth this satisfies)",
  "payload": {}
}
```

### 4.3 Proposals vs Facts

| Proposal | Fact |
|----------|------|
| Suggested by ML, user, or system | Accepted and recorded in context |
| Can be rejected by invariants | Cannot be undone (only corrected) |
| Has no authority | Is the authority |
| Example: "Create invoice for $500" | Example: `fact.invoice.created{id: inv_123}` |

**Rule:** Model output never "is" truth; it proposes effects that must converge under invariants.

---

## 5. Invariants First, Autonomy Second

Converge's differentiator: the system will **halt, explain, and restart safely**.

### 5.1 Halt → Explain → Restart

When invariants conflict:
1. **Halt** — Stop the run, do not proceed
2. **Explain** — Tell the user (and trace) why, in human terms
3. **Restart** — Allow resolution and retry

```
┌─────────────────────────────────────────────────────────────┐
│                     Proposal                                 │
│              (from ML, user, or system)                      │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                  Invariant Check                             │
│            Does this violate any Truth?                      │
└──────────────────────┬──────────────────────────────────────┘
                       │
           ┌───────────┴───────────┐
           │                       │
           ▼                       ▼
     ┌─────────┐             ┌─────────┐
     │  PASS   │             │  HALT   │
     │         │             │         │
     │ Accept  │             │ Explain │
     │ as Fact │             │ to User │
     └─────────┘             └────┬────┘
                                  │
                                  ▼
                            ┌─────────┐
                            │ Restart │
                            │ (Retry) │
                            └─────────┘
```

### 5.2 Guardrails Hit Semantics

When a guardrail fires:
- Emit `guardrail.hit` with `truth_id` and human-readable `reason`
- Show user: "This action was blocked because: {reason}"
- Offer: "Request override" (break-glass) or "Cancel"
- Never silently drop or "push through"

### 5.3 Acceptance vs Structural Invariants

| Type | Behavior | Example |
|------|----------|---------|
| **Structural** | Cannot be overridden | Period cannot be reopened after close |
| **Acceptance** | Requires approval to override | Amount > $1000 requires manager |

**Rule:** When invariants conflict, stop and explain. Never "push through."

---

## 6. Determinism Mode

Mobile supports explicit **deterministic / verifiable mode**:

### 6.1 Capabilities
- Fixtures for flows (reproducible test runs)
- Stable hashing of inputs/outputs
- Local model outputs treated as proposals, not truth
- No LLM required for core business logic

### 6.2 ML as Proposal Source

```
┌─────────────────────────────────────────────────────────────┐
│                    ML Model                                  │
│            (SmartAction predictions)                         │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
                  ┌─────────┐
                  │Proposal │  ← NOT a fact
                  └────┬────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                 Convergence Layer                            │
│         Truths + Invariants + Policy                         │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
                  ┌─────────┐
                  │  Fact   │  ← Only after acceptance
                  └─────────┘
```

**Rule:** Determinism mode must produce identical outcomes for identical context.

---

## 7. Traceability is Product, Not Telemetry

For Converge, trace is **user value**, not just debugging:

### 7.1 User Questions Traces Answer
- "Why did this happen?"
- "Who approved it?"
- "What Truth was satisfied?"
- "When did the state change?"
- "Can I undo this?"

### 7.2 Trace Entry Format

```json
{
  "trace_id": "trc_xyz789",
  "run_id": "run_abc123",
  "correlation_id": "cor_def456",
  "actor": { ... },
  "action": "create_invoice",
  "truth_id": "money.invoice.creation",
  "input_hash": "sha256:...",
  "output_hash": "sha256:...",
  "decision": "accepted|rejected|halted",
  "reason": "human-readable explanation",
  "timestamp": "ISO8601"
}
```

### 7.3 Audit Envelope

Every action that affects business state includes:
- `correlation_id` — Links to related entries
- `run_id` — The run that produced this
- `truth_id` — Which Truth was satisfied
- `actor` — Who/what performed the action
- `reason` — Why this happened (human-readable)

**Rule:** Correlation IDs and audit envelopes are part of the UX contract.

---

## 8. Capability Negotiation

Because Packs/Blueprints evolve, mobile starts every session with capability negotiation.

### 8.1 Session Handshake

```json
{
  "request": "capabilities",
  "device_id": "ios:abc123",
  "app_version": "1.0.0"
}
```

Response:
```json
{
  "packs": ["money", "customers", "delivery", "people", "trust"],
  "active_truths": ["money.invoice.*", "customers.lead.*"],
  "streaming_supported": true,
  "policies": ["money.approval_threshold", "people.access_grant"],
  "determinism_mode_available": true
}
```

### 8.2 What the Runtime Tells Mobile
- What packs exist
- What truths are active
- What streaming is supported
- What policies apply
- What budgets are in effect

**Rule:** The runtime tells mobile what is currently "true enough to run."

---

## 9. Packs and Truths

### 9.1 Core Packs

| Pack | Domain | Key Invariants |
|------|--------|----------------|
| **Money** | Financial operations | Period locks, immutable terms once signed |
| **Customers** | Revenue pipeline | Stage gates, no skipping pipeline steps |
| **Delivery** | Promise fulfillment | Promise ↔ completion tracking |
| **People** | Team operations | Access grants require approval |
| **Trust** | Governance | All actions auditable, no silent operations |

### 9.2 Truth Format

```yaml
# money.invoice.creation.truths
truth invoice_creation:
  when:
    - intent: create_invoice
    - pack: money
  requires:
    - customer_id exists
    - line_items not empty
    - amount > 0
  invariants:
    - amount > approval_threshold → requires_approval
    - period.closed → reject("Period is closed")
  produces:
    - fact.invoice.created
    - trace.invoice.creation
```

### 9.3 JTBD Format

```yaml
# money.invoice.jtbd
jtbd send_invoice:
  verb: Send
  object: Invoice
  outcome: "Customer receives accurate invoice promptly"
  triggers:
    - work_completed
    - milestone_reached
  artifacts:
    - draft_invoice → sent_invoice
  guardrails:
    - must_not: send without line items
    - must_not: send to invalid email
```

---

## 10. SmartAction Predictions

SmartAction produces **proposals**, not facts.

### 10.1 Prediction Strategies (Priority Order)

| # | Strategy | Confidence | Maps to Truth |
|---|----------|------------|---------------|
| 1 | Blueprint-Driven | 0.95 | `blueprint.{id}.next_step` |
| 2 | Artifact Flow | 0.85 | `flow.{id}.artifact_ready` |
| 3 | Frequency-Based | 0.70 | `user.{id}.pattern` |
| 4 | Pack Affinity | 0.50 | `context.current_pack` |
| 5 | Onboarding | 0.30 | `user.new.exploration` |

### 10.2 Prediction Output

```json
{
  "proposal_id": "prop_abc123",
  "action": "create_invoice",
  "confidence": 0.92,
  "reason": "Blueprint 'invoice_workflow' step 3/5",
  "truth_id": "blueprint.invoice_workflow.next_step",
  "requires_consent": "implicit|explicit|break_glass",
  "trace_id": "trc_xyz789"
}
```

**Rule:** Predictions are proposals that must pass through the convergence layer.

---

## 11. Action Lifecycle

Every action follows this lifecycle. **No step may be skipped.**

```
Propose → Consent → Check Invariants → Execute → Record Fact → Emit Trace
```

### 11.1 Detailed Flow

```swift
// 1. PROPOSE - ML or user suggests action
let proposal = SmartAction.predict(context)

// 2. CONSENT - Get user approval if needed
guard await ConsentManager.request(proposal) else { return }

// 3. CHECK INVARIANTS - Verify against Truths
let decision = await PolicyEngine.check(proposal)
if decision.halted {
    // HALT → EXPLAIN → offer RESTART
    showExplanation(decision.reason, truthId: decision.truthId)
    return
}

// 4. EXECUTE - Perform the action
let result = await execute(proposal, idempotencyKey: generateKey())

// 5. RECORD FACT - Append to context
await Context.append(.fact(result, truthId: proposal.truthId))

// 6. EMIT TRACE - Audit trail
await Context.append(.trace(proposal, result, actor: currentActor))
```

---

## 12. Truths-First UX Rule

**UI state must be derivable from context + run state, not local flags.**

### 12.1 Wrong (Local State Authority)

```swift
// BAD: UI decides business state
@State var isApproved = false

Button("Approve") {
    isApproved = true  // ← Local flag, no truth
    api.approve(invoice)
}
```

### 12.2 Right (Context Authority)

```swift
// GOOD: Context is authority, UI derives
@Published var invoice: Invoice  // From context

var isApproved: Bool {
    context.facts.contains { $0.type == "invoice.approved" && $0.id == invoice.id }
}

Button("Approve") {
    await submitProposal(.approve(invoice))  // → Convergence layer
}
```

**Rule:** If you can't derive it from context, it's not real state.

---

## 13. Converge Protocol (The Moat)

**The moat is the evented protocol, not the transport.** Transport is swappable. The protocol is the contract.

### 13.1 Protocol Layers

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Converge Protocol (Semantic Contract)                    │
│                                                                              │
│  • Jobs and Contexts are streams, not responses                              │
│  • Server emits partial answers early, then converges to better ones         │
│  • Client can push: facts, approvals, cancellations, budget changes          │
│  • Every event has seq + correlation, enabling resume                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Transport (Swappable Pipes)                           │
│                                                                              │
│  Primary:    gRPC over HTTP/2 (bidi stream)                                  │
│  Web:        WebSocket or SSE for progress + REST for submission             │
│  Future:     HTTP/3 (QUIC), WebTransport — pipe swap, not rewrite            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 13.2 Conceptual Split: Events Out, Control In

| Stream | Direction | Purpose |
|--------|-----------|---------|
| **Events** | Server → Client | Facts, proposals, traces, decisions, run status |
| **Control** | Client → Server | Inject facts, approve/reject, pause/resume, budget |

Even with gRPC bidi, maintain this conceptual separation.

### 13.3 Event Schema (The Core Contract)

Every emitted event:

```json
{
  "job_id": "job_abc123",
  "stream_id": "str_xyz789",
  "seq": 1234,
  "context_version": 56,
  "correlation_id": "cor_def456",
  "trace_id": "trc_ghi789",
  "event_type": "fact|proposal|trace|decision|status",
  "timestamp": "ISO8601",
  "payload": {}
}
```

| Field | Purpose |
|-------|---------|
| `seq` | Monotonic sequence for ordering and resume |
| `context_version` | Version of context at time of event |
| `correlation_id` | Links related events across actors |
| `trace_id` | Distributed tracing |

### 13.4 Resume Semantics (Critical)

**This is what makes transport swappable.**

Client reconnects:
```json
{ "resume_from_seq": 1234 }
```

Server responds with delta:
```json
{
  "resumed_at_seq": 1234,
  "current_seq": 1250,
  "events": [ /* seq 1235-1250 */ ]
}
```

If gap too large, server sends snapshot + recent events.

### 13.5 Control Messages (Client → Server)

```json
{
  "control_type": "inject_fact|approve|reject|pause|resume|update_budget|cancel",
  "job_id": "job_abc123",
  "correlation_id": "cor_def456",
  "payload": {}
}
```

| Control | Purpose |
|---------|---------|
| `inject_fact` | Add fact to context (HITL input) |
| `approve` | Approve pending proposal |
| `reject` | Reject pending proposal with reason |
| `pause` | Pause convergence (maintain state) |
| `resume` | Resume paused convergence |
| `update_budget` | Change time/token/cost budget |
| `cancel` | Cancel job entirely |

### 13.6 Transport Capability Flags

```json
{
  "capabilities": {
    "transports": [
      { "type": "h2-grpc", "status": "supported" },
      { "type": "h3-grpc", "status": "experimental" },
      { "type": "websocket", "status": "supported" },
      { "type": "sse", "status": "supported" },
      { "type": "webtransport", "status": "planned" }
    ],
    "default_transport": "h2-grpc",
    "resume_supported": true,
    "max_resume_gap": 10000
  }
}
```

### 13.7 Transport Selection Priority

```
Mobile (iOS/Android):
  1. gRPC over HTTP/2 (primary)
  2. HTTP/3 where mature (optional upgrade)
  3. SSE + REST (fallback)

Web (Browser):
  1. WebSocket (primary for streaming)
  2. SSE + REST (fallback)
  3. WebTransport (future)

CLI:
  1. gRPC over HTTP/2 (primary)
  2. REST (for simple commands)
```

**Rule:** Transport is a capability flag, not an architectural decision. The protocol (seq, resume, events/control split) stays constant.

---

## 14. gRPC Implementation (Primary Transport)

gRPC over HTTP/2 is the primary implementation of the Converge Protocol.

### 14.1 Context Service

```protobuf
service ContextService {
  // Stream context entries (facts, proposals, traces, decisions)
  rpc Watch(WatchRequest) returns (stream ContextEntry);

  // Append a new entry (proposal, trace)
  rpc Append(AppendRequest) returns (AppendResponse);

  // Get current snapshot (for initial load or reconnect)
  rpc Snapshot(SnapshotRequest) returns (SnapshotResponse);

  // Subscribe to run status changes
  rpc WatchRun(WatchRunRequest) returns (stream RunStatus);
}

message ContextEntry {
  string entry_id = 1;
  string entry_type = 2;  // fact|proposal|trace|decision
  string correlation_id = 3;
  string run_id = 4;
  string truth_id = 5;
  string actor_id = 6;
  int64 sequence = 7;     // For ordering and resume
  int64 timestamp = 8;
  bytes payload = 9;
}

message RunStatus {
  string run_id = 1;
  string status = 2;      // running|converged|halted|waiting
  repeated string waiting_for = 3;
  int32 facts_count = 4;
  int32 pending_count = 5;
  string halt_reason = 6;
  string halt_truth_id = 7;
}

message WatchRequest {
  string correlation_id = 1;  // Watch specific correlation
  string run_id = 2;          // Or watch specific run
  int64 since_sequence = 3;   // Resume from sequence number
  repeated string entry_types = 4;  // Filter by type
}
```

### 14.2 Entry Types

| Type | Meaning | UI Action |
|------|---------|-----------|
| `fact` | Accepted truth, now in context | Update state, may trigger re-render |
| `proposal` | Suggested change, pending | Show pending indicator |
| `trace` | Audit record | Update activity log |
| `decision` | Invariant check result | Show result, may show halt explanation |

### 14.3 Connection Lifecycle

```swift
// On app start / session begin
func startSession() async {
    // 1. Capability negotiation (REST is fine here)
    let capabilities = await api.getCapabilities()

    // 2. Open primary stream
    do {
        contextStream = try await grpc.watch(
            correlationId: nil,  // Watch all
            sinceSequence: lastKnownSequence
        )
        connectionState = .streaming

        // 3. Process stream
        for try await entry in contextStream {
            await handleEntry(entry)
        }
    } catch {
        // Fall back to SSE or polling
        await fallbackToSSE()
    }
}

// Handle stream entries
func handleEntry(_ entry: ContextEntry) async {
    // Update last known sequence (for resume)
    lastKnownSequence = entry.sequence

    switch entry.entryType {
    case "fact":
        await contextStore.appendFact(entry)
        // UI will react via published state

    case "decision":
        let decision = try decode(RunStatus.self, from: entry.payload)
        await handleRunStatus(decision)

    case "proposal":
        // Someone (maybe another actor) proposed something
        await contextStore.appendProposal(entry)

    case "trace":
        await activityLog.append(entry)
    }
}
```

### 14.4 Reconnection Protocol

```swift
// On disconnect
func handleDisconnect() async {
    connectionState = .reconnecting

    var backoff = 1.0  // seconds
    let maxBackoff = 30.0

    while connectionState == .reconnecting {
        do {
            // Resume from last known sequence
            contextStream = try await grpc.watch(
                sinceSequence: lastKnownSequence
            )
            connectionState = .streaming

            for try await entry in contextStream {
                await handleEntry(entry)
            }
        } catch {
            // Exponential backoff
            try? await Task.sleep(nanoseconds: UInt64(backoff * 1_000_000_000))
            backoff = min(backoff * 2, maxBackoff)
        }
    }
}
```

### 14.5 REST Fallback (Degraded Mode)

When streaming is unavailable:

```swift
func fallbackToSSE() async {
    // Try SSE first
    if capabilities.sseSupported {
        connectionState = .degraded
        await startSSEConnection()
        return
    }

    // Last resort: polling
    connectionState = .degraded
    await startPolling(interval: 5.0)  // 5 second poll
}

// UI must indicate degraded mode
var connectionIndicator: some View {
    switch connectionState {
    case .streaming:
        Circle().fill(.green)
    case .reconnecting:
        Circle().fill(.yellow)
    case .degraded:
        HStack {
            Circle().fill(.orange)
            Text("Limited mode")
        }
    case .offline:
        HStack {
            Circle().fill(.red)
            Text("Offline")
        }
    }
}
```

### 14.6 Sequence Numbers and Resume

Every entry has a `sequence` number:
- Monotonically increasing per context
- Used to resume after disconnect
- Used to detect gaps (request snapshot if gap too large)

```swift
func handleEntry(_ entry: ContextEntry) async {
    let expectedSequence = lastKnownSequence + 1

    if entry.sequence > expectedSequence + 100 {
        // Large gap, request snapshot
        await requestSnapshot()
    } else if entry.sequence > expectedSequence {
        // Small gap, request missing entries
        await requestRange(from: expectedSequence, to: entry.sequence)
    }

    lastKnownSequence = entry.sequence
}
```

---

## 15. REST API Surface (Fallback/Degraded)

REST is always available as fallback. These endpoints mirror the protocol semantics.

### 15.1 Job Endpoints

```
POST   /jobs                    → Create job, returns job_id
GET    /jobs/{job_id}           → Get job status + latest facts
GET    /jobs/{job_id}/events    → Poll events (include ?since_seq=N)
POST   /jobs/{job_id}/control   → Send control message
DELETE /jobs/{job_id}           → Cancel job
```

### 15.2 Polling Pattern (Degraded Mode)

```swift
// When streaming unavailable
func pollLoop() async {
    while jobStatus != .converged && jobStatus != .halted {
        let events = await api.get("/jobs/\(jobId)/events?since_seq=\(lastSeq)")
        for event in events {
            await handleEvent(event)
            lastSeq = event.seq
        }
        try? await Task.sleep(nanoseconds: 5_000_000_000)  // 5 sec
    }
}
```

**Rule:** Polling is degraded mode. UI must indicate. Resume semantics still apply.

---

## 16. Offline Behavior

### 16.1 Idempotency Keys

Format: `{device_id}:{action}:{timestamp_ms}:{random_4}`

Example: `ios:abc:create_invoice:1704067200000:f7a2`

### 16.2 Offline Queue

```json
{
  "idempotency_key": "ios:abc:create_invoice:1704067200000:f7a2",
  "proposal": { ... },
  "truth_id": "money.invoice.creation",
  "queued_at": "ISO8601",
  "replay_safe": true
}
```

### 16.3 Replay Semantics

- Replay on network restore
- Server dedupes by idempotency_key
- If Truth changed while offline → HALT and explain

**Rule:** Every action is an event; every event is attributable; every event can be replayed.

---

## 17. Eval Cases

### 17.1 Core Evals (All Platforms Must Pass)

| ID | Name | Validates |
|----|------|-----------|
| `truth.fact_from_proposal` | Proposal becomes Fact only after invariant check | Convergence layer |
| `truth.halt_on_violation` | System halts when invariant violated | Halt → Explain → Restart |
| `truth.trace_has_truth_id` | Every trace references the Truth it satisfies | Traceability |
| `context.actor_present` | Every entry has actor | Audit |
| `context.correlation_linked` | Related entries share correlation_id | Traceability |
| `context.idempotent_replay` | Same input → same output | Determinism |
| `determinism.no_llm_mode` | Core logic works without ML | Determinism mode |
| `capability.session_handshake` | Session starts with capability negotiation | Capability negotiation |
| `stream.early_facts` | UI renders facts before run completes | Live convergence |
| `stream.multi_actor` | UI updates when other actor injects fact | Multi-actor |
| `stream.resume_sequence` | Reconnect resumes from last sequence | Stream resume |
| `stream.degraded_indicator` | UI shows degraded mode when REST fallback | Connection state |
| `stream.waiting_state` | UI shows "waiting for X" when run blocked | Run lifecycle |

### 17.2 Eval Fixture Format

```json
{
  "id": "truth.halt_on_violation",
  "context": {
    "facts": [{ "type": "period.closed", "period_id": "2024-Q1" }]
  },
  "proposal": {
    "action": "create_invoice",
    "period_id": "2024-Q1"
  },
  "expected": {
    "decision": "halted",
    "truth_id": "money.period.closed_invariant",
    "reason_contains": "Period is closed"
  }
}
```

---

## 18. Implementation Checklist

### Axioms
- [ ] Monotonicity: Context only grows (append, never delete)
- [ ] Idempotency: Same context → same outcome
- [ ] Determinism: Can run without LLM
- [ ] Convergence: Runs halt at stable state
- [ ] No Hidden State: All state in context
- [ ] Governance Separation: Trust pack separate from outcome packs
- [ ] Live Convergence: Stream-first, not request/response

### Stream-First Architecture
- [ ] gRPC bidirectional streaming as primary transport
- [ ] Sequence numbers tracked for resume
- [ ] Reconnection with exponential backoff
- [ ] SSE fallback when gRPC blocked
- [ ] REST fallback (degraded mode) as last resort
- [ ] Connection state indicator in UI (streaming/reconnecting/degraded/offline)

### Live Convergence
- [ ] UI handles early/partial facts
- [ ] UI updates when facts arrive from other actors
- [ ] Run status tracked (running/converged/halted/waiting)
- [ ] "Waiting for X" shown when run blocked
- [ ] Progressive refinement (early answers may be revised)

### Truths Integration
- [ ] Every business action maps to a Truth
- [ ] Proposals pass through convergence layer
- [ ] Facts only created after invariant check
- [ ] Traces reference truth_id

### Context Behavior
- [ ] All entries have actor
- [ ] All entries have correlation_id
- [ ] UI state derived from context
- [ ] No local state as authority

### Halt → Explain → Restart
- [ ] Invariant violations halt the run
- [ ] User sees human-readable explanation
- [ ] Restart path available
- [ ] Break-glass requires reason in trace

### Capability Negotiation
- [ ] Session starts with handshake
- [ ] Packs/Truths/Policies fetched from runtime
- [ ] Determinism mode available
- [ ] Streaming capability detected

---

## 19. One-Liner

> **A Converge mobile app is a Truths-driven, ledger-minded, deterministic-when-needed client that treats ML as proposals, not truth.**

---

*This contract applies to iOS, Android, and CLI. The implementation is native; the behavior is identical.*
