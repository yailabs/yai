# YAILabs CLI family contract

YAILabs CLIs share engineering discipline, not domain vocabulary. YVEX exposes
model execution infrastructure; YAI exposes governed operational work.

- One compiled command metadata authority defines syntax and discovery.
- A stable operation identity is independent from its projected command path.
- One resolved descriptor selects exactly one invocation lane.
- Visibility is independent from execution lane.
- Product porcelain is distinct from engineering plumbing.
- Parser, help, machine discovery and completion cannot maintain competing
  syntax catalogs.
- Syntax admission is centralized; domain owners retain semantic validation.
- Human and machine output project the same typed operation result.
- Machine JSON is deterministic, untruncated, ANSI-free and contains no secret
  configuration values.
- Human success uses stdout; diagnostics, warnings and hints use stderr.
- Semantic color is restrained, never carries meaning alone, honors
  `NO_COLOR`, and is disabled for redirected output.
- Aliases and removed paths are explicit, bounded and identity-preserving.
- No command silently enters a TUI, alternate screen or mutable session
  context.
- Environment variables configure a process; they do not become ordinary
  object-selection workflow state.

This contract does not require identical command trees, registry schemas,
renderers, JSON envelopes or binary protocols across products.
