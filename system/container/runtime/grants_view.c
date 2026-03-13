#include <string.h>

#include "internal/model.h"
#include "yai/container/grants_view.h"

int yai_container_grants_view_get(const char *container_id,
                                  yai_container_grants_view_t *out_view) {
  yai_container_record_t record;

  if (!container_id || !out_view || container_id[0] == '\0') {
    return -1;
  }

  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }

  memset(out_view, 0, sizeof(*out_view));
  out_view->grants_view_handle = record.config.grants_profile;
  out_view->active_grants = record.state.services_online;
  out_view->suspended_grants = record.state.health_flags & 1u;
  out_view->revoked_grants = 0;
  return 0;
}
