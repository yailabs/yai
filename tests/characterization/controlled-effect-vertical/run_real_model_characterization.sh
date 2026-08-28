#!/usr/bin/env bash
set -euo pipefail

# Optional evidence run against an actual local/remote OpenAI-compatible model.
# It exercises the product HTTP and controlled-effect boundaries; exact model
# wording is never an assertion. The Case/provider/resource must already be
# admitted so this script cannot invent authority as test setup.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"

: "${YAI_HOME:?set YAI_HOME}"
: "${YAI_JOURNAL:?set YAI_JOURNAL to the admitted Case journal}"
: "${YAI_CASE_REF:?set YAI_CASE_REF}"
: "${YAI_PROVIDER_SUBJECT_REF:?set YAI_PROVIDER_SUBJECT_REF}"
: "${YAI_PROVIDER_BASE_URL:?set YAI_PROVIDER_BASE_URL}"
: "${YAI_PROVIDER_MODEL:?set YAI_PROVIDER_MODEL}"
: "${YAI_RESOURCE_ATTACHMENT_ID:?set YAI_RESOURCE_ATTACHMENT_ID}"

prompt=${YAI_REAL_EFFECT_PROMPT:-"Propose writing a short hello message to the admitted controlled filesystem resource."}

"$YAI_BIN" effect filesystem-write \
  --case "$YAI_CASE_REF" \
  --subject "$YAI_PROVIDER_SUBJECT_REF" \
  --attachment "$YAI_RESOURCE_ATTACHMENT_ID" \
  --prompt "$prompt" \
  --base-url "$YAI_PROVIDER_BASE_URL" \
  --model "$YAI_PROVIDER_MODEL"
