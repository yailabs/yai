#include "yai/container/container.h"
#include "internal/model.h"

int yai_container_get_identity(const char *container_id, yai_container_identity_t *out_identity) {
  yai_container_record_t record;
  if (!out_identity) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  *out_identity = record.identity;
  return 0;
}

int yai_container_get_state(const char *container_id, yai_container_state_t *out_state) {
  yai_container_record_t record;
  if (!out_state) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  *out_state = record.state;
  return 0;
}

int yai_container_get_root_view(const char *container_id, yai_container_root_t *out_root) {
  yai_container_record_t record;
  if (!out_root) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  *out_root = record.root;
  return 0;
}

int yai_container_get_session_view(const char *container_id, yai_container_session_domain_t *out_session) {
  yai_container_record_t record;
  if (!out_session) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  *out_session = record.session_domain;
  return 0;
}

int yai_container_get_policy_view(const char *container_id, yai_container_policy_view_t *out_view) {
  return yai_container_policy_view_get(container_id, out_view);
}

int yai_container_get_grants_view(const char *container_id, yai_container_grants_view_t *out_view) {
  return yai_container_grants_view_get(container_id, out_view);
}

int yai_container_get_runtime_view(const char *container_id, yai_container_runtime_view_t *out_view) {
  return yai_container_runtime_view_get(container_id, out_view);
}
