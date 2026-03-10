#include "../internal.h"

int yai_law_overlay_loader_validate(const yai_law_runtime_t *rt, char *err, size_t err_cap) {
  char path[768];
  /*
   * Lightweight runtime validation for overlay surfaces.
   * Overlay materialization itself is performed in stack_builder using
   * per-domain and per-specialization matrices.
   */
  char buf[8192];
  if (err && err_cap > 0) err[0] = '\0';
  if (!rt || !rt->root[0]) {
    if (err && err_cap > 0) (void)yai_law_safe_snprintf(err, err_cap, "%s", "overlay_loader_missing_root");
    return -1;
  }
  if (yai_law_safe_snprintf(path, sizeof(path), "%s/overlays/regulatory/index/overlay-attachment-matrix.json", rt->root) != 0 ||
      yai_law_read_text_file(path, buf, sizeof(buf)) != 0) {
    if (err && err_cap > 0) (void)yai_law_safe_snprintf(err, err_cap, "%s", "overlay_regulatory_index_missing");
    return -1;
  }
  if (yai_law_safe_snprintf(path, sizeof(path), "%s/overlays/sector/index/overlay-attachment-matrix.json", rt->root) != 0 ||
      yai_law_read_text_file(path, buf, sizeof(buf)) != 0) {
    if (err && err_cap > 0) (void)yai_law_safe_snprintf(err, err_cap, "%s", "overlay_sector_index_missing");
    return -1;
  }
  return 0;
}
