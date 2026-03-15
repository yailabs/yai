#include "yai/abi/errors.h"
#include "yai/kernel/grants.h"
#include "yai/kernel/policy.h"

static int map_check_to_policy_result(int check_rc, enum yai_kernel_policy_result* out_result) {
    if (out_result == 0) {
        return YAI_ERR_INVALID;
    }

    if (check_rc == YAI_OK) {
        *out_result = YAI_KERNEL_POLICY_ALLOW;
        return YAI_OK;
    }

    if (check_rc == YAI_ERR_DENIED) {
        *out_result = YAI_KERNEL_POLICY_DENY;
        return YAI_OK;
    }

    *out_result = YAI_KERNEL_POLICY_DEFER;
    return YAI_OK;
}

int yai_kernel_policy_can_admit_session(
    yai_object_id_t subject_handle,
    const struct yai_kernel_session_request* request,
    enum yai_kernel_policy_result* out_result) {
    int rc;
    struct yai_kernel_grant grant;

    if (request == 0 || out_result == 0) {
        return YAI_ERR_INVALID;
    }

    rc = yai_kernel_grant_check(subject_handle, YAI_KCAP_BIND_SESSION, 0, &grant);
    if (request->requested_privilege_class >= YAI_KERNEL_PRIV_SYSTEM && rc != YAI_OK) {
        *out_result = YAI_KERNEL_POLICY_REQUIRE_PRIVILEGED_PATH;
        return YAI_OK;
    }

    return map_check_to_policy_result(rc, out_result);
}

int yai_kernel_policy_can_bind_container(
    yai_object_id_t subject_handle,
    yai_object_id_t session_id,
    yai_object_id_t container_id,
    enum yai_kernel_policy_result* out_result) {
    int rc;
    struct yai_kernel_grant grant;

    (void)session_id;
    rc = yai_kernel_grant_check(subject_handle, YAI_KCAP_ENTER_CONTAINER, container_id, &grant);
    return map_check_to_policy_result(rc, out_result);
}

int yai_kernel_policy_can_mount(
    yai_object_id_t subject_handle,
    yai_object_id_t container_id,
    enum yai_mount_policy_class mount_policy,
    enum yai_kernel_policy_result* out_result) {
    int rc;
    struct yai_kernel_grant grant;

    (void)mount_policy;
    rc = yai_kernel_grant_check(subject_handle, YAI_KCAP_ATTACH_MOUNT, container_id, &grant);
    return map_check_to_policy_result(rc, out_result);
}

int yai_kernel_policy_can_escape(
    yai_object_id_t subject_handle,
    yai_object_id_t container_id,
    enum yai_escape_policy_class requested_class,
    enum yai_kernel_policy_result* out_result) {
    int rc;
    struct yai_kernel_grant grant;

    (void)requested_class;
    rc = yai_kernel_grant_check(subject_handle, YAI_KCAP_REQUEST_ESCAPE, container_id, &grant);
    return map_check_to_policy_result(rc, out_result);
}

int yai_kernel_policy_can_spawn(
    yai_object_id_t subject_handle,
    yai_object_id_t container_id,
    enum yai_kernel_policy_result* out_result) {
    int rc;
    struct yai_kernel_grant grant;

    rc = yai_kernel_grant_check(subject_handle, YAI_KCAP_SPAWN_DAEMON_BINDING, container_id, &grant);
    return map_check_to_policy_result(rc, out_result);
}
