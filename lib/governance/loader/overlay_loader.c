#include "../internal.h"

int yai_governance_overlay_loader_validate(const yai_governance_runtime_t *rt, char *err, size_t err_cap) {
  /*
   * Lightweight runtime validation for overlay surfaces.
   * Overlay materialization itself is performed in stack_builder using
   * per-domain and per-specialization matrices.
   */
  char buf[8192];
  if (err && err_cap > 0) err[0] = '\0';
  if (!rt) {
    if (err && err_cap > 0) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "overlay_loader_missing_root");
    return -1;
  }
  if (yai_governance_read_governance_surface_file(rt,
                                           "overlays/index/overlays.descriptors.index.json",
                                           buf,
                                           sizeof(buf)) != 0) {
    if (err && err_cap > 0) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "overlay_descriptors_missing");
    return -1;
  }
  if (yai_governance_read_governance_surface_file(rt,
                                           "overlays/matrices/overlay-attachment.matrix.v1.json",
                                           buf,
                                           sizeof(buf)) != 0) {
    if (err && err_cap > 0) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "overlay_attachment_matrix_missing");
    return -1;
  }
  if (yai_governance_read_governance_surface_file(rt,
                                           "overlays/matrices/overlay-precedence.matrix.v1.json",
                                           buf,
                                           sizeof(buf)) != 0) {
    if (err && err_cap > 0) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "overlay_precedence_matrix_missing");
    return -1;
  }
  if (yai_governance_read_governance_surface_file(rt,
                                           "overlays/matrices/overlay-evidence.matrix.v1.json",
                                           buf,
                                           sizeof(buf)) != 0) {
    if (err && err_cap > 0) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "overlay_evidence_matrix_missing");
    return -1;
  }
  return 0;
}
