#pragma once

#include <stdint.h>

#include <yai/con/paths.h>
#include <yai/con/root.h>
#include <yai/con/runtime_view.h>

typedef enum {
  YAI_CONTAINER_SESSION_MODE_GLOBAL = 0,
  YAI_CONTAINER_SESSION_MODE_NORMAL,
  YAI_CONTAINER_SESSION_MODE_PRIVILEGED,
  YAI_CONTAINER_SESSION_MODE_RECOVERY,
  YAI_CONTAINER_SESSION_MODE_DIAGNOSTIC,
} yai_container_session_mode_t;

typedef enum {
  YAI_CONTAINER_ESCAPE_NONE = 0,
  YAI_CONTAINER_ESCAPE_CONTROLLED_ADMIN,
  YAI_CONTAINER_ESCAPE_RECOVERY,
  YAI_CONTAINER_ESCAPE_DEBUG,
} yai_container_escape_policy_class_t;

typedef struct {
  uint64_t container_session_scope;
  uint64_t bound_session_count;
  uint64_t last_bound_session_id;
  uint64_t last_bound_at;
  uint64_t active_session_id;
  uint64_t root_handle;
  uint64_t runtime_view_handle;
  uint64_t capability_mask;
  uint64_t interactive_flags;
  yai_container_session_mode_t session_mode;
  yai_container_escape_policy_class_t escape_policy_class;
  uint8_t privileged_access;
  uint8_t bound;
} yai_container_session_domain_t;

typedef struct {
  uint64_t session_id;
  char bound_container_id[64];
  yai_container_session_mode_t session_mode;
  uint64_t root_handle;
  yai_container_path_context_t path_context;
  uint64_t runtime_view_handle;
  uint64_t capability_mask;
  yai_container_escape_policy_class_t escape_policy_class;
  uint64_t interactive_flags;
  uint8_t bound;
} yai_container_bound_session_t;

int yai_container_bind_session(const char *container_id,
                               uint64_t session_id,
                               yai_container_session_mode_t mode,
                               uint64_t capability_mask,
                               uint64_t interactive_flags);
int yai_container_unbind_session(const char *container_id, uint64_t session_id);
int yai_container_rebind_session(const char *container_id,
                                 uint64_t old_session_id,
                                 uint64_t new_session_id,
                                 yai_container_session_mode_t mode,
                                 uint64_t capability_mask,
                                 uint64_t interactive_flags);
int yai_container_session_enter(const char *container_id,
                                uint64_t session_id,
                                yai_container_bound_session_t *out_session);
int yai_container_session_leave(const char *container_id, uint64_t session_id);
int yai_container_session_get_root(const char *container_id,
                                   uint64_t session_id,
                                   yai_container_root_t *out_root);
int yai_container_session_get_path_context(const char *container_id,
                                           uint64_t session_id,
                                           yai_container_path_context_t *out_context);
int yai_container_session_get_runtime_view(const char *container_id,
                                           uint64_t session_id,
                                           yai_container_runtime_view_t *out_view);
int yai_container_session_can_escape(const char *container_id, uint64_t session_id);
int yai_container_session_request_escape(const char *container_id,
                                         uint64_t session_id,
                                         yai_container_escape_policy_class_t requested_class);
int yai_container_session_enter_recovery(const char *container_id,
                                         uint64_t session_id,
                                         uint64_t reason_flags);
