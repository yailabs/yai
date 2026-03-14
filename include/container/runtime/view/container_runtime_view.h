#pragma once

#include <yai/con/identity.h>
#include <yai/con/bindings.h>
#include <yai/con/grants.h>
#include <yai/con/policy.h>
#include <yai/con/services.h>
#include <yai/con/state.h>
#include <yai/con/recovery.h>

typedef struct {
  yai_container_health_state_t health_state;
  uint64_t degraded_flags;
  uint64_t last_error_class;
} yai_container_health_view_t;

typedef struct {
  yai_container_recovery_status_t recovery_status;
  uint64_t recovery_reason_flags;
  yai_container_recovery_outcome_t last_outcome;
  uint64_t marker_flags;
} yai_container_recovery_view_t;

typedef struct {
  yai_container_identity_t identity;
  yai_container_state_t state;
  yai_container_service_surface_t services;
  yai_container_policy_view_t policy_view;
  yai_container_grants_view_t grants_view;
  yai_container_health_view_t health;
  yai_container_recovery_view_t recovery;
  yai_container_bindings_t bindings;
} yai_container_runtime_view_t;

int yai_container_runtime_view_get(const char *container_id,
                                   yai_container_runtime_view_t *out_view);
int yai_container_get_identity_view(const char *container_id,
                                    yai_container_identity_t *out_identity);
int yai_container_get_state_view(const char *container_id,
                                 yai_container_state_t *out_state);
int yai_container_get_service_view(const char *container_id,
                                   yai_container_service_surface_t *out_service_view);
int yai_container_get_health_view(const char *container_id,
                                  yai_container_health_view_t *out_health_view);
int yai_container_get_recovery_view(const char *container_id,
                                    yai_container_recovery_view_t *out_recovery_view);
