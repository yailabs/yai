#include <string.h>

#include "internal/model.h"
#include <yai/con/policy.h>

int yai_container_policy_view_get(const char *container_id,
                                  yai_container_policy_view_t *out_view) {
  yai_container_record_t record;

  if (!container_id || !out_view || container_id[0] == '\0') {
    return -1;
  }

  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }

  memset(out_view, 0, sizeof(*out_view));
  out_view->policy_view_handle = record.config.policy_profile;
  out_view->active_rules = record.state.health_flags;
  out_view->deferred_rules = record.state.attachments;
  return 0;
}
