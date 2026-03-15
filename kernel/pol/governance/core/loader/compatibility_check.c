#include "../internal.h"

#include <stdio.h>

static int require_surface(const yai_governance_runtime_t *rt, const char *rel, char *json, size_t cap) {
  return yai_governance_read_governance_surface_file(rt, rel, json, cap);
}

static int require_repo_file(const yai_governance_runtime_t *rt, const char *rel, char *json, size_t cap) {
  char path[512];
  if (yai_governance_safe_snprintf(path, sizeof(path), "%s/%s", rt->root, rel) != 0) return -1;
  return yai_governance_read_text_file(path, json, cap);
}

int yai_governance_compatibility_check(yai_governance_runtime_t *rt, char *err, size_t err_cap) {
  char json[4096];

  if (!rt) return -1;

  if (require_surface(rt, "model/manifests/indexes/compatibility-matrix.v1.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "model/manifests/runtime/governance-manifest.v1.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "model/manifests/runtime/runtime-entrypoints.v1.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "model/manifests/publish/index.v1.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "model/manifests/publish/layers.v1.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical manifest spine");
    return -1;
  }

  (void)yai_governance_json_extract_string(json, "schema_set", rt->compatibility.profile, sizeof(rt->compatibility.profile));
  (void)yai_governance_json_extract_string(json, "governance_version", rt->compatibility.governance_version, sizeof(rt->compatibility.governance_version));
  if (rt->compatibility.profile[0] == '\0') {
    (void)yai_governance_safe_snprintf(rt->compatibility.profile, sizeof(rt->compatibility.profile), "%s", "runtime-consumer.v4");
  }

  if (require_surface(rt, "model/schema/policy/policy.v1.schema.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "model/schema/policy/authority/decision-record.v1.schema.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "model/schema/governance/review-state.v1.schema.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "model/schema/evidence/evidence-index.v1.schema.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "model/schema/governance/workspace-attachment.v1.schema.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "model/schema/runtime/retention-policy.v1.schema.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "model/schema/runtime/containment-metrics.v1.schema.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "model/schema/evidence/verification-report.v1.schema.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical schema surfaces");
    return -1;
  }

  if (require_surface(rt, "control/ingestion/templates/governance-source.template.v1.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "control/ingestion/pipeline/sources/src.sample.digital-outbound.source.v1.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "control/ingestion/pipeline/parsed/src.sample.digital-outbound.parsed.v1.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "control/ingestion/pipeline/normalized/src.sample.digital-outbound.normalized.v1.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "control/ingestion/pipeline/candidates/enterprise.sample.src-sample-digital-outbound.candidate.v1.json", json, sizeof(json)) != 0 ||
      require_surface(rt, "control/ingestion/pipeline/review/enterprise.sample.src-sample-digital-outbound.candidate.v1.review.v1.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical ingestion pipeline assets");
    return -1;
  }

  if (require_repo_file(rt, "control/contracts/control/control-plane.v1.json", json, sizeof(json)) != 0 ||
      require_repo_file(rt, "control/contracts/control/control-call.v1.json", json, sizeof(json)) != 0 ||
      require_repo_file(rt, "control/contracts/control/exec-reply.v1.json", json, sizeof(json)) != 0 ||
      require_repo_file(rt, "control/contracts/vault/vault-abi.v1.json", json, sizeof(json)) != 0 ||
      require_repo_file(rt, "model/registry/providers/providers.v1.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical protocol/registry contracts");
    return -1;
  }

  return 0;
}
