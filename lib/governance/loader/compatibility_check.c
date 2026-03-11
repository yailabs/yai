#include "../internal.h"

#include <stdio.h>

static int yai_law_require_governance_surface(const yai_law_runtime_t *rt, const char *rel, char *json, size_t cap) {
  return yai_law_read_governance_surface_file(rt, rel, json, cap);
}

static int yai_law_require_file(const yai_law_runtime_t *rt, const char *rel, char *json, size_t cap) {
  char path[512];
  if (yai_law_safe_snprintf(path, sizeof(path), "%s/%s", rt->root, rel) != 0) return -1;
  return yai_law_read_text_file(path, json, cap);
}

int yai_law_compatibility_check(yai_law_runtime_t *rt, char *err, size_t err_cap) {
  char json[4096];

  if (!rt) return -1;

  if (yai_law_require_file(rt, "COMPATIBILITY.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_law_safe_snprintf(err, err_cap, "missing compatibility file");
    return -1;
  }

  (void)yai_law_json_extract_string(json, "compatibility_profile", rt->compatibility.profile, sizeof(rt->compatibility.profile));
  (void)yai_law_json_extract_string(json, "law_version", rt->compatibility.law_version, sizeof(rt->compatibility.law_version));

  if (rt->compatibility.profile[0] == 0) {
    (void)yai_law_safe_snprintf(rt->compatibility.profile, sizeof(rt->compatibility.profile), "%s", "runtime-consumer.v3");
  }

  if (yai_law_require_file(rt, "classification/classification-map.json", json, sizeof(json)) != 0 ||
      yai_law_require_file(rt, "control-families/index/families.index.json", json, sizeof(json)) != 0 ||
      yai_law_require_file(rt, "domain-specializations/index/specializations.index.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "compliance/index/compliance.index.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "overlays/regulatory/index/regulatory.index.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "overlays/sector/index/sector.index.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "overlays/contextual/index/contextual.index.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_law_safe_snprintf(err, err_cap, "missing six-layer runtime indexes");
    return -1;
  }

  if (yai_law_require_governance_surface(rt,
                                         "packs/compliance/gdpr-eu/2026Q1/pack.meta.json",
                                         json,
                                         sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_law_safe_snprintf(err, err_cap, "missing canonical compliance pack metadata");
    return -1;
  }

  if (yai_law_require_governance_surface(rt, "manifests/law.manifest.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "manifests/runtime.entrypoints.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "manifests/publish.index.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "manifests/publish.layers.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "manifests/compatibility.matrix.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "manifests/domain-resolution-order.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "manifests/compliance-resolution-order.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "manifests/governance-attachability.constraints.v1.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "manifests/customer-policy-packs.index.json", json, sizeof(json)) != 0 ||
      yai_law_require_governance_surface(rt, "manifests/enterprise-custom-governance.index.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_law_safe_snprintf(err, err_cap, "missing canonical governance manifest spine");
    return -1;
  }

  if (yai_law_require_file(rt, "generated/runtime-resolution-view.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_law_safe_snprintf(err, err_cap, "missing generated runtime-resolution-view");
    return -1;
  }

  if (yai_law_read_domain_model_matrix(json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_law_safe_snprintf(err, err_cap, "missing domain-model.matrix");
    return -1;
  }

  return 0;
}
