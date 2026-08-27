# Tests and executable evidence

Tests are classified by what they prove, not by the nouns they exercise:

- `characterization/` freezes product-reachable behavior before ownership or
  layout changes. These tests may expose behavior that the Constitution later
  requires the implementation to replace.
- `smoke/` contains bounded product, fixture, and C component checks inherited
  from the current executable baseline. Their authority class is recorded in
  `classification.tsv`.
- `cases/` contains operator procedures linked from the canonical validation
  guide.
- `fixtures/` contains test-owned peers and data. A fixture effect is never
  evidence that a product path performed that effect.

Rust unit tests live beside their implementation. Empty unit, integration, and
adversarial placeholder roots are intentionally absent; a test category does
not justify a directory until it has an executable consumer.
