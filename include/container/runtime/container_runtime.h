#pragma once

#include <yai/con/bindings.h>
#include <yai/con/config.h>
#include <yai/con/grants.h>
#include <yai/con/identity.h>
#include <yai/con/lifecycle.h>
#include <yai/con/mounts.h>
#include <yai/con/paths.h>
#include <yai/con/policy.h>
#include <yai/con/recovery.h>
#include <yai/con/registry.h>
#include <yai/con/root.h>
#include <yai/con/runtime_view.h>
#include <yai/con/services.h>
#include <yai/con/session.h>
#include <yai/con/state.h>
#include <yai/con/tree.h>

int yai_container_create(const yai_container_record_t *record);
int yai_container_open(const char *container_id);
int yai_container_attach(const char *container_id, uint64_t session_id);
int yai_container_initialize(const char *container_id);
int yai_container_recover(const char *container_id, uint64_t reason_flags);
int yai_container_seal_runtime(const char *container_id, int64_t sealed_at);
int yai_container_destroy(const char *container_id, int64_t destroyed_at);

int yai_container_get_identity(const char *container_id, yai_container_identity_t *out_identity);
int yai_container_get_state(const char *container_id, yai_container_state_t *out_state);
int yai_container_get_root_view(const char *container_id, yai_container_root_t *out_root);
int yai_container_get_session_view(const char *container_id, yai_container_session_domain_t *out_session);
int yai_container_get_policy_view(const char *container_id, yai_container_policy_view_t *out_view);
int yai_container_get_grants_view(const char *container_id, yai_container_grants_view_t *out_view);
int yai_container_get_runtime_view(const char *container_id, yai_container_runtime_view_t *out_view);
