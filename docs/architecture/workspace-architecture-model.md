# Workspace Architecture Model (6/8)

## Purpose

This document defines `workspace` as a first-class execution cell in YAI.

A workspace is not only a convenience for CLI commands. It is the formal boundary where:

- session binding is resolved
- runtime context is attached
- normative context is carried
- resolution/evidence summaries are stored and inspected

## Canonical Boundary Layers

1. Execution boundary: a workspace is the runtime entry context for command resolution.
2. Context boundary: declared/inferred/effective normative context is workspace-scoped.
3. Policy boundary: policy summaries and effect snapshots are attached to a workspace.
4. Inspect/debug boundary: trace refs and resolution summaries are queryable per workspace.
5. Shell-binding boundary: prompt token represents active binding, not shell path.

## Non-goals (this tranche)

This tranche does not claim:

- full OS sandboxing
- full memory isolation
- full anti-cross-workspace enforcement

Those are addressed in 7/8 hardening.

## Required Distinctions

YAI now treats the following as separate concepts:

- Active workspace context: selected via session binding.
- Workspace root: physical path anchored for that workspace.
- Repo/shell path: current directory in terminal.

They can coincide, but are not equivalent.

