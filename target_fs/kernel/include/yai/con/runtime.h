#pragma once

#include <yai/container/bindings.h>
#include <yai/container/config.h>
#include <yai/container/grants.h>
#include <yai/container/identity.h>
#include <yai/container/lifecycle.h>
#include <yai/container/mounts.h>
#include <yai/container/paths.h>
#include <yai/container/policy.h>
#include <yai/container/recovery.h>
#include <yai/container/registry.h>
#include <yai/container/root.h>
#include <yai/container/runtime_view.h>
#include <yai/container/services.h>
#include <yai/container/session.h>
#include <yai/container/state.h>
#include <yai/container/tree.h>

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
