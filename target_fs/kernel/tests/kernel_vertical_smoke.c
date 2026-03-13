#include <stdio.h>

#include "yai/abi/errors.h"
#include <yai/security/containment.h>
#include "yai/kernel/grants.h"
#include "yai/kernel/kernel.h"
#include "yai/kernel/lifecycle.h"
#include "yai/kernel/session.h"
#include "yai/kernel/state.h"

static int expect_ok(const char* step, int rc) {
    if (rc != YAI_OK) {
        fprintf(stderr, "kernel-smoke: %s failed rc=%d\n", step, rc);
        return 1;
    }
    return 0;
}

int main(void) {
    const yai_object_id_t container_id = 0x2001u;
    const yai_object_id_t session_id = 0x3001u;

    struct yai_kernel_grant grant;
    struct yai_kernel_grant_request grant_req;
    struct yai_kernel_session_request session_req;
    struct yai_kernel_session admitted;
    struct yai_security_containment_request containment_req;
    struct yai_security_containment_state containment_state;
    const struct yai_kernel_state* kernel_state;

    int rc;

    yai_kernel_bootstrap(0x11u);
    kernel_state = yai_kernel_state_get();
    if (kernel_state->lifecycle.current_state != YAI_KERNEL_STATE_READY) {
        fprintf(stderr, "kernel-smoke: kernel not READY after bootstrap\n");
        return 1;
    }

    grant_req.grant_id = 0x5001u;
    grant_req.subject_handle = session_id;
    grant_req.capability_class = YAI_KCAP_BIND_SESSION;
    grant_req.scope_handle = 0;
    grant_req.issued_at = 100;
    grant_req.expires_at = 0;
    grant_req.flags = 0;
    if (expect_ok("grant issue bind-session", yai_kernel_grant_issue(&grant_req, &grant))) return 1;
    if (expect_ok("grant activate bind-session", yai_kernel_grant_activate(grant.grant_id))) return 1;

    grant_req.grant_id = 0x5002u;
    grant_req.subject_handle = session_id;
    grant_req.capability_class = YAI_KCAP_ENTER_CONTAINER;
    grant_req.scope_handle = container_id;
    grant_req.issued_at = 101;
    grant_req.expires_at = 0;
    grant_req.flags = 0;
    if (expect_ok("grant issue enter-container", yai_kernel_grant_issue(&grant_req, &grant))) return 1;
    if (expect_ok("grant activate enter-container", yai_kernel_grant_activate(grant.grant_id))) return 1;

    grant_req.grant_id = 0x5003u;
    grant_req.subject_handle = container_id;
    grant_req.capability_class = YAI_KCAP_REQUEST_ESCAPE;
    grant_req.scope_handle = container_id;
    grant_req.issued_at = 102;
    grant_req.expires_at = 0;
    grant_req.flags = 0;
    if (expect_ok("grant issue escape", yai_kernel_grant_issue(&grant_req, &grant))) return 1;
    if (expect_ok("grant activate escape", yai_kernel_grant_activate(grant.grant_id))) return 1;

    containment_req.container_id = container_id;
    containment_req.isolation_profile = 1;
    containment_req.mode = YAI_CONTAINMENT_CONTAINED;
    containment_req.escape_policy = YAI_ESCAPE_CONTROLLED_ADMIN;
    containment_req.flags = 0;
    if (expect_ok("containment request", yai_security_containment_request(&containment_req, &containment_state))) return 1;
    if (containment_state.state != YAI_CONTAINMENT_STATE_REQUESTED) {
        fprintf(stderr, "kernel-smoke: containment not REQUESTED after request\n");
        return 1;
    }
    if (expect_ok("containment activate", yai_security_containment_activate(container_id, 0))) return 1;
    if (expect_ok("containment get", yai_security_containment_get(container_id, &containment_state))) return 1;
    if (containment_state.state != YAI_CONTAINMENT_STATE_ACTIVE) {
        fprintf(stderr, "kernel-smoke: containment not ACTIVE after activate\n");
        return 1;
    }

    session_req.requested_type = YAI_KERNEL_SESSION_GLOBAL;
    session_req.requested_privilege_class = YAI_KERNEL_PRIV_OPERATOR;
    session_req.admission_source = 1;
    session_req.capability_mask = YAI_KCAP_BIND_SESSION | YAI_KCAP_ENTER_CONTAINER;
    session_req.created_at = 110;
    session_req.flags = 0;
    if (expect_ok("session admit", yai_kernel_session_admit(session_id, &session_req, &admitted))) return 1;
    if (expect_ok("session bind container", yai_kernel_session_bind_container(session_id, container_id))) return 1;

    if (expect_ok("escape check", yai_security_containment_can_escape(container_id, YAI_ESCAPE_CONTROLLED_ADMIN))) return 1;

    if (expect_ok("grant revoke escape", yai_kernel_grant_revoke(0x5003u, 200, 0))) return 1;
    rc = yai_security_containment_can_escape(container_id, YAI_ESCAPE_CONTROLLED_ADMIN);
    if (rc == YAI_OK) {
        fprintf(stderr, "kernel-smoke: escape should be denied after grant revoke\n");
        return 1;
    }

    printf("kernel vertical smoke: ok\n");
    return 0;
}
