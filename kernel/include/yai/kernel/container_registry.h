#pragma once

#include <stddef.h>
#include <stdint.h>

#include "config.h"
#include "grants_view.h"
#include "identity.h"
#include "lifecycle.h"
#include "policy_view.h"
#include "root.h"
#include "services.h"
#include "session.h"
#include "state.h"
#include "tree.h"

#define YAI_CONTAINER_REGISTRY_LIST_MAX 256u

typedef struct {
  yai_container_identity_t identity;
  yai_container_config_t config;
  yai_container_lifecycle_t lifecycle;
  yai_container_root_t root;
  yai_container_tree_t tree;
  yai_container_session_domain_t session_domain;
  yai_container_state_t state;
} yai_container_record_t;

typedef struct {
  char container_id[YAI_CONTAINER_ID_MAX + 1u];
  yai_container_class_t container_class;
  yai_container_lifecycle_state_t lifecycle_state;
  uint64_t root_handle;
  uint64_t runtime_view_handle;
  uint64_t policy_view_handle;
  uint64_t grants_view_handle;
  uint64_t service_surface_handle;
  yai_container_health_state_t health_state;
  yai_container_recovery_status_t recovery_state;
  uint64_t bound_session_count;
  uint64_t attached_daemon_count;
  int64_t created_at;
  int64_t updated_at;
} yai_container_registry_entry_t;

int yai_container_registry_register(const yai_container_record_t *record);
int yai_container_registry_update(const yai_container_record_t *record);
int yai_container_registry_get_record(const char *container_id,
                                      yai_container_record_t *out_record);
int yai_container_registry_lookup(const char *container_id,
                                  yai_container_registry_entry_t *out_entry);
int yai_container_registry_list(yai_container_registry_entry_t *entries,
                                size_t cap,
                                size_t *out_len);
int yai_container_registry_unregister(const char *container_id);
int yai_container_registry_set_lifecycle(const char *container_id,
                                         yai_container_lifecycle_state_t next_state,
                                         int64_t at);
