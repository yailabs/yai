#pragma once

#ifndef YAI_SECURITY_CONTAINMENT_H
#define YAI_SECURITY_CONTAINMENT_H

#include <stdint.h>
#include <yai/kernel/handles.h>
#include <yai/kernel/objects.h>

enum yai_containment_mode {
    YAI_CONTAINMENT_NONE = 0,
    YAI_CONTAINMENT_SOFT = 1,
    YAI_CONTAINMENT_SCOPED = 2,
    YAI_CONTAINMENT_CONTAINED = 3,
    YAI_CONTAINMENT_HARDENED = 4
};

enum yai_containment_state {
    YAI_CONTAINMENT_STATE_REQUESTED = 0,
    YAI_CONTAINMENT_STATE_ACTIVE = 1,
    YAI_CONTAINMENT_STATE_DEGRADED = 2,
    YAI_CONTAINMENT_STATE_BREACHED = 3,
    YAI_CONTAINMENT_STATE_SUSPENDED = 4,
    YAI_CONTAINMENT_STATE_REVOKED = 5
};

enum yai_escape_policy_class {
    YAI_ESCAPE_NONE = 0,
    YAI_ESCAPE_CONTROLLED_ADMIN = 1,
    YAI_ESCAPE_RECOVERY = 2,
    YAI_ESCAPE_DEBUG = 3
};

struct yai_security_containment_request {
    yai_object_id_t container_id;
    yai_object_id_t isolation_profile;
    enum yai_containment_mode mode;
    enum yai_escape_policy_class escape_policy;
    uint64_t flags;
};

struct yai_security_containment_state {
    yai_object_id_t container_id;
    yai_object_id_t isolation_profile;
    enum yai_containment_mode mode;
    yai_rootfs_handle_t rootfs_handle;
    yai_namespace_handles_t namespaces;
    yai_cgroup_id_t resource_group;
    enum yai_escape_policy_class escape_policy;
    enum yai_containment_state state;
    uint64_t flags;
};

int yai_security_containment_request(
    const struct yai_security_containment_request *request,
    struct yai_security_containment_state *out_state);

int yai_security_containment_activate(yai_object_id_t container_id, uint64_t flags);
int yai_security_containment_suspend(yai_object_id_t container_id, uint64_t flags);
int yai_security_containment_revoke(yai_object_id_t container_id, uint64_t flags);
int yai_security_containment_mark_breached(yai_object_id_t container_id, uint64_t flags);
int yai_security_containment_can_escape(yai_object_id_t container_id,
                                        enum yai_escape_policy_class requested_class);
int yai_security_containment_get(yai_object_id_t container_id,
                                 struct yai_security_containment_state *out_state);

#endif /* YAI_SECURITY_CONTAINMENT_H */
