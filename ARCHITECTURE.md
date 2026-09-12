# meta-signal-aggregator — architecture

`meta-signal-aggregator` is the owner/meta contract for the `aggregator`
component. It defines how the daemon is configured and how configuration is
observed or validated, including the daemon-local durable fragile output index
policy consumed by the ordinary aggregator runtime.

## Role

The contract exposes three operations:

- `Configure(ConfigurationChange)` submits the complete active configuration.
- `ObserveConfiguration(ObserveConfiguration)` returns the current typed
  configuration view.
- `ValidateConfiguration(ConfigurationCandidate)` checks a candidate without
  committing it.

Replies distinguish configured, observed, validated, and rejected outcomes.
The output-interface slice names the durable index as daemon-local state under
the configured store path while preserving that produced references are opaque,
fragile, and stale-capable. `ClaudeSubagentOutput` configures `.output` JSONL
roots explicitly; task notification paths are link evidence, not authority to
read arbitrary files. Optional legacy report and agent-output roots are
tagged as read-only recovery inputs, not primary design surfaces.

## Boundary

This crate owns only configuration wire vocabulary and tests. It does not read
Claude, Codex, Pi, repository, legacy report, or legacy agent-output sources; it
does not implement collection; it does not store daemon state. The runtime
daemon imports these types and applies configuration through its meta
Signal/Nexus/SEMA path.

## Code map

```text
ethos/signal.ethos          the schema authority: queries, responses, and types
build.rs                    regenerates from the ethos and refuses drift
src/generated/signal.rs     the checked-in Rust projection
src/lib.rs                  the module surface and the runtime default ceilings
tests/generated_contract.rs rkyv frame and Datom text round-trip witnesses
```

The ceilings the old contract carried as `Default` impls are now the
`DefaultingPolicy` kind in `src/lib.rs`: the generated projection derives no
`Default`, and a ceiling is a policy a runtime asks for, not a zero value.
