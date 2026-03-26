# opcrew

Give it a problem. It builds a team. They fix it.

opcrew is a Rust CLI that assembles AI agent squads to diagnose and fix infrastructure problems in seconds. It pre-fetches system context, triages with a single LLM call, and — if confident — applies the fix directly. For complex issues, it deploys a full team of specialist agents that execute real commands through a multi-layer safety system.

```
$ opcrew --problem "container test-web returns 502"

⚡ Collecting system context... 9 data points in 214ms
⚡ Triage: confidence 90% — nginx upstream unreachable
→ docker exec test-web nginx -t
→ docker restart test-web
✓ Verify: HTTP 200 OK

Duration: 15s | Guardian: 3 approved, 0 blocked
```

## Performance

| Problem | Time |
|---------|------|
| Container crash loop | **7 seconds** |
| Docker status check | **13 seconds** |
| nginx 502 | **15 seconds** |
| K8s pod failures (remote SSH) | **45-90 seconds** |
| Complex multi-service issue | **3-5 minutes** |

## How it works

```
Problem
  │
  ├─ Pre-fetch: collect system data in parallel (1-3s, no LLM)
  │   docker ps, logs, df, free, dmesg — selected by problem keywords + infra graph
  │
  ├─ Triage: single LLM call with all context (3-5s)
  │   confidence >= 80% → apply fix directly → verify → done (10-15s total)
  │   confidence < 80%  ↓
  │
  ├─ Hypothesis Agent: ranked root causes with Bayesian priors from memory
  ├─ Smart Router: scores problem on 5 signals → FastPath or FullPipeline
  │
  ├─ CEO Agent: creates plan with diagnose + fix + verify tasks
  ├─ Agent Factory: builds specialists with service catalog from infra graph
  ├─ Squad Runner: executes concurrently, Guardian reviews every command
  │   → ServiceTool: agents say "restart nginx", code translates to
  │     docker restart / kubectl rollout restart / systemctl restart
  ├─ Verifier: runs real commands to confirm the fix worked
  │   → Not resolved? CEO replans, new squad, up to 3 rounds
  │   → Still not resolved? Escalation report with recommended manual steps
  │
  └─ Memory: saves what worked, what failed, Bayesian hypothesis updates
```

## Quick start

```bash
# Build and install
cargo build --release
cargo install --path .

# Set your API key
export DEEPSEEK_API_KEY=sk-...    # or ANTHROPIC_API_KEY, OPENAI_API_KEY, etc.

# Fix a problem
opcrew --provider deepseek --problem "container my-app is crashing"

# Preview without executing
opcrew --problem "disk full on /var" --dry-run

# Discover your infrastructure first (recommended)
opcrew --provider deepseek infra discover

# See all examples
opcrew examples
```

## Multi-provider LLM

| Provider | Flag | API Key | Default Model |
|----------|------|---------|---------------|
| Claude | `--provider claude` | `ANTHROPIC_API_KEY` | claude-sonnet-4-20250514 |
| OpenAI | `--provider openai` | `OPENAI_API_KEY` | gpt-4o |
| DeepSeek | `--provider deepseek` | `DEEPSEEK_API_KEY` | deepseek-chat |
| Gemini | `--provider gemini` | `GEMINI_API_KEY` | gemini-2.5-flash |
| Local | `--provider local` | None | llama3 |

```bash
opcrew --provider deepseek --problem "nginx 502"
opcrew --provider openai --model gpt-4o-mini --problem "disk full"
opcrew --provider local --problem "check services"    # Ollama, no API key
```

## ServiceTool — intent-based execution

Agents don't guess shell commands. They express **intentions**:

```json
{"tool": "service", "action": "logs", "args": {"service": "test-web"}}
```

The ServiceTool translates to the correct command for the service's runtime:

| Runtime | `service logs test-web` becomes |
|---------|-------------------------------|
| Docker | `docker logs test-web --tail 50` |
| Kubernetes | `kubectl logs -n prod test-web --tail 50` |
| systemd | `journalctl -u test-web -n 50` |
| Podman | `podman logs test-web --tail 50` |
| Any other | LLM translates on the fly |

Actions: `logs`, `status`, `config`, `restart`, `stop`, `start`, `edit_config`, `exec`, `env`, `scale`

No hardcoded runtimes. The LLM translates for any runtime — Docker, K8s, Podman, LXC, Nomad, Fly.io — without code changes.

## Infrastructure discovery

Adaptive discovery: fingerprints your server, then generates tailored commands.

```bash
opcrew infra discover                          # scan localhost
opcrew infra discover --sudo                   # retry denied commands with sudo
opcrew infra discover --host admin@prod-01     # scan remote host
opcrew infra show                              # display the graph
opcrew infra show --json                       # structured output
```

```
Phase 1: Fingerprint — detects OS, Docker, K8s, Podman, nginx, haproxy (no LLM)
Phase 2: LLM generates up to 30 read-only commands tailored to your stack
Phase 3: Execute with classification (success / permission-denied / not-found)
Phase 4: LLM extracts service graph + execution contexts
```

Each service gets an `ExecutionContext` (runtime + identifier) so the ServiceTool knows how to interact with it. Discovery fills this automatically.

## Smart routing

Problems are scored on 5 objective signals:

| Signal | Weight | Fast if |
|--------|--------|---------|
| LLM complexity assessment | 25% | Simple |
| Top hypothesis confidence | 25% | H1 > 60% |
| Number of services involved | 15% | 1 service |
| Confirm command simplicity | 15% | No pipes |
| Memory: past success rate | 20% | > 60% success |

**Score >= 60 → FastPath** (1 agent, ~15s)
**Score < 60 → FullPipeline** (CEO → squad → verifier, ~3-5min)

**Memory replay**: if the same problem was solved before with >70% success rate, opcrew skips everything and replays the known fix.

## Watch mode

Continuous monitoring with optional auto-remediation:

```bash
opcrew --watch --watch-config checks.toml
opcrew --watch --auto-fix --max-rounds 1      # cautious auto-fix
```

```toml
interval_secs = 30
auto_fix = false

[[checks]]
type = "ServiceDown"
service_name = "nginx"
check_cmd = "systemctl is-active nginx"

[[checks]]
type = "DiskUsage"
path = "/var"
threshold_pct = 85

[[checks]]
type = "PortUnreachable"
host = "localhost"
port = 5432
```

When a critical issue is detected with `--auto-fix`, the full pipeline is triggered automatically.

## Safety

Every command passes through a multi-layer Guardian:

1. **Shell composition block** — `;`, `|`, `&&`, `$()` rejected. Commands must be atomic.
2. **bash -c block** — `docker exec X bash -c '...'` blocked (bypasses Guardian).
3. **Static allowlist** — read-only operations auto-approved (ls, cat, ps, df, grep).
4. **AI review** — LLM classifies as SAFE / RISKY / BLOCKED.
5. **User approval** — risky commands prompt: `[y]es / [n]o / [a]ll-similar / [b]lock-all`.

Additional layers:
- **File path denylist** — writes to `/etc/`, `/boot/`, `/sys/`, `/proc/` blocked at tool level
- **Secret masking** — API keys, passwords, tokens redacted everywhere (conversation, audit, output)
- **Token budget** — hard cap on API costs per session and per agent
- **Circuit breaker** — if LLM API is down, Guardian fails closed (blocks all execution)
- **Prompt limit** — max approval prompts per session, then fail-closed
- **Session timeout** — hard wall-clock limit

## Persistent memory

Stored in `~/.opcrew/memory.db` (SQLite):

- **Past solutions** — what worked and what failed, with failure reasons
- **Approach statistics** — success rate per approach per problem type
- **Hypothesis priors** — Bayesian-updated probabilities from real outcomes
- **Infrastructure graph** — services, dependencies, execution contexts

The system learns from every session. Failed approaches are flagged. Successful approaches get higher priority next time. Use `--no-memory` to disable.

## CLI reference

| Flag | Default | Description |
|------|---------|-------------|
| `--problem`, `-p` | | Problem to solve |
| `--file`, `-f` | | Read problem from a file |
| `--provider` | `claude` | LLM provider: claude, openai, deepseek, gemini, local |
| `--model`, `-m` | per provider | Override default model |
| `--dry-run` | off | Preview without executing |
| `--auto-approve` | off | Skip approval prompts |
| `--target` | localhost | Remote host via SSH (user@host) |
| `--max-agents` | `5` | Max specialist agents |
| `--max-rounds` | `3` | Max verification rounds |
| `--session-timeout` | `1800` | Hard time limit (seconds) |
| `--session-budget` | `2000000` | Token budget |
| `--no-memory` | off | Disable persistent memory |
| `--watch` | off | Continuous monitoring |
| `--watch-interval` | `60` | Check interval (seconds) |
| `--auto-fix` | off | Auto-fix in watch mode |
| `--watch-config` | | TOML config for checks |
| `--verbose`, `-v` | off | Debug logging |
| `--json` | off | JSON output |

## Architecture

```
src/
  main.rs                  Pipeline orchestration (turbo + full)
  cli.rs                   CLI with detailed --help
  config.rs                Environment-based configuration
  error.rs                 Error types (thiserror)

  api/
    provider.rs            LlmProvider trait (multi-provider)
    client.rs              Claude (streaming, rate limiting, retries)
    openai.rs              OpenAI + DeepSeek (OpenAI-compatible)
    gemini.rs              Google Gemini
    local.rs               Ollama / llama.cpp
    types.rs               Shared types
    schema.rs              JSON Schema validation with retry

  agents/
    ceo.rs                 CEO (chain-of-thought planning, synthesis)
    specialist.rs          Specialist (autonomous tool-use loop)
    factory.rs             Agent factory (service catalog injection)
    verifier.rs            Verifier (executes real checks)
    hypothesis.rs          Hypothesis (Bayesian priors)

  execution/
    prefetch.rs            Parallel system context collection
    triage.rs              Single-shot LLM diagnosis
    routing.rs             Smart routing (5-signal scoring)
    runner.rs              Squad runner (topological sort, JoinSet)
    budget.rs              Token budget (atomic, no TOCTOU)
    circuit_breaker.rs     Circuit breaker (fail-closed)

  tools/
    service.rs             ServiceTool (LLM-powered intent translation)
    shell.rs               Shell (atomic commands, no composition)
    file_ops.rs            File operations (path denylist)
    log_reader.rs          Log reader/searcher
    code_writer.rs         Code editor
    registry.rs            Tool registry
    target.rs              Local / remote (SSH)

  safety/
    guardian.rs            Multi-layer command review
    allowlist.rs           Read-only command allowlist
    approval.rs            User approval (rate-limited)
    audit.rs               Audit log (HMAC, rotation, secrets masked)
    secrets.rs             Secret detection and masking

  infra/
    graph.rs               Service graph + ExecutionContext
    discovery.rs           Adaptive discovery (fingerprint → commands)
    commands.rs            Infra CLI handlers

  watch/
    monitor.rs             Health check types
    trigger.rs             Watch loop + auto-fix trigger

  memory/
    store.rs               SQLite (sessions, solutions, hypotheses, infra)
    models.rs              Data models

  output/
    formatter.rs           Colored CLI output
  observability/
    logging.rs             Structured JSON logging
    metrics.rs             Runtime metrics (Guardian stats, tokens)
    export.rs              Audit log export
```

## Requirements

- Rust 1.75+
- Linux
- An API key for any supported provider, or a local Ollama instance

## License

Apache License 2.0 — see [LICENSE](LICENSE).
