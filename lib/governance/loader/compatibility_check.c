#include "../internal.h"

#include <stdio.h>

static int yai_governance_require_governance_surface(const yai_governance_runtime_t *rt, const char *rel, char *json, size_t cap) {
  return yai_governance_read_governance_surface_file(rt, rel, json, cap);
}

static int yai_governance_require_file(const yai_governance_runtime_t *rt, const char *rel, char *json, size_t cap) {
  char path[512];
  if (yai_governance_safe_snprintf(path, sizeof(path), "%s/%s", rt->root, rel) != 0) return -1;
  return yai_governance_read_text_file(path, json, cap);
}

int yai_governance_compatibility_check(yai_governance_runtime_t *rt, char *err, size_t err_cap) {
  char json[4096];

  if (!rt) return -1;

  if (yai_governance_require_governance_surface(rt, "manifests/compatibility.matrix.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical compatibility matrix");
    return -1;
  }

  (void)yai_governance_json_extract_string(json, "schema_set", rt->compatibility.profile, sizeof(rt->compatibility.profile));
  (void)yai_governance_json_extract_string(json, "governance_version", rt->compatibility.governance_version, sizeof(rt->compatibility.governance_version));

  if (rt->compatibility.profile[0] == 0) {
    (void)yai_governance_safe_snprintf(rt->compatibility.profile, sizeof(rt->compatibility.profile), "%s", "runtime-consumer.v4");
  }

  if (yai_governance_require_file(rt, "classification/classification-map.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "families/index/families.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "families/index/families.descriptors.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "families/index/family.matrix.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "families/schema/family-descriptor.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "families/schema/family-registry-entry.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "families/schema/family-hierarchy-node.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "families/schema/family-matrix-entry.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "specializations/index/specializations.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "specializations/index/specializations.descriptors.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "specializations/index/specialization.matrix.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "specializations/schema/specialization-descriptor.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "specializations/schema/specialization-registry-entry.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "specializations/schema/specialization-matrix-entry.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "specializations/templates/specialization.descriptor.template.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "specializations/templates/specialization.bundle.template.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "compliance/index/compliance.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "compliance/index/compliance.descriptors.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "compliance/index/compliance.matrix.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "compliance/schema/compliance-descriptor.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "compliance/schema/compliance-registry-entry.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "compliance/schema/compliance-matrix-entry.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "overlays/regulatory/index/regulatory.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "overlays/sector/index/sector.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "overlays/contextual/index/contextual.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "overlays/index/overlays.descriptors.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "overlays/index/overlays.matrix.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "overlays/schema/overlay-descriptor.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "overlays/schema/overlay-registry-entry.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "overlays/schema/overlay-matrix-entry.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "overlays/matrices/overlay-attachment.matrix.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "overlays/matrices/overlay-precedence.matrix.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "overlays/matrices/overlay-evidence.matrix.v1.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical compliance-overlay descriptor surfaces");
    return -1;
  }

  if (yai_governance_require_governance_surface(rt,
                                         "compliance/materialized/packs/compliance/gdpr-eu/2026Q1/pack.meta.json",
                                         json,
                                         sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical compliance pack metadata");
    return -1;
  }

  if (yai_governance_require_governance_surface(rt, "manifests/governance.manifest.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "manifests/runtime.entrypoints.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "manifests/publish.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "manifests/publish.layers.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "manifests/compatibility.matrix.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "manifests/domain-resolution-order.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "manifests/compliance-resolution-order.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "manifests/governance-attachability.constraints.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "manifests/customer-policy-packs.index.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "manifests/enterprise-custom-governance.index.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical governance manifest spine");
    return -1;
  }

  if (yai_governance_require_file(rt, "../lib/protocol/contracts/schema/control/control_plane.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "../lib/protocol/contracts/schema/control/control_call.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "../lib/protocol/contracts/schema/control/exec_reply.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "../lib/protocol/contracts/schema/providers/providers.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_file(rt, "../lib/protocol/contracts/schema/vault/vault_abi.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical protocol contracts");
    return -1;
  }

  if (yai_governance_require_governance_surface(rt, "schema/policy.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "schema/decision_record.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "schema/governance_review_state.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "schema/evidence_index.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "schema/workspace_governance_attachment.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "schema/retention_policy.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "schema/containment_metrics.v1.schema.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "schema/verification_report.v1.schema.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical governance schema");
    return -1;
  }

  if (yai_governance_require_governance_surface(rt, "ingestion/templates/governance_source.template.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "ingestion/sources/src.sample.digital-outbound.source.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "ingestion/parsed/src.sample.digital-outbound.parsed.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "ingestion/normalized/src.sample.digital-outbound.normalized.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "ingestion/candidates/enterprise.sample.src-sample-digital-outbound.candidate.v1.json", json, sizeof(json)) != 0 ||
      yai_governance_require_governance_surface(rt, "ingestion/review/enterprise.sample.src-sample-digital-outbound.candidate.v1.review.v1.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical governance ingestion pipeline assets");
    return -1;
  }

  if (yai_governance_require_governance_surface(rt, "overlays/index/overlay-compliance.runtime.v1.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing canonical overlay-compliance runtime view");
    return -1;
  }

  if (yai_governance_read_domain_model_matrix(json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "missing domain-model.matrix");
    return -1;
  }

  return 0;
}
