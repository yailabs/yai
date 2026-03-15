#include <string.h>

#include "yai/abi/errors.h"
#include <yai/con/cgroup.h>
#include <yai/con/namespace.h>
#include <yai/con/rootfs.h>
#include <yai/sec/containment.h>
#include "yai/kernel/lifecycle.h"
#include "yai/kernel/mount_policy.h"
#include "yai/kernel/policy.h"
#include "yai/kernel/state.h"

int yai_security_containment_request(
    const struct yai_security_containment_request* request,
    struct yai_security_containment_state* out_state) {
    struct yai_security_containment_state state;
    int rc;

    if (request == 0 || out_state == 0 || request->container_id == 0) {
        return YAI_ERR_INVALID;
    }

    rc = yai_kernel_can_create_container();
    if (rc != YAI_OK) {
        return rc;
    }

    memset(&state, 0, sizeof(state));
    state.container_id = request->container_id;
    state.isolation_profile = request->isolation_profile;
    state.mode = request->mode;
    state.escape_policy = request->escape_policy;
    state.state = YAI_CONTAINMENT_STATE_REQUESTED;
    state.flags = request->flags;

    rc = yai_kernel_namespace_allocate_set(&state.namespaces);
    if (rc != YAI_OK) {
        return rc;
    }

    rc = yai_kernel_rootfs_project(request->flags, &state.rootfs_handle);
    if (rc != YAI_OK) {
        return rc;
    }

    state.resource_group = request->container_id;
    rc = yai_kernel_cgroup_attach(state.resource_group, request->flags);
    if (rc != YAI_OK) {
        return rc;
    }

    rc = yai_kernel_mounts_validate_policy(YAI_MOUNT_POLICY_SCOPED_RW, request->flags);
    if (rc != YAI_OK) {
        return rc;
    }

    rc = yai_containment_registry_set(&state);
    if (rc != YAI_OK) {
        return rc;
    }

    yai_kernel_state_inc_containers();
    *out_state = state;
    return YAI_OK;
}

int yai_security_containment_activate(yai_object_id_t container_id, uint64_t flags) {
    yai_containment_registry_entry state;
    int rc;

    rc = yai_kernel_can_bind_container();
    if (rc != YAI_OK) {
        return rc;
    }

    rc = yai_containment_registry_get(container_id, &state);
    if (rc != YAI_OK) {
        return rc;
    }

    if (state.state != YAI_CONTAINMENT_STATE_REQUESTED &&
        state.state != YAI_CONTAINMENT_STATE_DEGRADED) {
        return YAI_ERR_DENIED;
    }

    state.state = YAI_CONTAINMENT_STATE_ACTIVE;
    state.flags |= flags;
    rc = yai_containment_registry_set(&state);
    if (rc != YAI_OK) {
        return rc;
    }

    return YAI_OK;
}

int yai_security_containment_suspend(yai_object_id_t container_id, uint64_t flags) {
    yai_containment_registry_entry state;
    int rc;

    rc = yai_containment_registry_get(container_id, &state);
    if (rc != YAI_OK) {
        return rc;
    }

    state.state = YAI_CONTAINMENT_STATE_SUSPENDED;
    state.flags |= flags;
    rc = yai_containment_registry_set(&state);
    if (rc != YAI_OK) {
        return rc;
    }

    return YAI_OK;
}

int yai_security_containment_revoke(yai_object_id_t container_id, uint64_t flags) {
    yai_containment_registry_entry state;
    int rc;

    rc = yai_containment_registry_get(container_id, &state);
    if (rc != YAI_OK) {
        return rc;
    }

    state.state = YAI_CONTAINMENT_STATE_REVOKED;
    state.flags |= flags;
    rc = yai_containment_registry_set(&state);
    if (rc != YAI_OK) {
        return rc;
    }

    return YAI_OK;
}

int yai_security_containment_mark_breached(yai_object_id_t container_id, uint64_t flags) {
    yai_containment_registry_entry state;
    int rc;

    rc = yai_containment_registry_get(container_id, &state);
    if (rc != YAI_OK) {
        return rc;
    }

    state.state = YAI_CONTAINMENT_STATE_BREACHED;
    state.flags |= flags;
    rc = yai_containment_registry_set(&state);
    if (rc != YAI_OK) {
        return rc;
    }

    return YAI_OK;
}

int yai_security_containment_can_escape(yai_object_id_t container_id, enum yai_escape_policy_class requested_class) {
    yai_containment_registry_entry state;
    enum yai_kernel_policy_result policy_result;
    int rc;

    rc = yai_containment_registry_get(container_id, &state);
    if (rc != YAI_OK) {
        return rc;
    }

    rc = yai_kernel_policy_can_escape(container_id, container_id, requested_class, &policy_result);
    if (rc != YAI_OK) {
        return rc;
    }

    if (policy_result != YAI_KERNEL_POLICY_ALLOW) {
        return YAI_ERR_DENIED;
    }

    if (state.escape_policy >= requested_class) {
        return YAI_OK;
    }

    return YAI_ERR_DENIED;
}

int yai_security_containment_get(yai_object_id_t container_id, struct yai_security_containment_state* out_state) {
    return yai_containment_registry_get(container_id, out_state);
}
