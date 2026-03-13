#include <string.h>

#include "internal/model.h"
#include <yai/container/runtime_view.h>

int yai_container_get_identity_view(const char *container_id,
                                    yai_container_identity_t *out_identity) {
  yai_container_record_t record;

  if (!container_id || !out_identity || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  *out_identity = record.identity;
  return 0;
}

int yai_container_get_state_view(const char *container_id,
                                 yai_container_state_t *out_state) {
  yai_container_record_t record;

  if (!container_id || !out_state || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  *out_state = record.state;
  return 0;
}

int yai_container_get_service_view(const char *container_id,
                                   yai_container_service_surface_t *out_service_view) {
  return yai_container_service_surface_get(container_id, out_service_view);
}

int yai_container_get_health_view(const char *container_id,
                                  yai_container_health_view_t *out_health_view) {
  yai_container_record_t record;

  if (!container_id || !out_health_view || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  memset(out_health_view, 0, sizeof(*out_health_view));
  out_health_view->health_state = record.state.health_state;
  out_health_view->degraded_flags = record.state.degraded_flags;
  out_health_view->last_error_class = record.state.last_error_class;
  return 0;
}

int yai_container_get_recovery_view(const char *container_id,
                                    yai_container_recovery_view_t *out_recovery_view) {
  yai_container_record_t record;
  yai_container_recovery_record_t recovery_record;

  if (!container_id || !out_recovery_view || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }

  memset(out_recovery_view, 0, sizeof(*out_recovery_view));
  out_recovery_view->recovery_status = record.state.recovery_status;
  out_recovery_view->recovery_reason_flags = record.state.recovery_reason_flags;
  if (yai_container_recovery_check(container_id, NULL, &recovery_record) == 0) {
    out_recovery_view->last_outcome = recovery_record.last_outcome;
    out_recovery_view->marker_flags = recovery_record.marker_flags;
  }
  return 0;
}

int yai_container_runtime_view_get(const char *container_id,
                                   yai_container_runtime_view_t *out_view) {
  if (!container_id || !out_view || container_id[0] == '\0') {
    return -1;
  }

  memset(out_view, 0, sizeof(*out_view));
  if (yai_container_get_identity_view(container_id, &out_view->identity) != 0) {
    return -1;
  }
  if (yai_container_get_state_view(container_id, &out_view->state) != 0) {
    return -1;
  }
  if (yai_container_get_service_view(container_id, &out_view->services) != 0) {
    return -1;
  }
  if (yai_container_policy_view_get(container_id, &out_view->policy_view) != 0) {
    return -1;
  }
  if (yai_container_grants_view_get(container_id, &out_view->grants_view) != 0) {
    return -1;
  }

  if (yai_container_get_health_view(container_id, &out_view->health) != 0) {
    return -1;
  }
  if (yai_container_get_recovery_view(container_id, &out_view->recovery) != 0) {
    return -1;
  }

  out_view->bindings.daemon_binding_handle = out_view->state.daemon_bindings;
  out_view->bindings.network_binding_handle = out_view->state.network_bindings;
  out_view->bindings.orchestration_binding_handle = out_view->state.attachments;
  return 0;
}
