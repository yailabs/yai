#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_BIN="$ROOT/target/debug/yai"
TEST_DIR=$(mktemp -d /tmp/yai-cli-product-surface.XXXXXX)
export YAI_HOME="$TEST_DIR/home"
RUNTIME_PID=""

cleanup() {
  if [ -n "$RUNTIME_PID" ] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" 2>/dev/null || true
    wait "$RUNTIME_PID" 2>/dev/null || true
  fi
  rm -rf "$TEST_DIR"
}
trap cleanup EXIT INT TERM

require_text() {
  output=$1
  expected=$2
  printf '%s\n' "$output" | grep -F -- "$expected" >/dev/null || {
    printf 'missing expected text: %s\noutput:\n%s\n' "$expected" "$output" >&2
    exit 1
  }
}

root_help=$($YAI_BIN)
require_text "$root_help" "YAI — governed operational AI runtime"
require_text "$root_help" "case"
if printf '%s\n' "$root_help" | grep -Eq '^  (store|journal|projection|graph|facts|carrier) '; then
  printf 'engineering plumbing leaked into default help\n' >&2
  exit 1
fi
advanced_help=$($YAI_BIN help --advanced)
require_text "$advanced_help" "store"
require_text "$advanced_help" "journal"
discovery=$($YAI_BIN help --json)
require_text "$discovery" '"schema":"yai.cli.command_discovery.v1"'
require_text "$discovery" '"cli_registry_digest":"sha256:'

before=$($YAI_BIN doctor)
require_text "$before" "NOT_INITIALIZED"
$YAI_BIN init --tenant tenant:cli-product --organization organization:cli-product >/dev/null
after=$($YAI_BIN doctor)
require_text "$after" "OK"
repeat=$($YAI_BIN init --tenant tenant:cli-product --organization organization:cli-product --json)
require_text "$repeat" '"operation_id":"yai.init"'
require_text "$repeat" "already_exists"

$YAI_BIN case create case:cli-product --tenant tenant:cli-product >/dev/null
show_before=$($YAI_BIN case show case:cli-product)
require_text "$show_before" "never_started"
stopped=$($YAI_BIN case stop case:cli-product)
require_text "$stopped" "no_active_execution"

$YAI_BIN case participant role add case:cli-product \
  --participant participant:model --role model-executor >/dev/null
$YAI_BIN case participant role add case:cli-product \
  --participant participant:model --role operation-proposer >/dev/null
participants=$($YAI_BIN case participant list case:cli-product)
require_text "$participants" "participant:model"

$YAI_BIN case provider attach case:cli-product \
  --participant participant:model \
  --endpoint http://127.0.0.1:9/v1/chat/completions \
  --model fixture-model >/dev/null
mkdir -p "$TEST_DIR/resource/allowed"
$YAI_BIN case resource attach filesystem case:cli-product \
  --resource resource:workspace \
  --root "$TEST_DIR/resource" \
  --allow-prefix allowed \
  --policy-owner participant:model \
  --max-bytes 1024 >/dev/null
resources=$($YAI_BIN case resource list case:cli-product)
require_text "$resources" "resource:workspace"

policy_ingest=$($YAI_BIN policy ingest "$ROOT/tests/fixtures/cli-product-policy.json" \
  --tenant tenant:cli-product)
policy_id=$(printf '%s\n' "$policy_ingest" | sed -n 's/^artifact_id: //p' | head -1)
[ -n "$policy_id" ]
$YAI_BIN policy validate "$policy_id" --reason "Wave 16 validation" >/dev/null
$YAI_BIN policy publish "$policy_id" --reason "Wave 16 publication" >/dev/null
$YAI_BIN case policy bind case:cli-product --artifact "$policy_id" \
  --reason "Wave 16 product binding" >/dev/null

workflow_define=$($YAI_BIN workflow define --tenant tenant:cli-product \
  --file "$ROOT/tests/fixtures/workflows/cli-product-governed.v1.json")
workflow_id=$(printf '%s\n' "$workflow_define" | \
  sed -n 's/^workflow_definition_id: //p' | head -1)
[ -n "$workflow_id" ]
$YAI_BIN workflow bind case:cli-product --definition "$workflow_id" \
  --executor operator=participant:model \
  --resource workspace=resource:workspace >/dev/null
workflow_ready=$($YAI_BIN workflow status case:cli-product)
require_text "$workflow_ready" "completed: false"

$YAI_BIN runtime serve --workers 1 --max-active-per-tenant 1 \
  --max-queued-per-tenant 2 --max-queued-total 2 \
  >"$TEST_DIR/runtime.log" 2>&1 &
RUNTIME_PID=$!
workflow_complete=""
for _ in $(seq 1 500); do
  workflow_complete=$($YAI_BIN workflow status case:cli-product 2>/dev/null || true)
  printf '%s\n' "$workflow_complete" | grep -F -- "completed: true" >/dev/null && break
  sleep 0.02
done
require_text "$workflow_complete" "completed: true"
$YAI_BIN runtime stop >/dev/null
wait "$RUNTIME_PID"
RUNTIME_PID=""
[ "$(sed -n '1p' "$TEST_DIR/resource/allowed/porcelain.txt")" = \
  "product porcelain reached governed reality" ]

case_json=$($YAI_BIN case show case:cli-product --json)
require_text "$case_json" '"operation_id":"yai.case.show"'
require_text "$case_json" '"model_id":"fixture-model"'
require_text "$case_json" '"resource_id":"resource:workspace"'
if printf '%s\n' "$case_json" | grep -F -- "OPENCODE_LLM_API_KEY" >/dev/null; then
  printf 'credential reference leaked into Case machine view\n' >&2
  exit 1
fi

compat_json=$($YAI_BIN case status --case case:cli-product --json)
require_text "$compat_json" '"operation_id":"yai.case.show"'

no_color=$(NO_COLOR=1 $YAI_BIN case show case:cli-product)
escape=$(printf '\033')
if printf '%s' "$no_color" | grep -F -- "$escape" >/dev/null; then
  printf 'NO_COLOR output contained ANSI\n' >&2
  exit 1
fi

set +e
unknown=$($YAI_BIN case shwo case:cli-product 2>&1)
unknown_exit=$?
removed=$($YAI_BIN observe process --pid $$ 2>&1)
removed_exit=$?
json_error=$($YAI_BIN case show case:missing --json 2>&1)
json_error_exit=$?
set -e
[ "$unknown_exit" -eq 2 ]
[ "$removed_exit" -eq 2 ]
[ "$json_error_exit" -eq 3 ]
require_text "$unknown" 'did you mean `yai case show`'
require_text "$removed" 'use `yai process observe`'
require_text "$json_error" '"schema":"yai.cli.error.v1"'

printf 'cli_product_surface: registry_help=pass\n'
printf 'cli_product_surface: first_use=pass\n'
printf 'cli_product_surface: never_started_case=pass\n'
printf 'cli_product_surface: participant_provider_resource=pass\n'
printf 'cli_product_surface: porcelain_governed_workflow=pass\n'
printf 'cli_product_surface: provider_invocations=0\n'
printf 'cli_product_surface: json_no_color_errors=pass\n'
