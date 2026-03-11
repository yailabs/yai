#include "../internal.h"

#include <stdio.h>
#include <string.h>

static int yai_law_read_manifest_surface(const yai_law_runtime_t *rt,
                                         const char *name,
                                         char *out,
                                         size_t out_cap) {
  char rel[256];
  if (!name || !out || out_cap == 0) return -1;
  if (yai_law_safe_snprintf(rel, sizeof(rel), "manifests/%s", name) != 0) return -1;
  return yai_law_read_governance_surface_file(rt, rel, out, out_cap);
}

static void yai_law_canonicalize_governance_ref(char *ref, size_t ref_cap) {
  char tmp[512];
  if (!ref || ref_cap == 0 || ref[0] == '\0') return;
  if (strncmp(ref, "governance/", 11) == 0) return;
  if (yai_law_safe_snprintf(tmp, sizeof(tmp), "governance/%s", ref) != 0) return;
  (void)yai_law_safe_snprintf(ref, ref_cap, "%s", tmp);
}

int yai_law_manifest_load(yai_law_runtime_t *rt, char *err, size_t err_cap) {
  char json[4096];

  if (!rt) return -1;

  if (yai_law_read_manifest_surface(rt, "law.manifest.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_law_safe_snprintf(err, err_cap, "cannot read governance/manifests/law.manifest.json");
    return -1;
  }

  (void)yai_law_json_extract_string(json, "law_version", rt->manifest.law_version, sizeof(rt->manifest.law_version));
  (void)yai_law_json_extract_string(json, "domain_index_ref", rt->manifest.domain_index_ref, sizeof(rt->manifest.domain_index_ref));
  (void)yai_law_json_extract_string(json, "compliance_index_ref", rt->manifest.compliance_index_ref, sizeof(rt->manifest.compliance_index_ref));
  (void)yai_law_json_extract_string(json, "resolution_entrypoints_ref", rt->manifest.resolution_entrypoints_ref, sizeof(rt->manifest.resolution_entrypoints_ref));
  yai_law_canonicalize_governance_ref(rt->manifest.domain_index_ref, sizeof(rt->manifest.domain_index_ref));
  yai_law_canonicalize_governance_ref(rt->manifest.compliance_index_ref, sizeof(rt->manifest.compliance_index_ref));
  yai_law_canonicalize_governance_ref(rt->manifest.resolution_entrypoints_ref,
                                      sizeof(rt->manifest.resolution_entrypoints_ref));

  if (yai_law_read_manifest_surface(rt, "runtime.entrypoints.json", json, sizeof(json)) != 0) {
    if (err && err_cap) (void)yai_law_safe_snprintf(err, err_cap, "cannot read governance/manifests/runtime.entrypoints.json");
    return -1;
  }

  (void)yai_law_json_extract_string(json, "entrypoint_id", rt->entrypoint.entrypoint_id, sizeof(rt->entrypoint.entrypoint_id));
  (void)yai_law_json_extract_string(json, "law_manifest_ref", rt->entrypoint.law_manifest_ref, sizeof(rt->entrypoint.law_manifest_ref));
  (void)yai_law_json_extract_string(json, "domains_ref", rt->entrypoint.domains_ref, sizeof(rt->entrypoint.domains_ref));
  (void)yai_law_json_extract_string(json, "compliance_ref", rt->entrypoint.compliance_ref, sizeof(rt->entrypoint.compliance_ref));
  (void)yai_law_json_extract_string(json, "resolution_order_ref", rt->entrypoint.resolution_order_ref, sizeof(rt->entrypoint.resolution_order_ref));
  (void)yai_law_json_extract_string(json, "compatibility_ref", rt->entrypoint.compatibility_ref, sizeof(rt->entrypoint.compatibility_ref));
  yai_law_canonicalize_governance_ref(rt->entrypoint.law_manifest_ref, sizeof(rt->entrypoint.law_manifest_ref));
  yai_law_canonicalize_governance_ref(rt->entrypoint.domains_ref, sizeof(rt->entrypoint.domains_ref));
  yai_law_canonicalize_governance_ref(rt->entrypoint.compliance_ref, sizeof(rt->entrypoint.compliance_ref));
  yai_law_canonicalize_governance_ref(rt->entrypoint.resolution_order_ref, sizeof(rt->entrypoint.resolution_order_ref));
  yai_law_canonicalize_governance_ref(rt->entrypoint.compatibility_ref, sizeof(rt->entrypoint.compatibility_ref));

  if (rt->entrypoint.entrypoint_id[0] == '\0') {
    (void)yai_law_safe_snprintf(rt->entrypoint.entrypoint_id, sizeof(rt->entrypoint.entrypoint_id), "%s", "default-runtime");
  }

  (void)yai_law_safe_snprintf(rt->runtime_view.domain_resolution_ref,
                              sizeof(rt->runtime_view.domain_resolution_ref),
                              "%s",
                              "governance/manifests/domain-resolution-order.json");
  (void)yai_law_safe_snprintf(rt->runtime_view.compliance_resolution_ref,
                              sizeof(rt->runtime_view.compliance_resolution_ref),
                              "%s",
                              "governance/manifests/compliance-resolution-order.json");

  return 0;
}
