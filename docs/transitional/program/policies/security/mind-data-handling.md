# Data Handling — YAI Mind

## Default stance

- Treat prompts and context as sensitive.
- Do not log raw prompts by default.
- Do not commit memory databases or real-user derived artifacts.

## Allowed in-repo data

- synthetic fixtures for tests
- small demo data with clear provenance and licensing
- model/schema/manifests describing external datasets

## Forbidden in-repo data

- provider keys/tokens
- raw conversation dumps
- private endpoints
- memory DBs derived from real usage

## Redaction

If logs or artifacts must be shared for debugging:
- remove secrets, tokens, and identifiers
- remove full prompt payloads
- keep only minimal evidence needed
