#pragma once

#include <stddef.h>

#include <yai/policy/governance/manifest.h>

typedef struct yai_governance_runtime {
  char root[512];
  yai_governance_manifest_t manifest;
  yai_governance_runtime_entrypoint_t entrypoint;
  yai_governance_runtime_view_t runtime_view;
  yai_governance_compatibility_t compatibility;
} yai_governance_runtime_t;

int yai_governance_load_runtime(yai_governance_runtime_t *out, char *err, size_t err_cap);
int yai_governance_load_domain_manifest(const yai_governance_runtime_t *rt,
                                 const char *domain_id,
                                 char *out_json,
                                 size_t out_cap);
int yai_governance_load_control_family_descriptor(const yai_governance_runtime_t *rt,
                                           const char *lookup_id,
                                           char *out_json,
                                           size_t out_cap);
int yai_governance_load_specialization_descriptor(const yai_governance_runtime_t *rt,
                                           const char *lookup_id,
                                           char *out_json,
                                           size_t out_cap);
int yai_governance_load_compliance_index(const yai_governance_runtime_t *rt,
                                  char *out_json,
                                  size_t out_cap);
int yai_governance_read_surface_json(const yai_governance_runtime_t *rt,
                              const char *rel_path,
                              char *out_json,
                              size_t out_cap);
