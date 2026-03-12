/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/sdk/transport.h>
#include <yai/sdk/paths.h>

#include <stdio.h>
#include <string.h>

static int starts_with(const char *s, const char *prefix)
{
    if (!s || !prefix) {
        return 0;
    }
    return strncmp(s, prefix, strlen(prefix)) == 0;
}

void yai_sdk_runtime_endpoint_init(yai_sdk_runtime_endpoint_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
    out->endpoint_kind = YAI_SDK_ENDPOINT_KIND_LOCAL_DEFAULT;
    out->transport_kind = YAI_SDK_TRANSPORT_KIND_UNSPECIFIED;
}

void yai_sdk_runtime_locator_state_init(yai_sdk_runtime_locator_state_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
    out->endpoint_kind = YAI_SDK_ENDPOINT_KIND_LOCAL_DEFAULT;
    out->transport_kind = YAI_SDK_TRANSPORT_KIND_UNSPECIFIED;
}

int yai_sdk_runtime_endpoint_local_default(yai_sdk_runtime_endpoint_t *out)
{
    if (!out) {
        return -1;
    }
    yai_sdk_runtime_endpoint_init(out);
    out->endpoint_kind = YAI_SDK_ENDPOINT_KIND_LOCAL_DEFAULT;
    out->transport_kind = YAI_SDK_TRANSPORT_KIND_UDS;
    return 0;
}

int yai_sdk_runtime_endpoint_local_uds(const char *uds_path, yai_sdk_runtime_endpoint_t *out)
{
    if (!out || !uds_path || !uds_path[0]) {
        return -1;
    }
    yai_sdk_runtime_endpoint_init(out);
    out->endpoint_kind = YAI_SDK_ENDPOINT_KIND_LOCAL_UDS;
    out->transport_kind = YAI_SDK_TRANSPORT_KIND_UDS;
    if (snprintf(out->locator, sizeof(out->locator), "%s", uds_path) <= 0) {
        return -1;
    }
    return 0;
}

int yai_sdk_runtime_endpoint_owner_ref(const char *owner_ref, yai_sdk_runtime_endpoint_t *out)
{
    if (!out || !owner_ref || !owner_ref[0]) {
        return -1;
    }
    yai_sdk_runtime_endpoint_init(out);
    out->endpoint_kind = YAI_SDK_ENDPOINT_KIND_OWNER_REF;
    if (snprintf(out->owner_ref, sizeof(out->owner_ref), "%s", owner_ref) <= 0) {
        return -1;
    }
    if (starts_with(owner_ref, "uds://")) {
        out->transport_kind = YAI_SDK_TRANSPORT_KIND_UDS;
    } else {
        out->transport_kind = YAI_SDK_TRANSPORT_KIND_UNSPECIFIED;
    }
    return 0;
}

int yai_sdk_runtime_endpoint_resolve(
    const yai_sdk_runtime_endpoint_t *endpoint,
    yai_sdk_runtime_locator_state_t *out_state,
    char *err,
    size_t err_cap)
{
    yai_sdk_runtime_endpoint_t local;
    const yai_sdk_runtime_endpoint_t *ep = endpoint;
    const char *owner_ref = NULL;
    const char *path = NULL;

    if (out_state) {
        yai_sdk_runtime_locator_state_init(out_state);
    }
    if (err && err_cap > 0) {
        err[0] = '\0';
    }

    if (!ep) {
        yai_sdk_runtime_endpoint_local_default(&local);
        ep = &local;
    }

    if (ep->endpoint_kind == YAI_SDK_ENDPOINT_KIND_LOCAL_DEFAULT) {
        if (yai_path_runtime_ingress_sock(out_state ? out_state->resolved_ingress : local.locator,
                                          out_state ? sizeof(out_state->resolved_ingress) : sizeof(local.locator)) != 0) {
            if (err && err_cap > 0) {
                (void)snprintf(err, err_cap, "%s", "runtime_ingress_unresolved");
            }
            return -2;
        }
        if (out_state) {
            out_state->endpoint_kind = ep->endpoint_kind;
            out_state->transport_kind = YAI_SDK_TRANSPORT_KIND_UDS;
            out_state->explicit_target = 0;
        }
        return 0;
    }

    if (ep->endpoint_kind == YAI_SDK_ENDPOINT_KIND_LOCAL_UDS) {
        if (!ep->locator[0]) {
            if (err && err_cap > 0) {
                (void)snprintf(err, err_cap, "%s", "runtime_endpoint_locator_empty");
            }
            return -2;
        }
        if (out_state) {
            out_state->endpoint_kind = ep->endpoint_kind;
            out_state->transport_kind = YAI_SDK_TRANSPORT_KIND_UDS;
            out_state->explicit_target = 1;
            (void)snprintf(out_state->resolved_ingress, sizeof(out_state->resolved_ingress), "%s", ep->locator);
        }
        return 0;
    }

    if (ep->endpoint_kind == YAI_SDK_ENDPOINT_KIND_OWNER_REF) {
        owner_ref = ep->owner_ref;
        if (!owner_ref[0]) {
            if (err && err_cap > 0) {
                (void)snprintf(err, err_cap, "%s", "runtime_owner_ref_empty");
            }
            return -2;
        }
        if (starts_with(owner_ref, "uds://")) {
            path = owner_ref + strlen("uds://");
            if (!path[0]) {
                if (err && err_cap > 0) {
                    (void)snprintf(err, err_cap, "%s", "runtime_owner_ref_invalid");
                }
                return -2;
            }
            if (out_state) {
                out_state->endpoint_kind = ep->endpoint_kind;
                out_state->transport_kind = YAI_SDK_TRANSPORT_KIND_UDS;
                out_state->explicit_target = 1;
                (void)snprintf(out_state->resolved_ingress, sizeof(out_state->resolved_ingress), "%s", path);
            }
            return 0;
        }
        if (err && err_cap > 0) {
            (void)snprintf(err, err_cap, "%s", "runtime_transport_unsupported");
        }
        return -3;
    }

    if (err && err_cap > 0) {
        (void)snprintf(err, err_cap, "%s", "runtime_endpoint_kind_unsupported");
    }
    return -3;
}
