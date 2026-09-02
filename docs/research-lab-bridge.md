# Research Lab Bridge

## Purpose

This document defines the conservative bridge between external Obsidian research material and YAI repository canon. It exists so research can be promoted into this repository without losing provenance, evidence level, claim safety, non-goals, or implementation boundary.

The bridge supports future promotion into docs, contracts, schemas, fixtures, guards, tests, source code, and wave or spine records. This bridge wave itself only creates documentation and does not change runtime behavior.

## Non-canonical research source

The YAI Research Lab is a non-canonical research and planning surface. YAI repo canon is defined by this repository's promoted docs, contracts, schemas, fixtures, guards, tests, source code, and wave/spine records.

Research notes, mathematical sketches, market reasoning, company strategy, and whiteboard material do not become YAI implementation claims until promoted into this repository with explicit evidence level, claim safety, and non-goals.

## Promotion boundary

Research material may enter this repository only as a bounded promotion. A promotion must identify the source vault note, the claim or invariant being promoted, the evidence level, the implementation gap, the claim-safety boundary, the non-goals, and the target repo file.

Research-only or implementation-missing material must remain labeled as such. Promotion must not convert a hypothesis, proof sketch, market claim, or company strategy into implementation language.

## Allowed promotion targets

Allowed promotion targets include:

```text
docs/
work/spines/
work/waves/
include/yai/
proto/schemas/
proto/fixtures/
tools/checks/
tests/
source implementation files
```

The target determines the required review and validation. Documentation promotion does not imply implementation. Contract, schema, fixture, guard, test, and source promotions require their own scoped repo changes and validation.

## Required promotion metadata

A promotion should record:

```text
source vault note
claim or invariant
evidence level
repo evidence references
implementation gap
claim safety
non-goals
target repo file
rollback/removal condition
```

The promotion should also update the vault-side promotion log and repo link ledger so provenance remains traceable after repo insertion.

## Research index

The repo-side research navigation surface is `docs/research-index.md`.

## What this document does not claim

This document does not claim that any research note is repo canon.

This document does not implement runtime behavior.

This document does not change C or Rust source code, schemas, headers, fixtures, tests, Makefile targets, or guards.

This document does not claim that mathematical sketches are implemented behavior.

This document does not claim production readiness, provider execution, Context Compiler implementation, generic Pack runtime, enterprise control plane, PMI workspace, Studio, SDK, access, entitlement, or machine authorization implementation.

## Current bridge status

The bridge is active as a documentation and promotion-governance surface. The first active use is conservative: record the boundary, point future agents to the promotion method, and prevent Obsidian research material from being treated as YAI repo canon until explicitly promoted.
