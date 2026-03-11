#include "../internal.h"

int yai_law_load_compliance_index(const yai_law_runtime_t *rt,
                                  char *out_json,
                                  size_t out_cap) {
  if (!rt || !out_json || out_cap == 0) return -1;

  if (yai_law_read_governance_surface_file(rt, "compliance/index/compliance.index.json", out_json, out_cap) == 0) {
    return 0;
  }

  /* Compatibility fallback for historical runtime payload shape. */
  return yai_law_read_governance_surface_file(rt, "overlays/regulatory/index/regulatory.index.json", out_json, out_cap);
}
