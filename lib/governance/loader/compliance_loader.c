#include "../internal.h"

int yai_governance_load_compliance_index(const yai_governance_runtime_t *rt,
                                  char *out_json,
                                  size_t out_cap) {
  if (!rt || !out_json || out_cap == 0) return -1;

  if (yai_governance_read_governance_surface_file(rt, "compliance/index/compliance.descriptors.index.json", out_json, out_cap) == 0) {
    return 0;
  }

  return yai_governance_read_governance_surface_file(rt, "compliance/index/compliance.index.json", out_json, out_cap);
}
